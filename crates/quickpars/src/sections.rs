//! Bytecode sections.

use core::fmt;

use crate::{op::Opcode, readers::BinaryReader, JsModule};

pub type OpcodeList = Vec<(u32, Opcode)>;

/// The start section of the bytecode.
#[derive(Debug, Clone)]
pub struct HeaderSection {
    /// The number of interned atoms in the bytecode.
    pub atom_count: u32,
    /// The entire list of atom names accessible to the module, including built-in atoms.
    pub atoms: Vec<String>,
}

impl HeaderSection {
    /// Creates a new [HeaderSection].
    pub(crate) fn new(atom_count: u32, atoms: Vec<String>) -> Self {
        Self { atom_count, atoms }
    }
}
#[derive(Debug, Clone)]
pub struct ModuleSection {
    /// The index of the module name.
    pub name_index: u32,
    /// The names of required modules, as index into the atom table.
    pub req_modules: Vec<u32>,
    /// The list of exports from this module.
    pub exports: Vec<ModuleExportEntry>,
    /// The names of the star export entries, as index into the atom table.
    pub star_exports: Vec<u32>,
    /// The list of imports from this module.
    pub imports: Vec<ModuleImportEntry>,
    /// Whether the module has top-level await.
    pub has_tla: u8,
}

impl ModuleSection {
    /// Creates a new [ModuleSection].
    pub(crate) fn new(
        name_index: u32,
        req_modules: Vec<u32>,
        exports: Vec<ModuleExportEntry>,
        star_exports: Vec<u32>,
        imports: Vec<ModuleImportEntry>,
        has_tla: u8,
    ) -> Self {
        Self {
            name_index,
            req_modules,
            exports,
            star_exports,
            imports,
            has_tla,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModuleExportEntry {
    Local {
        var_idx: u32,
        export_name_idx: u32,
    },
    Indirect {
        module_idx: u32,
        local_name_idx: u32,
        export_name_idx: u32,
    },
}

#[derive(Debug, Clone)]
pub struct ModuleImportEntry {
    pub var_idx: u32,
    pub name_idx: u32,
    pub req_module_idx: u32,
}

/// Function section metadata.
#[derive(Debug, Default, Clone)]
pub struct FunctionSectionHeader {
    /// Function flags.
    pub flags: u16,
    /// The index of the function name.
    pub name_index: u32,
    /// The argument count.
    pub arg_count: u32,
    /// The variable count.
    pub var_count: u32,
    /// The defined argument count.
    pub defined_arg_count: u32,
    /// The stack size.
    pub stack_size: u32,
    /// List of variables in the closure.
    pub closure_var_count: u32,
    /// The number of elements in the constant pool.
    pub constant_pool_size: u32,
    /// The function bytecode length.
    pub bytecode_len: u32,
    /// The number of locals.
    pub local_count: u32,
}

/// Closure variable information.
#[derive(Debug, Default, Clone)]
pub struct FunctionClosureVar {
    pub name_index: u32,
    pub index: u32,
    pub flags: u8,
}

/// Function local variable information.
#[derive(Debug, Default, Copy, Clone)]
pub struct FunctionLocal {
    pub name_index: u32,
    pub scope_level: u32,
    pub scope_next: u32,
    pub flags: u8,
}

#[derive(Clone, Copy)]
pub struct DebugInfo<'a> {
    pub filename: u32,
    pub lineno: u32,
    pub colno: u32,
    pub line_debug_reader: BinaryReader<'a>,
    pub col_debug_reader: BinaryReader<'a>,
}

impl<'a> DebugInfo<'a> {
    /// Create a new [DebugInfo].
    pub fn new(
        filename: u32,
        lineno: u32,
        colno: u32,
        line_debug_reader: BinaryReader<'a>,
        col_debug_reader: BinaryReader<'a>,
    ) -> Self {
        Self {
            filename,
            lineno,
            colno,
            line_debug_reader,
            col_debug_reader,
        }
    }
}

/// A function section.
#[derive(Clone)]
pub struct FunctionSection<'a> {
    /// The function section header.
    pub header: FunctionSectionHeader,
    /// The parsed local variables.
    pub locals: Vec<FunctionLocal>,
    /// The locals reader.
    pub locals_reader: BinaryReader<'a>,
    /// The parsed closure vars.
    /// Variables referenced by the function.
    pub closure_vars: Vec<FunctionClosureVar>,
    /// The closures reader.
    pub closure_vars_reader: BinaryReader<'a>,
    /// The parsed opcodes, with their offsets.
    pub operators: OpcodeList,
    /// The operators reader.
    pub operators_reader: BinaryReader<'a>,
    /// The function debug information.
    pub debug: Option<DebugInfo<'a>>,
}

impl<'a> FunctionSection<'a> {
    /// Create a new [FunctionSection].
    pub(crate) fn new(
        header: FunctionSectionHeader,
        locals: Vec<FunctionLocal>,
        locals_reader: BinaryReader<'a>,
        closure_vars: Vec<FunctionClosureVar>,
        closure_vars_reader: BinaryReader<'a>,
        mut operators_reader: BinaryReader<'a>,
        debug: Option<DebugInfo<'a>>,
    ) -> Self {
        let mut operators = vec![];
        while !operators_reader.done() {
            if let Ok(op) = Opcode::from_reader(&mut operators_reader) {
                operators.push(op);
            }
        }
        // reset the reader offset
        operators_reader.offset = 0;
        Self {
            header,
            locals,
            locals_reader,
            closure_vars,
            closure_vars_reader,
            operators,
            operators_reader,
            debug,
        }
    }

