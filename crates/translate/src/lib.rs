//! Data structures for translation of QuickJS bytecode to its memory
//! representation.

use quickpars::{
    AtomIndex, BinaryReader, ClosureVarIndex, ConstantPoolIndex, DebugInfo, FuncIndex,
    FunctionClosureVar, FunctionLocal, FunctionSectionHeader, HeaderSection, LocalIndex,
    ModuleSectionHeader, Parser, Payload,
};

pub use quickpars;

use anyhow::Result;

#[derive(Default, Clone, Debug)]
pub struct Translation<'data> {
    /// Overall information about the program.
    pub header: HeaderSection,
    /// Module translation.
    // TODO: One module initially, but this should be extended to N modules.
    pub module: ModuleTranslation<'data>,
}

/// When an instruction in a function references a function in the constant
/// pool, we always need to account for 1 more element, given that in our
/// translation layer we assume a flattened structure to represent function
/// indices.
const CONSTANT_POOL_OFFSET: u32 = 1;

impl<'data> Translation<'data> {
    // TODO: Asumes a single module.
    /// Resolves a function name from a given [`FuncIndex`].
    pub fn resolve_func_name(
        &self,
        index: FuncIndex,
        pool_index: Option<ConstantPoolIndex>,
    ) -> &str {
        let index = match pool_index {
            Some(i) => FuncIndex::from_u32(i.as_u32() + index.as_u32() + CONSTANT_POOL_OFFSET),
            None => index,
        };

        let func = &self.module.functions[index.as_u32() as usize];
        let index = func.header.name_index;
        &self.header.atoms[index.as_u32() as usize]
    }

    /// Resolves a closure variable name.
    pub fn resolve_closure_var_name(&self, index: FuncIndex, closure: ClosureVarIndex) -> &str {
        let func = &self.module.functions[index.as_u32() as usize];
        let var = &func.closure_vars[closure.as_u32() as usize];
        &self.header.atoms[var.name_index.as_u32() as usize]
    }

    /// Resolves a function local name from a  [`FuncIndex`] and [`LocalIndex`].
    pub fn resolve_func_local_name(&self, index: FuncIndex, local_index: LocalIndex) -> &str {
        let func = &self.module.functions[index.as_u32() as usize];
        let index = func.resolve_local_name_index(local_index);
        &self.header.atoms[index.as_u32() as usize]
    }

    /// Resolves the name of an argument name.
    ///
    /// Arguments are the first locals in a function.
    pub fn resolve_func_arg_name(&self, index: FuncIndex, local_index: LocalIndex) -> &str {
        let func = &self.module.functions[index.as_u32() as usize];
        let index = func.resolve_arg_name_index(local_index);
        &self.header.atoms[index.as_u32() as usize]
    }

    /// Resolves an atom name from an index.
    pub fn resolve_atom_name(&self, index: AtomIndex) -> &str {
        &self.header.atoms[index.as_u32() as usize]
    }
}

/// A function translation.
///
/// Contains resolved information about a function.
/// The operators are not resolved yet and instead this data structure contains
/// a reader for the operators to be resolved later at compilation time.
#[derive(Debug, Clone)]
pub struct FunctionTranslation<'data> {
    /// The function section header.
    pub header: FunctionSectionHeader,
    /// The function locals.
    pub locals: Vec<FunctionLocal>,
    /// Closure variable references.
    pub closure_vars: Vec<FunctionClosureVar>,
    /// Operators reader.
    pub operators: BinaryReader<'data>,
    /// Debug information.
    pub debug: Option<DebugInfo<'data>>,
    /// The index of this function in the module.
    pub index: FuncIndex,
}

impl<'data> FunctionTranslation<'data> {
    pub fn new(header: FunctionSectionHeader, index: FuncIndex) -> Self {
        Self {
            header,
            locals: Default::default(),
            closure_vars: Default::default(),
            operators: BinaryReader::empty(),
            debug: Default::default(),
            index,
        }
    }

    /// Resolves the atom index of a local.
    /// The returned index is an absolute index of locals for the function.
    fn resolve_local_name_index(&self, local: LocalIndex) -> AtomIndex {
        self.locals[local.as_u32() as usize + (self.header.arg_count as usize)].name_index
    }

    /// Resolves the atom index of function argument.
    fn resolve_arg_name_index(&self, local_index: LocalIndex) -> AtomIndex {
        self.locals[local_index.as_u32() as usize].name_index
    }
}

#[derive(Default, Clone, Debug)]
pub struct ModuleTranslation<'data> {
    /// The module section header.
    pub header: ModuleSectionHeader,
    /// The functions defined in the module.
    ///
    /// When functions are inserted, a [`FuncIndex`] handle is provided to the
    /// caller, which will serve as an indentifier of the function.
    ///
    /// References to function indices found in function bytecode are relative
    /// to the constant pool of each function, however given that in the
    /// translation layer there is not concept of constant pool, indices to
    /// functions are absolute to the functions found in the module.
    ///
    /// Each [`FunctionTranslation`] is associated to a [`FuncIndex`] therefore
    /// is an operator references a function index, such index can be resolved
    /// at the module by doing:
    ///     target function = current function index + operator index
    pub functions: Vec<FunctionTranslation<'data>>,
}

impl<'data> ModuleTranslation<'data> {
    /// Push a function.
    /// This marks the start of a function definition.
    pub fn push_func(&mut self, header: FunctionSectionHeader) -> FuncIndex {
        let index = FuncIndex::from_u32(self.functions.len() as u32);
        let translation = FunctionTranslation::new(header, index);
        self.functions.push(translation);
        index
    }
}
pub struct TranslationBuilder<'data> {
    pub translation: Translation<'data>,
    current_func: FuncIndex,
}

impl<'data> TranslationBuilder<'data> {
    pub fn new() -> Self {
        Self {
            translation: Default::default(),
            current_func: FuncIndex::default(),
        }
    }

    /// Parses, validates and converts QuickJS bytecode to an in-memory
    /// representation of a JavaScript module.
    pub fn translate(&mut self, buffer: &'data [u8]) -> Result<Translation> {
        let mut translation = Translation::default();
        for payload in Parser::new().parse_buffer(buffer) {
            match payload? {
                Payload::Header(h) => translation.header = h,
                Payload::Version(_) => {}
                Payload::ModuleHeader(h) => translation.module.header = h,
                Payload::FunctionHeader(fh) => {
                    self.current_func = self.translation.module.push_func(fh);
                }
                Payload::FunctionLocals(locals) => {
                    self.translation.module.functions[self.current_func.as_u32() as usize].locals =
                        locals;
                }
                Payload::FunctionDebugInfo(di) => {
                    self.translation.module.functions[self.current_func.as_u32() as usize].debug =
                        Some(di);
                }
                Payload::FunctionClosureVars(vars) => {
                    self.translation.module.functions[self.current_func.as_u32() as usize]
                        .closure_vars = vars;
                }
                Payload::FunctionOperators(reader) => {
                    self.translation.module.functions[self.current_func.as_u32() as usize]
                        .operators = reader;
                }
                Payload::End => {}
            }
        }

        Ok(translation)
    }
}
