use std::collections::HashMap;

use anyhow::Result;
use jac_translate::{
    quickpars::{ClosureVarIndex, FuncIndex, LocalIndex},
    Translation, TranslationBuilder,
};
use quickpars::Opcode;
use trace::BytecodeTraceEvent;
use utils::{generate_trace, match_all_functions, recover_bytecodes};
mod trace;
mod utils;

/// Produces a human readable report from QuickJS bytecode and a raw execution
/// trace.
pub fn trace(bytecode: &[u8], raw_trace: &str) -> Result<Vec<String>> {
    let builder = TranslationBuilder::new();
    let translation = builder.translate(bytecode)?;
    let trace_parser = ProfileTraceParser::new(raw_trace, &translation)?;
    Ok(trace_parser.report_trace().unwrap_or_default())
}

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
    /// In-memory representation of QuickJS bytecode.
    translation: &'a Translation<'a>,
    /// Maps recovered functions to the user defined functions in the JS module.
    matched_functions: HashMap<u32, MatchedFuncInfo>,
    /// Maps recovered intrinsic function ids to their default names (intrinsic_fn_#id).
    intrinsic_fn_names: HashMap<u32, String>,
    /// Per function operator metadata.
    operator_metadata: HashMap<u32, Vec<(u32, Opcode)>>,
}

