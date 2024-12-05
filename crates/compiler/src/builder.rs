//! Function Builder.
use anyhow::Result;
use jac_translate::{quickpars::BinaryReader, FunctionTranslation};
use std::collections::{HashMap, HashSet};
use waffle::{Block, FunctionBody, Local, Signature, Type, Value};

/// The current block.
#[derive(Default)]
pub struct CurrentBlock {
    /// The current block.
    block: Block,
    /// Local mapping in the current block.
    locals: HashMap<Local, Type>,
}

/// An IR builder.
pub(crate) struct FunctionBuilder<'a, 'data> {
    /// The QuickJS bytecode function translation.
    translation: &'a FunctionTranslation<'data>,
    /// The funciton signature.
    signature: Signature,
    /// The resulting function body.
    result: FunctionBody,
    /// Local declarations.
    decl: HashMap<Local, Type>,
    /// The current block.
    block: Option<CurrentBlock>,
    /// The sealed blocks.
    sealed: HashSet<Block>,
    /// Placeholder locals used to calculate block params.
    placeholders: HashMap<Block, Vec<(Local, Value)>>,
}

impl<'a, 'data> FunctionBuilder<'a, 'data> {
    pub fn new(translation: &'a FunctionTranslation<'data>) -> Self {
        Self {
            translation,
            signature: Default::default(),
            result: Default::default(),
            decl: Default::default(),
            block: None,
            sealed: Default::default(),
            placeholders: Default::default(),
        }
    }

    pub fn build(mut self) -> Result<(Signature, FunctionBody)> {
        let mut reader = &mut self.translation.operators.clone();
        self.handle_operator(&mut reader)?;
        Ok((self.signature, self.result))
    }

    fn handle_operator(&mut self, reader: &mut BinaryReader<'data>) -> Result<()> {
        Ok(())
    }
}
