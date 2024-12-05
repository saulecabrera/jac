use crate::builder::FunctionBuilder;
use anyhow::Result;
use jac_translate::{quickpars::Opcode, FunctionTranslation, Translation};
use waffle::{Block, FunctionBody, Module};

/// QuickJS-bytecode-to-Wasm compiler.
pub(crate) struct Compiler<'data> {
    /// QuickJS bytecode in memory representation.
    translation: Translation<'data>,
    /// The resulting Wasm module.
    module: Module<'data>,
}

impl<'data> Compiler<'data> {
    /// Create a new compiler from the translated QuickJS bytecode.
    pub fn new(translation: Translation<'data>) -> Self {
        Self {
            translation,
            module: Module::empty(),
        }
    }

    /// Perform compilation into Wasm bytes.
    pub fn compile(&mut self) -> Result<Vec<u8>> {
        for func in &self.translation.module.functions {
            let mut fbuilder = FunctionBuilder::new(&func);
            let (signature, body) = fbuilder.build()?;
        }
        self.module.to_wasm_bytes()
    }
}
