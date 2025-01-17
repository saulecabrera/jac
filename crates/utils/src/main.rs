use anyhow::Result;
use clap::{Parser, Subcommand};
use javy::{Config, Runtime};
use parsetrace::trace;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod printer;

#[derive(Debug, Parser)]
#[command(
    name = "jac-utils",
    version,
    about = "Utils for the development of JAC"
)]
struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(arg_required_else_help = true)]
    Trace(TraceOptions),
    #[command(arg_required_else_help = true)]
    Print(PrintOptions),
}

#[derive(Debug, Parser)]
pub struct TraceOptions {
    /// Path to the JavaScript input file.
    #[arg(value_name = "JS", required = true)]
    pub input: PathBuf,

    /// Parse to the generated trace file.
    ///
    /// Traces are generated with the Wizard Research WebAssembly engine.
    /// github.com/titzer/wizard-engine
    #[arg(long, required = true)]
    pub trace: PathBuf,

    /// The directory and file where to place the report.
    /// Defaults to `trace_out.txt`
    #[arg(short = 'o', required = false, default_value = "trace_out.txt")]
    pub out: PathBuf,
}

#[derive(Debug, Parser)]
pub struct PrintOptions {
    /// Path to the JavaScript input file.
    #[arg(value_name = "JS", required = true)]
    pub input: PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match &args.command {
        Command::Trace(opts) => {
            let bytecode = compile(&opts.input)?;
            let raw_trace = std::fs::read_to_string(&opts.trace)?;
            let report = trace(&bytecode, &raw_trace)?;
            let mut file = File::create(&opts.out)?;
            for line in report {
                file.write_all(line.as_bytes())?;
            }
        }
        Command::Print(opts) => {
            let bytecode = compile(&opts.input)?;
            printer::print(&bytecode)?;
        }
    }

    Ok(())
}

/// Compile JS source to bytecode.
fn compile(js: &PathBuf) -> Result<Vec<u8>> {
    let source = std::fs::read_to_string(js)?;
    let config = Config::default();
    let runtime = Runtime::new(config)?;
    let name = js
        .file_name()
        .map(|s| s.to_str())
        .flatten()
        .unwrap_or_else(|| "index.js");
    runtime.compile_to_bytecode(&name, &source)
}
