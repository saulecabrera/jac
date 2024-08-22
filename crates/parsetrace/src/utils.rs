use std::collections::{HashMap, HashSet};

use quickpars::{JsModule, Opcode};

use crate::{trace::BytecodeTraceEvent, MatchedFuncInfo, ProfiledOpcodeList};

/// Turns the raw execution trace into a vector of trace events.
pub(crate) fn generate_trace(raw_execution_trace: &str) -> Vec<BytecodeTraceEvent> {
    let result = raw_execution_trace
        .lines()
        .skip(1) // skip header
        .map(|line| line.parse().unwrap())
        .collect();
    result
}

/// builds a mapping of recovered function ids to their corresponding profiled opcodes.
pub(crate) fn recover_bytecodes(
    trace: &Vec<BytecodeTraceEvent>,
) -> HashMap<u32, ProfiledOpcodeList> {
    let mut result = HashMap::new();
    for event in trace {
        match event {
            BytecodeTraceEvent::OpcodeRun {
                recovered_func_id,
                opcode_offset,
                opcode_byte,
                fuel_consumption: _,
                native_calls: _,
            } => {
                if !result.contains_key(recovered_func_id) {
                    result.insert(*recovered_func_id, HashMap::new());
                }
                result
                    .get_mut(recovered_func_id)
                    .unwrap()
                    .insert(*opcode_offset, *opcode_byte);
            }
            _ => {}
        }
    }
    result
        .into_iter()
        .map(|(k, v)| {
            let mut val = v.into_iter().collect::<Vec<(u32, u8)>>();
            val.sort_by(|a, b| a.0.cmp(&b.0));
            (k, val)
        })
        .collect()
}

/// Match each recovered function bytecode to its corresponding function name,
/// by performing a partial match with the complete function bytecode from the js module.
pub(crate) fn match_all_functions(
    js_module: &JsModule,
    recovered_bytecodes: &HashMap<u32, Vec<(u32, u8)>>,
) -> HashMap<u32, MatchedFuncInfo> {
    let mut result = HashMap::new();
    let mut prev_matched_js_funcs = HashSet::new();
    for (recovered_func_id, recovered_bytes) in recovered_bytecodes {
        let matched_js_func =
            match_single_function(recovered_bytes, js_module, &mut prev_matched_js_funcs);
        if let Some(matched_js_func) = matched_js_func {
            result.insert(*recovered_func_id, matched_js_func);
        }
    }
    result
}

pub(crate) fn match_single_function(
    recovered_bytes: &Vec<(u32, u8)>,
    js_module: &JsModule,
    matched_js_func_idx: &mut HashSet<u32>,
) -> Option<MatchedFuncInfo> {
    for i in 0..js_module.functions.len() {
        if matched_js_func_idx.contains(&(i as u32)) {
            continue;
        }
        let func_bytecode = js_module.functions[i].operators();
        if let Some(matched_opcode_pairs) = match_pair(recovered_bytes, func_bytecode) {
            matched_js_func_idx.insert(i as u32);
            return Some((i as u32, matched_opcode_pairs));
        }
    }
    None
}

pub(crate) fn match_pair(
    recovered: &Vec<(u32, u8)>,
    defn: &[(u32, Opcode)],
) -> Option<HashMap<u32, u32>> {
    let mut i = 0;
    let mut j = 0;
    let mut matched_pairs = HashMap::new();
    while i < recovered.len() && j < defn.len() {
        let (recovered_offset, recovered_byte) = &recovered[i];
        let (defn_offset, defn_op) = &defn[j];
        if recovered_offset == defn_offset {
            if recovered_byte == &defn_op.discriminant() {
                matched_pairs.insert(*recovered_offset, j as u32);
                i += 1;
                j += 1;
            } else {
                return None;
            }
        } else if recovered_offset <= defn_offset {
            i += 1;
        } else {
            j += 1;
        }
    }
    if matched_pairs.len() == recovered.len() {
        Some(matched_pairs)
    } else {
        None
    }
}
