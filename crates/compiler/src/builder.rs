//! Function Builder.
use anyhow::{anyhow, bail, Result};
use jac_translate::{quickpars::BinaryReader, FunctionTranslation};
use std::collections::{HashMap, HashSet};
use waffle::{Block, FunctionBody, Local, Signature, Type, Value};

/// The current block.
#[derive(Default)]
pub(crate) struct CurrentBlock {
    /// The current block.
    block: Block,
    /// Local-to-SSA-Value mapping in the current block.
    local_to_value: HashMap<Local, Value>,
}

impl CurrentBlock {
    fn new(block: Block) -> Self {
        Self {
            block,
            local_to_value: Default::default(),
        }
    }

    /// Set the value of a local in the current block.
    fn set_local(&mut self, local: Local, val: Value) -> Result<()> {
        self.local_to_value.insert(local, val);
        Ok(())
    }
}

/// Control stack frame.
pub(crate) enum Frame {
    /// A block frame.
    Block(Block),
}

/// An IR builder.
pub(crate) struct FunctionBuilder<'a, 'data> {
    /// The QuickJS bytecode function translation.
    translation: &'a FunctionTranslation<'data>,
    /// The funciton signature.
    signature: Signature,
    /// The resulting function body.
    result: FunctionBody,
    /// Local type declarations.
    decl: HashMap<Local, Type>,
    /// The current block.
    block: Option<CurrentBlock>,
    /// The sealed blocks.
    sealed: HashSet<Block>,
    /// Undefined locals used to calculate block params.
    undefined: HashMap<Block, Vec<(Local, Value)>>,
    /// Shadow operand stack.
    stack: Vec<(Type, Value)>,
    /// Control frame stack.
    control: Vec<Frame>,
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
            undefined: Default::default(),
            stack: Default::default(),
            control: Default::default(),
        }
    }

    pub fn build(mut self) -> Result<(Signature, FunctionBody)> {
        let mut reader = &mut self.translation.operators.clone();
        self.handle_function_header()?;
        self.handle_entry()?;
        self.handle_operator(&mut reader)?;
        self.validate_blocks()?;
        Ok((self.signature, self.result))
    }

    /// Return values for JS functions.
    /// All functions return a NaN-boxed value, represented as a u64.
    fn rets() -> [Type; 1] {
        [Type::I64]
    }

    // At the bebginning of the body builder:
    // * Add a block, which will be the "out" block
    // * This block will contain the function's return as block params. From the
    // translation information, we might not know all the block params until the
    // end.What we're certain is that JS functions _always_ return a value.
    fn handle_function_header(&mut self) -> Result<()> {
        self.result.rets.push(Self::rets()[0]);

        // `local_count` accounts for: both defined arguments and function
        // locals.
        for _ in 0..self.translation.header.local_count {
            // JS values are NaN-boxed. So we treat all of them as I64s.
            self.result.locals.push(Type::I64);
        }

        let out_block = self.result.add_block();
        self.add_blockparams(out_block, &Self::rets())?;
        self.control.push(Frame::Block(out_block));

        Ok(())
    }

    // The function params *only*, will get added as the block params of the
    // entry block.
    //
    // The params and rest of locals, will get declared in the current block.
    fn handle_entry(&mut self) -> Result<()> {
        let entry = self.result.add_block();
        self.result.entry = entry;
        // TODO: Seal entry.
        self.block = Some(CurrentBlock::new(entry));

        // TODO: What about args that are not explicitly defined?.

        // Iterate over the defined arguments and declare the locals as well as
        // add them as block params.
        for i in 0..self.translation.header.defined_arg_count {
            let local: Local = i.into();
            self.result.add_blockparam(entry, Type::I64);

            let val = self.result.blocks[entry].params.last().cloned().unwrap();
            self.declare_local(local)?;
            self.set_local(local, val.1.clone())?;
        }

        // Declare non-argument locals.
        for i in self.translation.header.defined_arg_count..self.translation.header.local_count {
            let local: Local = i.into();
            self.declare_local(local)?;
        }

        Ok(())
    }

    fn validate_blocks(&self) -> Result<()> {
        for block in self.result.blocks.iter() {
            if !self.is_block_sealed(&block) {
                bail!("Expected block {} to be sealed", block);
            }
        }

        Ok(())
    }

    fn current_block_mut(&mut self) -> Result<&mut CurrentBlock> {
        self.block
            .as_mut()
            .ok_or_else(|| anyhow!("Expected current block. None found"))
    }

    fn declare_local(&mut self, local: Local) -> Result<()> {
        let ty = Type::I64;
        let existed = self.decl.insert(local, ty).is_none();
        if !existed {
            bail!("Local {} {} already declared in current block", local, ty);
        }

        Ok(())
    }

    fn set_local(&mut self, local: Local, val: Value) -> Result<()> {
        self.current_block_mut()?.set_local(local, val)
    }

    fn is_block_sealed(&self, block: &Block) -> bool {
        self.sealed.contains(block)
    }

    fn handle_operator(&mut self, _reader: &mut BinaryReader<'data>) -> Result<()> {
        Ok(())
    }

    fn add_blockparams(&mut self, block: Block, types: &[Type]) -> Result<()> {
        for ty in types {
            self.result.add_blockparam(block, *ty);
        }

        Ok(())
    }
}
