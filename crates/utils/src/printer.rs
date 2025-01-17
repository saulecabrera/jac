use anyhow::{anyhow, Result};
use jac_translate::{quickpars::Opcode, FunctionTranslation, Translation, TranslationBuilder};
use std::fmt::Write;

/// Pretty-print QuickJS bytecode.
pub fn print(bytecode: &[u8]) -> Result<()> {
    let builder = TranslationBuilder::new();
    let translation = builder.translate(bytecode)?;
    let printer = Printer::new();
    let _result = printer.print(&translation)?;

    Ok(())
}

/// Writer implementation.
#[derive(Default)]
struct Writer {
    inner: String,
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.inner.push_str(s);
        Ok(())
    }
}

/// Printer abstraction.
struct Printer {
    /// Writer struct to accumulate the result.
    writer: Writer,
}

impl Printer {
    /// Create a new [`Printer`].
    fn new() -> Self {
        Self {
            writer: Writer::default(),
        }
    }

    /// Print.
    fn print<'data>(mut self, translation: &Translation<'data>) -> Result<String> {
        for func in &translation.module.functions {
            self.print_func(translation, func)?;
        }

        Ok(std::mem::take(&mut self.writer.inner))
    }

    /// Print a function.
    fn print_func(&mut self, translation: &Translation, func: &FunctionTranslation) -> Result<()> {
        let func_name = translation.resolve_atom_name(func.header.name_index);
        write!(&mut self.writer, "func: {}", func_name).map_err(|e| anyhow!("{}", e))?;

        let mut reader = func.operators.clone();

        while (!reader.done()) {
            let op = Opcode::from_reader(&mut reader)?;
            self.print_op(op, &translation, &func)?;
        }

        Ok(())
    }

    /// Print an op code.
    fn print_op(&mut self, op: Opcode, translation: &Translation, func: &FunctionTranslation) -> Result<()> {
        use Opcode::*;
        match op {
            Invalid => write!(&mut self.writer, "{}", "Invalid"),
            PushI32 { value } = write!(&mut self.writer, "PushI32 {}", value),
            PushConst { index } => {
                let imm = translation.resolve_func_name(func.index, Some(index));
                write!(&mut self.writer, "PushI32 {}", imm)
            },
            _ => unreachable()
        }
        Ok(())
    }
}
