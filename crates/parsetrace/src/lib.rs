use core::fmt;
use std::{collections::HashMap, fmt::Formatter};

use anyhow::Result;
use quickpars::{JsModule, Opcode, Parser};
use trace::BytecodeTraceEvent;
use utils::{generate_trace, match_all_functions, recover_bytecodes};
mod trace;
mod utils;

/// Represents all the profiled opcode bytes for a single function, ordered by their offset.
/// Each element is a tuple of (opcode_offset, opcode_byte).
type ProfiledOpcodeList = Vec<(u32, u8)>;

/// Represents the matched source function for a recovered function from profiled trace.
/// value contains (idx of matched function in js module, opcode_offset to [OpcodeList] idx pairs)
type MatchedFuncInfo = (u32, HashMap<u32, u32>);

/// Parser for the wasm-based quickjs bytecode profile trace.
///
/// This parser takes the raw execution trace and the parsed JS bytecode module, then
/// matches profiled source events with meaningful JS source level names, thus producing
/// a more human-readable profiling report.
#[derive(Clone)]
pub struct ProfileTraceParser<'a> {
    /// Fully parsed execution trace events.
    trace: Vec<BytecodeTraceEvent>,
    /// Parsed quickjs bytecode module.
    js_module: JsModule<'a>,
    /// Maps recovered functions to the user defined functions in the JS module.
    matched_functions: HashMap<u32, MatchedFuncInfo>,
    /// Maps recovered intrinsic function ids to their default names (intrinsic_fn_#id).
    intrinsic_fn_names: HashMap<u32, String>,
}

impl<'a> ProfileTraceParser<'a> {
    pub fn new(raw_execution_trace: &str, js_bytecode: &'a [u8]) -> Result<Self> {
        let trace = generate_trace(raw_execution_trace);
        let recovered_opcodes = recover_bytecodes(&trace);
        let js_module = Parser::parse_buffer_sync(js_bytecode)?;
        let matched_functions = match_all_functions(&js_module, &recovered_opcodes);
        let intrinsic_fn_names = recovered_opcodes
            .keys()
            .filter(|k| !matched_functions.contains_key(k))
            .enumerate()
            .map(|(idx, k)| (*k, format!("intrinsic_fn_{}", idx)))
            .collect::<HashMap<_, _>>();
        Ok(Self {
            trace,
            js_module,
            matched_functions,
            intrinsic_fn_names,
        })
    }

    pub fn report_trace(&self) -> Option<Vec<String>> {
        let mut call_depth = 0;
        let mut output = vec![];
        for event in &self.trace {
            let entry_report = match event {
                BytecodeTraceEvent::OpcodeRun {
                    recovered_func_id,
                    opcode_offset,
                    fuel_consumption,
                    opcode_byte,
                    ..
                } => {
                    if let Some((js_func_idx, opcode_idx_map)) =
                        self.matched_functions.get(recovered_func_id)
                    {
                        let opcode_idx = opcode_idx_map.get(opcode_offset)?;
                        let (offset, opcode) = self
                            .js_module
                            .functions
                            .get(*js_func_idx as usize)
                            .and_then(|f| f.operators().get(*opcode_idx as usize))?;
                        let opcode_str = opcode.report(*offset, *js_func_idx, &self.js_module);
                        Some(format!(
                            "{:indent$}{} fuel_cost: {}",
                            "",
                            opcode_str,
                            fuel_consumption,
                            indent = call_depth * 2
                        ))
                    } else {
                        Some(format!(
                            "{:indent$}{}: {} fuel_cost: {}",
                            "",
                            opcode_offset,
                            Opcode::name_from_byte(*opcode_byte),
                            fuel_consumption,
                            indent = call_depth * 2
                        ))
                    }
                }
                BytecodeTraceEvent::FunctionStart(recovered_func_id) => {
                    let js_func_name = self
                        .matched_functions
                        .get(recovered_func_id)
                        .map(|x| self.js_module.get_fn_name(x.0))
                        .unwrap_or(self.intrinsic_fn_names.get(recovered_func_id).cloned())?;
                    call_depth += 1;
                    Some(format!(
                        "{:indent$}FUNCTION START {}:",
                        "",
                        js_func_name,
                        indent = (call_depth - 1) * 2
                    ))
                }
                BytecodeTraceEvent::FunctionEnd(recovered_func_id) => {
                    let js_func_name = self
                        .matched_functions
                        .get(recovered_func_id)
                        .map(|x| self.js_module.get_fn_name(x.0))
                        .unwrap_or(self.intrinsic_fn_names.get(recovered_func_id).cloned())?;
                    call_depth -= 1;
                    Some(format!(
                        "{:indent$}FUNCTION END {}",
                        "",
                        js_func_name,
                        indent = call_depth * 2
                    ))
                }
                BytecodeTraceEvent::FunctionSetup {
                    fuel_consumption, ..
                } => Some(format!(
                    "{:indent$}SYSTEM COST: {}",
                    "",
                    fuel_consumption,
                    indent = call_depth * 2
                )),
                BytecodeTraceEvent::SystemSetup(fuel_consumption) => Some(format!(
                    "{:indent$}SYSTEM COST: {}",
                    "",
                    fuel_consumption,
                    indent = call_depth * 2
                )),
            };
            if let Some(entry) = entry_report {
                output.push(entry);
            }
        }
        Some(output)
    }
}

impl fmt::Debug for ProfileTraceParser<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.js_module.fmt_report(f)
    }
}
