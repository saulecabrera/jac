#![allow(warnings)]
use std::env;
use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::Result;
use javy::Config;
use javy::Runtime;
use parsetrace::ProfileTraceParser;
use quickpars::Parser;

/// Entry point to parse a js execution trace running in wasm, and generate a human readable report.
///
/// # Arguments
///   0. `js_file_name` - The name of the JS source file.
///   1. `trace_file_name` - The name of the trace file (currently generated from the wizard engine profile-bytecode monitor).
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let current_dir = env::current_dir()?;
    let js_file_name = args.get(1).expect("Please provide a JS file");
    let trace_file_name = args.get(2).expect("Please provide a trace file");
    let default_out_name = "trace_out.txt".to_string();
    let out_file_name = args.get(3).unwrap_or(&default_out_name);

    let js_file_dir = current_dir.join(js_file_name);
    let trace_file_dir = current_dir.join(trace_file_name);

    let js_str = from_file(&js_file_dir)?;
    let trace_str = from_file(&trace_file_dir)?;

    let mut config = Config::default();
    let runtime = Runtime::new(config).unwrap();
    let results = runtime.compile_to_bytecode(&js_file_name, &js_str);
    let binding = results.unwrap();
    let trace_parser = ProfileTraceParser::new(&trace_str, binding.as_slice())?;
    let mut trace_out_file = File::create(current_dir.join(out_file_name))?;
    trace_parser.report_trace().map(|trace| {
        for line in trace {
            writeln!(trace_out_file, "{}", line);
        }
    });
    Ok(())
}

fn from_file(path: &PathBuf) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = vec![];
    file.read_to_end(&mut contents)?;
    Ok(String::from_utf8(contents)?)
}
