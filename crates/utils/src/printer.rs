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
            write!(self.writer, "{:#01x}", op.0)?;
            self.space2()?;
            self.print_op(op.1, &translation, &func)?;
            self.nl()?;
        }
        self.nl()?;

        Ok(())
    }

    /// Print an op code.
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
            DefineVar { flags, atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "DefineVar {} {}", imm, flags)
            }
            CheckDefineVar { flags, atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "CheckDefineVar {} {}", imm, flags)
            }
            DefineFunc { flags, atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "DefineFunc {} {}", imm, flags)
            }
            GetField { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "GetField {}", imm)
            }
            GetField2 { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "GetField2 {}", imm)
            }
            PutField { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "PutField {}", imm)
            }
            GetPrivateField => write!(self.writer, "GetPrivateField"),
            PutPrivateField => write!(self.writer, "PutPrivateField"),
            DefinePrivateField => write!(self.writer, "DefinePrivateField"),
            GetArrayEl => write!(self.writer, "GetArrayEl"),
            GetArrayEl2 => write!(self.writer, "GetArrayEl2"),
            PutArrayEl => write!(self.writer, "PutArrayEl"),
            GetSuperValue => write!(self.writer, "GetSuperValue"),
            PutSuperValue => write!(self.writer, "PutSuperValue"),
            DefineField { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "DefineField {}", imm)
            }
            SetName { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "SetName {}", imm)
            }
            SetNameComputed => write!(self.writer, "SetNameComputed"),
            SetProto => write!(self.writer, "SetProto"),
            SetHomeObject => write!(self.writer, "SetHomeObject"),
            DefineArrayEl => write!(self.writer, "DefineArrayEl"),
            Append => write!(self.writer, "Append"),
            CopyDataProperties { mask } => write!(self.writer, "CopyDataProperties {}", mask),
            DefineMethod { atom, flags } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "DefineMethod {} {}", imm, flags)
            }
            DefineMethodComputed { flags } => write!(self.writer, "DefineMethodComputed {}", flags),
            DefineClass { flags, atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "DefineClass {} {}", imm, flags)
            }
            DefineClassComputed { flags, atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "DefineClassComputed {} {}", imm, flags)
            }
            GetLoc { index } => write!(self.writer, "GetLoc {}", index.as_u32()),
            PutLoc { index } => write!(self.writer, "PutLoc {}", index.as_u32()),
            SetLoc { index } => write!(self.writer, "SetLoc {}", index.as_u32()),
            GetArg { index } => write!(self.writer, "GetArg {}", index.as_u32()),
            PutArg { index } => write!(self.writer, "PutArg {}", index.as_u32()),
            SetArg { index } => write!(self.writer, "SetArg {}", index.as_u32()),
            GetVarRef { index } => {
                let closure_var = translation.resolve_closure_var_name(func.index, index);
                write!(self.writer, "GetVarRef {}", closure_var)
            }
            PutVarRef { index } => {
                let closure_var = translation.resolve_closure_var_name(func.index, index);
                write!(self.writer, "PutVarRef {}", closure_var)
            }
            SetVarRef { index } => {
                let closure_var = translation.resolve_closure_var_name(func.index, index);
                write!(self.writer, "SetVarRef {}", closure_var)
            }
            SetLocUninit { index } => write!(self.writer, "SetLocUninit {}", index.as_u32()),
            GetLocCheck { index } => write!(self.writer, "GetLocCheck {}", index.as_u32()),
            PutLocCheck { index } => write!(self.writer, "PutLocCheck {}", index.as_u32()),
            PutLocCheckInit { index } => write!(self.writer, "PutLocCheckInit {}", index.as_u32()),
            GetLocCheckThis { index } => write!(self.writer, "GetLocCheckThis {}", index.as_u32()),
            GetVarRefCheck { index } => {
                let closure_var = translation.resolve_closure_var_name(func.index, index);
                write!(self.writer, "GetVarRefCheck {}", closure_var)
            }
            PutVarRefCheck { index } => {
                let closure_var = translation.resolve_closure_var_name(func.index, index);
                write!(self.writer, "PutVarRefCheck {}", closure_var)
            }
            PutVarRefCheckInit { index } => {
                let closure_var = translation.resolve_closure_var_name(func.index, index);
                write!(self.writer, "PutVarRefCheckInit {}", closure_var)
            }
            CloseLoc { index } => write!(self.writer, "CloseLoc {}", index),
            IfFalse { offset } => write!(self.writer, "IfFalse {}", offset),
            IfTrue { offset } => write!(self.writer, "IfTrue {}", offset),
            GoTo { offset } => write!(self.writer, "GoTo {}", offset),
            Catch { diff } => write!(self.writer, "Catch {}", diff),
            GoSub { diff } => write!(self.writer, "GoSub {}", diff),
            Ret => write!(self.writer, "Ret"),
            NipCatch => write!(self.writer, "NipCatch"),
            ToObject => write!(self.writer, "ToObject"),
            ToPropKey => write!(self.writer, "ToPropKey"),
            ToPropKey2 => write!(self.writer, "ToPropKey2"),
            WithGetVar {
                atom,
                diff,
                is_with,
            } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "WithGetVar {} {} {}", imm, diff, is_with)
            }
            WithPutVar {
                atom,
                diff,
                is_with,
            } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "WithPutVar {} {} {}", imm, diff, is_with)
            }
            WithDeleteVar {
                atom,
                diff,
                is_with,
            } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "WithDeleteVar {} {} {}", imm, diff, is_with)
            }
            WithMakeRef {
                atom,
                diff,
                is_with,
            } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "WithMakeRef {} {} {}", imm, diff, is_with)
            }
            WithGetRef {
                atom,
                diff,
                is_with,
            } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "WithGetRef {} {} {}", imm, diff, is_with)
            }
            WithGetRefUndef {
                atom,
                diff,
                is_with,
            } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "WithGetRefUndef {} {} {}", imm, diff, is_with)
            }
            MakeLocRef { atom, idx } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "MakeLocRef {} {}", imm, idx)
            }
            MakeArgRef { atom, idx } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "MakeArgRef {} {}", imm, idx)
            }
            MakeVarRefRef { atom, idx } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "MakeVarRefRef {} {}", imm, idx)
            }
            MakeVarRef { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "MakeVarRef {}", imm)
            }
            ForInStart => write!(self.writer, "ForInStart"),
            ForOfStart => write!(self.writer, "ForOfStart"),
            ForAwaitOfStart => write!(self.writer, "ForAwaitOfStart"),
            ForInNext => write!(self.writer, "ForInNext"),
            ForOfNext { offset } => write!(self.writer, "ForOfNext {}", offset),
            IteratorCheckObject => write!(self.writer, "IteratorCheckObject"),
            IteratorGetValueDone => write!(self.writer, "IteratorGetValueDone"),
            IteratorClose => write!(self.writer, "IteratorClose"),
            IteratorNext => write!(self.writer, "IteratorNext"),
            IteratorCall { flags } => write!(self.writer, "IteratorCall {}", flags),
            InitialYield => write!(self.writer, "InitialYield"),
            Yield => write!(self.writer, "Yield"),
            YieldStar => write!(self.writer, "YieldStar"),
            AsyncYieldStar => write!(self.writer, "AsyncYieldStar"),
            Await => write!(self.writer, "Await"),
            Neg => write!(self.writer, "Neg"),
            Plus => write!(self.writer, "Plus"),
            Dec => write!(self.writer, "Dec"),
            Inc => write!(self.writer, "Inc"),
            PostDec => write!(self.writer, "PostDec"),
            PostInc => write!(self.writer, "PostInc"),
            DecLoc { index } => write!(self.writer, "DecLoc {}", index.as_u32()),
            IncLoc { index } => write!(self.writer, "IncLoc {}", index.as_u32()),
            AddLoc { index } => write!(self.writer, "AddLoc {}", index.as_u32()),
            Not => write!(self.writer, "Not"),
            LNot => write!(self.writer, "LNot"),
            TypeOf => write!(self.writer, "TypeOf"),
            Delete => write!(self.writer, "Delete"),
            DeleteVar { atom } => {
                let imm = translation.resolve_atom_name(atom);
                write!(self.writer, "DeleteVar {}", imm)
            }
            Mul => write!(self.writer, "Mul"),
            Div => write!(self.writer, "Div"),
            Mod => write!(self.writer, "Mod"),
            Add => write!(self.writer, "Add"),
            Sub => write!(self.writer, "Sub"),
            Pow => write!(self.writer, "Pow"),
            Shl => write!(self.writer, "Shl"),
            Sar => write!(self.writer, "Sar"),
            Shr => write!(self.writer, "Shr"),
            Lt => write!(self.writer, "Lt"),
            Lte => write!(self.writer, "Lte"),
            Gt => write!(self.writer, "Gt"),
            Gte => write!(self.writer, "Gte"),
            InstanceOf => write!(self.writer, "InstanceOf"),
            In => write!(self.writer, "In"),
            Eq => write!(self.writer, "Eq"),
            Neq => write!(self.writer, "Neq"),
            StrictEq => write!(self.writer, "StrictEq"),
            StrictNeq => write!(self.writer, "StrictNeq"),
            And => write!(self.writer, "And"),
            Xor => write!(self.writer, "Xor"),
            Or => write!(self.writer, "Or"),
            UndefOrNull => write!(self.writer, "UndefOrNull"),
            PrivateIn => write!(self.writer, "PrivateIn"),
            MulPow10 => write!(self.writer, "MulPow10"),
            MathMod => write!(self.writer, "MathMod"),
            Nop => write!(self.writer, "Nop"),
            PushMinus1 => write!(self.writer, "PushMinus1"),
            Push0 => write!(self.writer, "Push0"),
            Push1 => write!(self.writer, "Push1"),
            Push2 => write!(self.writer, "Push2"),
            Push3 => write!(self.writer, "Push3"),
            Push4 => write!(self.writer, "Push4"),
            Push5 => write!(self.writer, "Push5"),
            Push6 => write!(self.writer, "Push6"),
            Push7 => write!(self.writer, "Push7"),
            PushI8 { val } => write!(self.writer, "PushI8 {}", val),
            PushI16 { val } => write!(self.writer, "PushI16 {}", val),
            PushConst8 { index } => write!(self.writer, "PushConst8 {}", index),
            // FIXME: Should be able to figure out the closure name.
            FClosure8 { index } => {
                let closure = translation.resolve_func_name(func.index, Some(index));
                write!(self.writer, "FClosure8 {}", closure)
            }
            PushEmptyString => write!(self.writer, "PushEmptyString"),
            GetLoc8 { index } => write!(self.writer, "GetLoc8 {}", index.as_u32()),
            PutLoc8 { index } => write!(self.writer, "PutLoc8 {}", index.as_u32()),
            SetLoc8 { index } => write!(self.writer, "SetLoc8 {}", index.as_u32()),
            GetLoc0 => write!(self.writer, "GetLoc0"),
            GetLoc1 => write!(self.writer, "GetLoc1"),
            GetLoc2 => write!(self.writer, "GetLoc2"),
            GetLoc3 => write!(self.writer, "GetLoc3"),
            PutLoc0 => write!(self.writer, "PutLoc0"),
            PutLoc1 => write!(self.writer, "PutLoc1"),
            PutLoc2 => write!(self.writer, "PutLoc2"),
            PutLoc3 => write!(self.writer, "PutLoc3"),
            SetLoc0 => write!(self.writer, "SetLoc0"),
            SetLoc1 => write!(self.writer, "SetLoc1"),
            SetLoc2 => write!(self.writer, "SetLoc2"),
            SetLoc3 => write!(self.writer, "SetLoc3"),
            GetArg0 => write!(self.writer, "GetArg0"),
            GetArg1 => write!(self.writer, "GetArg1"),
            GetArg2 => write!(self.writer, "GetArg2"),
            GetArg3 => write!(self.writer, "GetArg3"),
            PutArg0 => write!(self.writer, "PutArg0"),
            PutArg1 => write!(self.writer, "PutArg1"),
            PutArg2 => write!(self.writer, "PutArg2"),
            PutArg3 => write!(self.writer, "PutArg3"),
            SetArg0 => write!(self.writer, "SetArg0"),
            SetArg1 => write!(self.writer, "SetArg1"),
            SetArg2 => write!(self.writer, "SetArg2"),
            SetArg3 => write!(self.writer, "SetArg3"),
            GetVarRef0 => write!(self.writer, "GetVarRef0"),
            GetVarRef1 => write!(self.writer, "GetVarRef1"),
            GetVarRef2 => write!(self.writer, "GetVarRef2"),
            GetVarRef3 => write!(self.writer, "GetVarRef3"),
            PutVarRef0 => write!(self.writer, "PutVarRef0"),
            PutVarRef1 => write!(self.writer, "PutVarRef1"),
            PutVarRef2 => write!(self.writer, "PutVarRef2"),
            PutVarRef3 => write!(self.writer, "PutVarRef3"),
            SetVarRef0 => write!(self.writer, "SetVarRef0"),
            SetVarRef1 => write!(self.writer, "SetVarRef1"),
            SetVarRef2 => write!(self.writer, "SetVarRef2"),
            SetVarRef3 => write!(self.writer, "SetVarRef3"),
            GetLength => write!(self.writer, "GetLength"),
            IfFalse8 { offset } => write!(self.writer, "IfFalse8 {}", offset),
            IfTrue8 { offset } => write!(self.writer, "IfTrue8 {}", offset),
            GoTo8 { offset } => write!(self.writer, "GoTo8 {}", offset),
            GoTo16 { offset } => write!(self.writer, "GoTo16 {}", offset),
            Call0 => write!(self.writer, "Call0"),
            Call1 => write!(self.writer, "Call1"),
            Call2 => write!(self.writer, "Call2"),
            Call3 => write!(self.writer, "Call3"),
            IsUndefined => write!(self.writer, "IsUndefined"),
            IsNull => write!(self.writer, "IsNull"),
            TypeOfIsUndefined => write!(self.writer, "TypeOfIsUndefined"),
            TypeOfIsFunction => write!(self.writer, "TypeOfIsFunction"),
            SpecialObject { argument } => write!(self.writer, "SpecialObject {}", argument),
        }?;

        Ok(())
    }
}
