#![allow(dead_code)]
#![allow(unused)]

//! JAC - The Javy Ahead-of-Time Compiler.

use anyhow::Result;
use jac_translate::TranslationBuilder;
mod builder;
mod compiler;

pub fn compile(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut builder = TranslationBuilder::new();
    let _translation = builder.translate(bytes)?;

    Ok(vec![])
}
