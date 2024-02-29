//! Bytecode sections.

use crate::readers::BinaryReader;

/// The start section of the bytecode.
#[derive(Debug, Copy, Clone)]
pub struct HeaderSection<'a> {
    /// The number of interned atoms in the bytecode.
    pub atom_count: u32,
    /// The binary reader
    reader: BinaryReader<'a>,
}

// TODO
// Add a way to read the atoms.
impl<'a> HeaderSection<'a> {
    /// Creates a new [HeaderSection].
    pub(crate) fn new(atom_count: u32, reader: BinaryReader<'a>) -> Self {
        Self { atom_count, reader }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct ModuleSection<'a> {
    /// The index of the module name.
    name_index: u32,
    /// The binary reader over the module section.
    reader: BinaryReader<'a>,
}

impl<'a> ModuleSection<'a> {
    /// Creates a new [ModuleSection].
    pub(crate) fn new(name_index: u32, reader: BinaryReader<'a>) -> Self {
        Self { name_index, reader }
    }
}

/// Function section metadata.
#[derive(Debug, Default, Copy, Clone)]
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
    /// The closure count.
    pub closure_count: u32,
    /// The number of elements in the constant pool.
    pub constant_pool_size: u32,
    /// The function bytecode length.
    pub bytecode_len: u32,
    /// The number of locals.
    pub local_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct DebugInfo<'a> {
    filename: u32,
    lineno: u32,
    reader: BinaryReader<'a>,
}

impl<'a> DebugInfo<'a> {
    /// Create a new [DebugInfo].
    pub fn new(filename: u32, lineno: u32, reader: BinaryReader<'a>) -> Self {
        Self {
            filename,
            lineno,
            reader,
        }
    }
}

/// A function section.
#[derive(Debug, Clone, Copy)]
pub struct FunctionSection<'a> {
    /// The function section header.
    pub header: FunctionSectionHeader,
    /// The locals reader.
    locals_reader: BinaryReader<'a>,
    /// The closures reader.
    closures_reader: BinaryReader<'a>,
    /// The operators reader.
    operators_reader: BinaryReader<'a>,
    /// The function debug information.
    debug: Option<DebugInfo<'a>>,
}

impl<'a> FunctionSection<'a> {
    /// Create a new [FunctionSection].
    pub(crate) fn new(
        header: FunctionSectionHeader,
        locals_reader: BinaryReader<'a>,
        closures_reader: BinaryReader<'a>,
        operators_reader: BinaryReader<'a>,
        debug: Option<DebugInfo<'a>>,
    ) -> Self {
        Self {
            header,
            locals_reader,
            closures_reader,
            operators_reader,
            debug,
        }
    }
}
