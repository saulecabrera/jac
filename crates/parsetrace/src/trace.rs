use std::str::FromStr;

use anyhow::{Error, Result, bail};

/// Represents a single trace event profiled during the execution of the quickjs engine as a wasm module.
/// The profiler captures source level bytecode execution events and fuel consumption.
#[derive(Debug, Clone)]
pub enum BytecodeTraceEvent {
    /// The full execution of a single source opcode, including the fuel consumption
    /// and any native calls made to execute the opcode logic. The trace format is:
    ///
    /// `recovered_func_id,opcode_offset,opcode_byte,fuel_consumption,native_calls`
    ///
    /// `native_calls` captures the wasm function call/return traces within opcode execution.
    /// The contains wasm function start/end events separated by '|'. Each event has the format:
    /// `wasm_func_id,[S|E],fuel_watermark`
    ///
    /// `wasm_func_id` is the wasm function id, `S` indicates function start, `E` indicates function end,
    /// and `fuel_watermark` is the (fuel level at the event - fuel level at opcode start). The events
    /// represented by `native_calls` should be well formed and properly nested.
    OpcodeRun {
        recovered_func_id: u32,
        opcode_offset: u32,
        opcode_byte: u8,
        fuel_consumption: u32,
        #[allow(dead_code)]
        native_calls: Vec<WasmCallFrame>,
    },
    /// Source function start event, trace format is:
    ///
    /// `recovered_func_id,0,START,0,`
    FunctionStart(u32),
    /// Source function return event, trace format is:
    ///
    /// `recovered_func_id,0,END,0,`
    FunctionEnd(u32),
    /// fuel cost within a source function's invocation that cannot be attributed to
    /// a specific opcode, trace format is:
    ///
    /// `recovered_func_id,0,00,fuel_consumption,`
    FunctionSetup {
        #[allow(dead_code)]
        recovered_func_id: u32,
        fuel_consumption: u32,
    },
    /// fuel cost during the wasm module's execution that cannot be attributed to a
    /// specific source function.
    SystemSetup(u32),
}

/// Represents all the wasm function invocations within a single wasm function execution.
///
/// This is used to track the wasm native calls during execution of a single quickjs opcode.
#[derive(Debug, Clone)]
pub struct WasmCallFrame {
    pub wasm_func_id: u32,
    /// fuel when wasm func starts - fuel when opcode execution starts
    pub start_fuel_watermark: u32,
    /// possible invocations to other wasm functions
    pub calls: Vec<WasmCallFrame>,
    /// fuel when wasm func returns - fuel when opcode execution starts
    pub end_fuel_watermark: u32,
}

impl Default for WasmCallFrame {
    fn default() -> Self {
        Self {
            wasm_func_id: Default::default(),
            start_fuel_watermark: Default::default(),
            calls: Default::default(),
            end_fuel_watermark: Default::default(),
        }
    }
}

impl FromStr for BytecodeTraceEvent {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use BytecodeTraceEvent::*;
        let parts: Vec<&str> = s.split(",").collect();
        if parts.len() < 4 {
            bail!("Invalid trace event: {s}");
        }
        let recovered_func_id: u32 = parts[0].parse()?;
        let opcode_offset: u32 = parts[1].parse()?;
        let opcode_or_event = parts[2];
        let fuel_consumption: u32 = parts[3].parse()?;
        match opcode_or_event {
            "START" => Ok(FunctionStart(recovered_func_id)),
            "END" => Ok(FunctionEnd(recovered_func_id)),
            _ => {
                let opcode_byte = u8::from_str_radix(opcode_or_event, 16)?;
                if opcode_offset == 0 {
                    if recovered_func_id == 0 {
                        Ok(SystemSetup(fuel_consumption))
                    } else {
                        Ok(FunctionSetup {
                            recovered_func_id,
                            fuel_consumption,
                        })
                    }
                } else {
                    let wasm_calls_trace = parts.get(4).unwrap_or(&"");
                    let native_calls = native_calls_from_str(wasm_calls_trace)?;
                    Ok(OpcodeRun {
                        recovered_func_id,
                        opcode_offset,
                        opcode_byte,
                        fuel_consumption,
                        native_calls,
                    })
                }
            }
        }
    }
}

/// Parse the native calls from trace string.
fn native_calls_from_str(trace: &str) -> Result<Vec<WasmCallFrame>> {
    if trace.is_empty() {
        return Ok(vec![]);
    }
    let mut wasm_call_list = WasmCallFrame::default();
    let events: Vec<&str> = trace.split("|").collect();
    build_call_frame(&mut wasm_call_list, &events, 0)?;
    Ok(wasm_call_list.calls)
}

/// helper function to build the call frame from the trace string.
fn build_call_frame(
    call_frame: &mut WasmCallFrame,
    trace_list: &Vec<&str>,
    cur_pos: usize,
) -> Result<usize> {
    let mut i = cur_pos;
    let list_len = trace_list.len();
    while i < list_len {
        let parts: Vec<&str> = trace_list[i].split(":").collect();
        let wasm_func_id: u32 = parts[0].parse().unwrap();
        let is_start = parts[1] == "S";
        let fuel_watermark: u32 = parts[2].parse().unwrap();
        if wasm_func_id == call_frame.wasm_func_id && !is_start {
            call_frame.end_fuel_watermark = fuel_watermark;
            return Ok(i + 1);
        } else if is_start {
            let mut new_frame = WasmCallFrame::default();
            new_frame.wasm_func_id = wasm_func_id;
            new_frame.start_fuel_watermark = fuel_watermark;
            i = build_call_frame(&mut new_frame, trace_list, i + 1)?;
            if new_frame.end_fuel_watermark == 0 {
                new_frame.end_fuel_watermark = fuel_watermark;
            }
            call_frame.calls.push(new_frame);
        } else {
            return Err(Error::msg("Improperly formatted trace"));
        }
    }
    Ok(i)
}
