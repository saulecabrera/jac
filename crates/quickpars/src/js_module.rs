use core::fmt;
use std::fmt::Write;

use crate::sections::{FunctionSection, HeaderSection, ModuleSection};

/// The entire parsed js module from bytecode.
#[derive(Debug, Clone)]
pub struct JsModule<'a> {
    pub header: HeaderSection,
    pub module: ModuleSection,
    pub functions: Vec<FunctionSection<'a>>,
}

impl<'a> JsModule<'a> {
    /// Creates a new [JsModule].
    pub fn new(
        header: HeaderSection,
        module: ModuleSection,
        functions: Vec<FunctionSection<'a>>,
    ) -> Self {
        Self {
            header,
            module,
            functions,
        }
    }

    pub fn get_module_name(&self) -> Option<String> {
        self.get_atom_name(self.module.name_index)
    }

    pub fn get_fn_name(&self, fn_idx: u32) -> Option<String> {
        self.functions.get(fn_idx as usize).and_then(|f| {
            let name_idx = f.header().name_index;
            if name_idx == 0 {
                Some(format!("lambda_fn_{}", fn_idx))
            } else {
                self.get_atom_name(name_idx)
            }
        })
    }

    // func local var name
    pub fn get_fn_loc_name(&self, fn_idx: u32, loc_index: u16) -> Option<String> {
        self.functions
            .get(fn_idx as usize)
            .and_then(|f| f.get_local(loc_index + f.header().arg_count as u16))
            .map(|local| self.get_atom_name(local.name_index))
            .unwrap_or(None)
    }

    pub fn get_fn_arg_name(&self, fn_idx: u32, arg_index: u16) -> Option<String> {
        self.functions
            .get(fn_idx as usize)
            .and_then(|f| f.get_local(arg_index))
            .map(|local| self.get_atom_name(local.name_index))
            .unwrap_or(None)
    }

    pub fn get_fn_closure_name(&self, fn_idx: u32, closure_index: u16) -> Option<String> {
        self.functions
            .get(fn_idx as usize)
            .and_then(|f| f.get_closure(closure_index))
            .map(|closure| self.get_atom_name(closure.name_index))
            .unwrap_or(None)
    }

    // returns the string representing the atom name at the given index.
    pub fn get_atom_name(&self, atom_idx: u32) -> Option<String> {
        self.header.atoms.get(atom_idx as usize).map(String::clone)
    }

    // returns the canonicalized offset + operator for a given fn, at the given operator_idx.
    pub fn report_operator(&self, fn_idx: u32, operator_idx: u32) -> Option<String> {
        self.functions
            .get(fn_idx as usize)
            .and_then(|f| f.operators().get(operator_idx as usize))
            .map(|(offset, op)| op.report(*offset, fn_idx, self))
    }

    pub fn fmt_report(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!(
            "module_name: {:}\n",
            self.get_atom_name(self.module.name_index)
                .unwrap_or("".to_string())
        ))?;
        for (i, func) in self.functions.iter().enumerate() {
            func.fmt_report(self, i as u32, f)?;
            f.write_char('\n')?;
        }
        Ok(())
    }
}