    /// Returns the function section header.
    pub fn header(&self) -> &FunctionSectionHeader {
        &self.header
    }

    pub fn operators(&self) -> &[(u32, Opcode)] {
        &self.operators
    }

    pub fn get_local(&self, idx: u16) -> Option<&FunctionLocal> {
        self.locals.get(idx as usize)
    }

    pub fn get_closure_var(&self, idx: u16) -> Option<&FunctionClosureVar> {
        self.closure_vars.get(idx as usize)
    }

    pub fn fmt_report(
        &self,
        js_module: &JsModule,
        fn_idx: u32,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        f.debug_struct("FunctionSection")
            .field(
                "name",
                &js_module.get_fn_name(fn_idx).unwrap_or("".to_string()),
            )
            .field(
                "args",
                &self
                    .locals
                    .iter()
                    .take(self.header.arg_count as usize)
                    .map(|l| {
                        js_module
                            .get_atom_name(l.name_index)
                            .unwrap_or("".to_string())
                    })
                    .collect::<Vec<_>>(),
            )
            .field(
                "locals",
                &self
                    .locals
                    .iter()
                    .skip(self.header.arg_count as usize)
                    .map(|l| {
                        js_module
                            .get_atom_name(l.name_index)
                            .unwrap_or("".to_string())
                    })
                    .collect::<Vec<_>>(),
            )
            .field(
                "closures_vars",
                &self
                    .closure_vars
                    .iter()
                    .map(|c| {
                        js_module
                            .get_atom_name(c.name_index)
                            .unwrap_or("".to_string())
                    })
                    .collect::<Vec<_>>(),
            )
            .field(
                "operators",
                &self
                    .operators
                    .iter()
                    .map(|(pc, op)| op.report(*pc, fn_idx, &js_module))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl fmt::Debug for FunctionSection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionSection")
            .field("header", &self.header)
            .field("locals", &self.locals)
            .field("closures_vars", &self.closure_vars)
            .field("operators", &self.operators)
            .field("debug", &self.debug)
            .finish()
    }
}

impl fmt::Debug for DebugInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DebugInfo")
            .field("filename", &self.filename)
            .field("lineno", &self.lineno)
            .field("colno", &self.colno)
            .finish()
    }
}
