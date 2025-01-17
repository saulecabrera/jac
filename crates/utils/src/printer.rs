use anyhow::{anyhow, Result};
use jac_translate::{quickpars::Opcode, FunctionTranslation, Translation, TranslationBuilder};
use std::fmt::Write;

/// Pretty-print QuickJS bytecode.
pub fn print(bytecode: &[u8]) -> Result<()> {
    let builder = TranslationBuilder::new();
    let translation = builder.translate(bytecode)?;
    let printer = Printer::new();
    let result = printer.print(&translation)?;
    println!("{}", result);

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

    /// Prints a new line.
    fn nl(&mut self) -> Result<()> {
        self.writer.write_str("\n").map_err(|e| anyhow!(e))
    }

    /// Prints two spaces.
    fn space2(&mut self) -> Result<()> {
        self.writer.write_str("  ").map_err(|e| anyhow!(e))
    }

    /// Print a function.
    fn print_func(&mut self, translation: &Translation, func: &FunctionTranslation) -> Result<()> {
        let func_name = translation.resolve_atom_name(func.header.name_index);
        write!(&mut self.writer, "func: {}", func_name).map_err(|e| anyhow!("{}", e))?;
        self.nl()?;

        let mut reader = func.operators.clone();

        while !reader.done() {
            let op = Opcode::from_reader(&mut reader)?;
            self.print_op(op.1, &translation, &func)?;
            self.nl()?;
        }

        Ok(())
    }

    /// Print an op code.
    //
    // TODO: Handle offsets.
    fn print_op(
        &mut self,
        op: Opcode,
        translation: &Translation,
        func: &FunctionTranslation,
    ) -> Result<()> {
        use Opcode::*;

        self.space2()?;
        match op {
            Invalid => write!(&mut self.writer, "{}", "Invalid"),
            PushI32 { value } => write!(&mut self.writer, "PushI32 {}", value),
            PushConst { index } => {
                let imm = translation.resolve_func_name(func.index, Some(index));
                write!(&mut self.writer, "PushI32 {}", imm)
            }
            FClosure { index } => {
                let imm = translation.resolve_func_name(func.index, Some(index));
                write!(self.writer, "FClosure {}", imm)
            }
            PushAtomValue { atom } => {
                let v = translation.resolve_atom_name(atom);
                write!(self.writer, "PushAtomValue {}", v)
            }
            PrivateSymbol { atom } => {
                let sym = translation.resolve_atom_name(atom);
                write!(self.writer, "PrivateSymbol {}", sym)
            }
            Undefined => write!(self.writer, "Undefined"),
            Null => write!(self.writer, "Null"),
            PushThis => write!(self.writer, "PushThis"),
            PushFalse => write!(self.writer, "PushFalse"),
            PushTrue => write!(self.writer, "PushTrue"),
            Object => write!(self.writer, "Object"),
            Rest { first } => write!(self.writer, "Rest {}", first),
            Drop => write!(self.writer, "Drop"),
            Nip => write!(self.writer, "Nip"),
            Nip1 => write!(self.writer, "Nip1"),
            Dup => write!(self.writer, "Dup"),
            Dup1 => write!(self.writer, "Dup1"),
            Dup2 => write!(self.writer, "Dup2"),
            Dup3 => write!(self.writer, "Dup3"),
            Insert2 => write!(self.writer, "Insert2"),
            Insert3 => write!(self.writer, "Insert3"),
            Insert4 => write!(self.writer, "Insert4"),
            Perm3 => write!(self.writer, "Perm3"),
            Perm4 => write!(self.writer, "Perm4"),
            Perm5 => write!(self.writer, "Perm5"),
            Swap => write!(self.writer, "Swap"),
            Swap2 => write!(self.writer, "Swap2"),
            Rot3L => write!(self.writer, "Rot3L"),
            Rot3R => write!(self.writer, "Rot3R"),
            Rot4L => write!(self.writer, "Rot4L"),
            Rot5L => write!(self.writer, "Rot5L"),
            CallConstructor { argc } => write!(self.writer, "CallConstructor {}", argc),
            Call { argc } => write!(self.writer, "Call {}", argc),
            TailCall { argc } => write!(self.writer, "TailCall {}", argc),
            CallMethod { argc } => write!(self.writer, "CallMethod {}", argc),
            TailCallMethod { argc } => write!(self.writer, "TailCallMethod {}", argc),
            ArrayFrom { argc } => write!(self.writer, "ArrayFrom {}", argc),
            Apply { magic } => write!(self.writer, "Apply {}", magic),
            Return => write!(self.writer, "Return"),
            ReturnUndef => write!(self.writer, "ReturnUndef"),
            CheckCtorReturn => write!(self.writer, "CheckCtorReturn"),
            CheckCtor => write!(self.writer, "CheckCtor"),
            CheckBrand => write!(self.writer, "CheckBrand"),
            AddBrand => write!(self.writer, "AddBrand"),
            ReturnAsync => write!(self.writer, "ReturnAsync"),
            Throw => write!(self.writer, "Throw"),
            ThrowError { ty, atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "ThrowError {} {}", ty, imm)
            }
            Eval { scope, argc } => write!(self.writer, "Eval {} {}", scope, argc),
            ApplyEval { scope } => write!(self.writer, "ApplyEval {}", scope),
            Regexp => write!(self.writer, "Regexp"),
            GetSuper => write!(self.writer, "GetSuper"),
            Import => write!(self.writer, "Import"),
            CheckVar { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "CheckVar {}", imm)
            }
            GetVarUndef { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "GetVarUndef {}", imm)
            }
            GetVar { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "GetVar {}", imm)
            }
            PutVar { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "PutVar {}", imm)
            }
            PutVarInit { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "PutVarInit {}", imm)
            }
            PutVarStrict { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "PutVarStrict {}", imm)
            }
            GetRefValue => write!(self.writer, "GetRefValue"),
            PutRefValue => write!(self.writer, "PutRefValue"),

            _ => write!(&mut self.writer, "Op"),
        }?;

        Ok(())
    }
}
