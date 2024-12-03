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
            let mut fcompiler = FunctionCompiler::new(&func);
            let body = fcompiler.compile(&self.translation)?;
        }
        self.module.to_wasm_bytes()
    }
}

/// A single function compiler.
struct FunctionCompiler<'a, 'operators> {
    translation: &'a FunctionTranslation<'operators>,
}

impl<'a, 'operators> FunctionCompiler<'a, 'operators> {
    fn new(translation: &'a FunctionTranslation<'operators>) -> Self {
        Self { translation }
    }

    fn compile(&mut self, translation: &Translation<'operators>) -> Result<FunctionBody> {
        let mut body = FunctionBody::default();
        let mut reader = &mut self.translation.operators.clone();

        while !reader.done() {
            self.compile_op(&mut body, Opcode::from_reader(&mut reader)?.1)?;
        }

        Ok(body)
    }

    fn compile_op(&mut self, body: &mut FunctionBody, op: Opcode) -> Result<()> {
        match op {
            Opcode::FClosure8 { index } => {}
            _ => unreachable!(),
        }

        Ok(())
    }
}