impl<'a> ProfileTraceParser<'a> {
    pub fn new(raw_execution_trace: &str, translation: &'a Translation) -> Result<Self> {
        let mut operator_metadata = HashMap::new();
        for func in &translation.module.functions {
            let mut operators = vec![];
            let mut reader = func.operators.clone();
            while !reader.done() {
                if let Ok(op) = Opcode::from_reader(&mut reader) {
                    operators.push(op);
                }
            }
            operator_metadata.insert(func.index.as_u32(), operators);
        }

        let trace = generate_trace(raw_execution_trace);
        let recovered_opcodes = recover_bytecodes(&trace);
        let matched_functions = match_all_functions(&operator_metadata, &recovered_opcodes);
        let intrinsic_fn_names = recovered_opcodes
            .keys()
            .filter(|k| !matched_functions.contains_key(k))
            .enumerate()
            .map(|(idx, k)| (*k, format!("intrinsic_fn_{}", idx)))
            .collect::<HashMap<_, _>>();

        Ok(Self {
            trace,
            translation,
            matched_functions,
            intrinsic_fn_names,
            operator_metadata,
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
                            .translation
                            .module
                            .functions
                            .get(*js_func_idx as usize)
                            .and_then(|f| {
                                self.operator_metadata
                                    .get(&f.index.as_u32())
                                    .map(|f| f.get(*opcode_idx as usize))
                                    .unwrap()
                            })?;
                        let opcode_str = report(
                            *offset,
                            FuncIndex::from_u32(*js_func_idx),
                            &self.translation,
                            opcode,
                        );
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
                        .map(|x| {
                            Some(
                                self.translation
                                    .resolve_func_name(FuncIndex::from_u32(x.0), None)
                                    .to_string(),
                            )
                        })
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
                        .map(|x| {
                            Some(
                                self.translation
                                    .resolve_func_name(FuncIndex::from_u32(x.0), None)
                                    .to_string(),
                            )
                        })
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

pub fn report(pc: u32, func_index: FuncIndex, translation: &Translation, op: &Opcode) -> String {
    use Opcode::*;
    dbg!(&op);
    format!(
        "{}: {}",
        pc,
        match *op {
            FClosure { index } => translation
                .resolve_func_name(func_index, Some(index))
                .to_string(),
            PushAtomValue { atom } => translation.resolve_atom_name(atom).to_string(),
            PrivateSymbol { atom } => translation.resolve_atom_name(atom).to_string(),
            ThrowError { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            CheckVar { atom } => translation.resolve_atom_name(atom).to_string(),
            GetVarUndef { atom } => translation.resolve_atom_name(atom).to_string(),
            GetVar { atom } => translation.resolve_atom_name(atom).to_string(),
            PutVar { atom } => translation.resolve_atom_name(atom).to_string(),
            PutVarInit { atom } => translation.resolve_atom_name(atom).to_string(),
            PutVarStrict { atom } => translation.resolve_atom_name(atom).to_string(),
            DefineVar { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            CheckDefineVar { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            DefineFunc { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            GetField { atom } => translation.resolve_atom_name(atom).to_string(),
            GetField2 { atom } => translation.resolve_atom_name(atom).to_string(),
            PutField { atom } => translation.resolve_atom_name(atom).to_string(),
            DefineField { atom } => translation.resolve_atom_name(atom).to_string(),
            SetName { atom } => translation.resolve_atom_name(atom).to_string(),
            DefineMethod { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            DefineMethodComputed { .. } => "DefineComputedMethod".to_string(),
            DefineClass { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            DefineClassComputed { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            GetLoc { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            PutLoc { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            SetLoc { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            GetArg { index } => translation
                .resolve_func_arg_name(func_index, index)
                .to_string(),
            PutArg { index } => translation
                .resolve_func_arg_name(func_index, index)
                .to_string(),
            SetArg { index } => translation
                .resolve_func_arg_name(func_index, index)
                .to_string(),
            GetVarRef { index } => translation
                .resolve_closure_var_name(func_index, index)
                .to_string(),
            PutVarRef { index } => translation
                .resolve_closure_var_name(func_index, index)
                .to_string(),
            SetVarRef { index } => translation
                .resolve_closure_var_name(func_index, index)
                .to_string(),
            SetLocUninit { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            GetLocCheck { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            PutLocCheck { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            PutLocCheckInit { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            GetLocCheckThis { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            GetVarRefCheck { index } => translation
                .resolve_closure_var_name(func_index, index)
                .to_string(),
            PutVarRefCheck { index } => translation
                .resolve_closure_var_name(func_index, index)
                .to_string(),
            PutVarRefCheckInit { index } => translation
                .resolve_closure_var_name(func_index, index)
                .to_string(),
            WithGetVar { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            WithPutVar { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            WithDeleteVar { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            WithMakeRef { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            WithGetRef { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            WithGetRefUndef { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            MakeLocRef { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            MakeArgRef { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            MakeVarRefRef { atom, .. } => translation.resolve_atom_name(atom).to_string(),
            MakeVarRef { atom } => translation.resolve_atom_name(atom).to_string(),
            DecLoc { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            IncLoc { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            AddLoc { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            FClosure8 { index } => translation
                .resolve_func_name(func_index, Some(index))
                .to_string(),
            GetLoc8 { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            PutLoc8 { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            SetLoc8 { index } => translation
                .resolve_func_local_name(func_index, index)
                .to_string(),
            GetLoc0 | GetLoc1 | GetLoc2 | GetLoc3 => {
                let index = op.discriminant() - 199 as u8;
                translation
                    .resolve_func_local_name(func_index, LocalIndex::from_u32(index as _))
                    .to_string()
            }
            PutLoc0 | PutLoc1 | PutLoc2 | PutLoc3 => {
                let index = op.discriminant() - 203 as u8;
                translation
                    .resolve_func_local_name(func_index, LocalIndex::from_u32(index as _))
                    .to_string()
            }
            SetLoc0 | SetLoc1 | SetLoc2 | SetLoc3 => {
                let index = op.discriminant() - 207 as u8;
                translation
                    .resolve_func_local_name(func_index, LocalIndex::from_u32(index as _))
                    .to_string()
            }
            GetArg0 | GetArg1 | GetArg2 | GetArg3 => {
                let index = op.discriminant() - 211 as u8;
                translation
                    .resolve_func_arg_name(func_index, LocalIndex::from_u32(index as _))
                    .to_string()
            }
            PutArg0 | PutArg1 | PutArg2 | PutArg3 => {
                let index = op.discriminant() - 215 as u8;
                translation
                    .resolve_func_arg_name(func_index, LocalIndex::from_u32(index as _))
                    .to_string()
            }
            SetArg0 | SetArg1 | SetArg2 | SetArg3 => {
                let index = op.discriminant() - 219 as u8;
                translation
                    .resolve_func_arg_name(func_index, LocalIndex::from_u32(index as _))
                    .to_string()
            }
            GetVarRef0 | GetVarRef1 | GetVarRef2 | GetVarRef3 => {
                let index = op.discriminant() - 223 as u8;
                translation
                    .resolve_closure_var_name(func_index, ClosureVarIndex::from_u32(index as _))
                    .to_string()
            }
            PutVarRef0 | PutVarRef1 | PutVarRef2 | PutVarRef3 => {
                let index = op.discriminant() - 227 as u8;
                translation
                    .resolve_closure_var_name(func_index, ClosureVarIndex::from_u32(index as _))
                    .to_string()
            }
            SetVarRef0 | SetVarRef1 | SetVarRef2 | SetVarRef3 => {
                let index = op.discriminant() - 231 as u8;
                translation
                    .resolve_closure_var_name(func_index, ClosureVarIndex::from_u32(index as _))
                    .to_string()
            }
            op => format!("{:?}", op),
        }
    )
}
