//! Bytecode sections.

use core::fmt;

use crate::AtomIndex;
use crate::{op::Opcode, readers::BinaryReader};

pub type OpcodeList = Vec<(u32, Opcode)>;

/// The start section of the bytecode.
#[derive(Debug, Clone)]
pub struct HeaderSection {
    /// The number of interned atoms in the bytecode.
    pub atom_count: u32,
    /// The entire list of atom names accessible to the module, including built-in atoms.
    pub atoms: Vec<String>,
}

impl Default for HeaderSection {
    fn default() -> Self {
        Self {
            atom_count: u32::MAX,
            atoms: Default::default(),
        }
    }
}

impl HeaderSection {
    /// Creates a new [HeaderSection].
    pub(crate) fn new(atom_count: u32, atoms: Vec<String>) -> Self {
        Self { atom_count, atoms }
    }
}
#[derive(Clone, Debug)]
pub struct ModuleSectionHeader {
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

impl Default for ModuleSectionHeader {
    fn default() -> Self {
        Self {
            name_index: u32::MAX,
            req_modules: Default::default(),
            exports: Default::default(),
            star_exports: Default::default(),
            imports: Default::default(),
            has_tla: u8::MAX,
        }
    }
}

impl ModuleSectionHeader {
    /// Creates a new [ModuleSectionHeader].
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
    pub name_index: AtomIndex,
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
    pub name_index: AtomIndex,
    pub index: u32,
    pub flags: u8,
}

/// Function local variable information.
#[derive(Debug, Default, Copy, Clone)]
pub struct FunctionLocal {
    pub name_index: AtomIndex,
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

impl fmt::Debug for DebugInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DebugInfo")
            .field("filename", &self.filename)
            .field("lineno", &self.lineno)
            .field("colno", &self.colno)
            .finish()
    }
}
