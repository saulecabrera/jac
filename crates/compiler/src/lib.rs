//! JAC - The Javy Ahead-of-Time Compiler.
use anyhow::Result;
use quickpars::Parser;

pub fn compile(bytes: &[u8]) -> Result<Vec<u8>> {
    let module = Parser::parse_buffer_sync(bytes)?;
    println!("{:#?}", &module);

    Ok(vec![])
}
