//! Bytecode sections.

use crate::{op::Opcode, readers::BinaryReader};
use anyhow::{bail, Result};

/// The start section of the bytecode.
#[derive(Debug, Clone)]
pub struct HeaderSection {
    /// The number of interned atoms in the bytecode.
    pub atom_count: u32,
    /// The entire list of atom names accessible to the module, including built-in atoms.
    pub atoms: Vec<String>,
}

impl HeaderSection {
    /// Creates a new [HeaderSection].
    pub(crate) fn new(atom_count: u32, atoms: Vec<String>) -> Self {
        Self { atom_count, atoms }
    }
}
#[derive(Debug, Clone)]
pub struct ModuleSection {
    /// The index of the module name.
    name_index: u32,
    /// The names of required modules, as index into the atom table.
    req_modules: Vec<u32>,
    /// The list of exports from this module.
    exports: Vec<ModuleExportEntry>,
    /// The names of the star export entries, as index into the atom table.
    star_exports: Vec<u32>,
    /// The list of imports from this module.
    imports: Vec<ModuleImportEntry>,
    /// Whether the module has top-level await.
    has_tla: u8,
}

impl<'a> ModuleSection {
    /// Creates a new [ModuleSection].
    pub(crate) fn new(
        name_index: u32,
        req_modules: Vec<u32>,
        exports: Vec<ModuleExportEntry>,
        star_exports: Vec<u32>,
        imports: Vec<ModuleImportEntry>,
        has_tla: u8,
    ) -> Self {
        Self {
            name_index,
            req_modules,
            exports,
            star_exports,
            imports,
            has_tla,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModuleExportEntry {
    Local {
        var_idx: u32,
        export_name_idx: u32,
    },
    Indirect {
        module_idx: u32,
        local_name_idx: u32,
        export_name_idx: u32,
    },
}

#[derive(Debug, Clone)]
pub struct ModuleImportEntry {
    pub var_idx: u32,
    pub name_idx: u32,
    pub req_module_idx: u32,
}

/// Function section metadata.
#[derive(Debug, Default, Copy, Clone)]
pub struct FunctionSectionHeader {
    /// Function flags.
    pub flags: u16,
    /// The index of the function name.
    pub name_index: u32,
    /// The argument count.
    pub arg_count: u32,
    /// The variable count.
    pub var_count: u32,
    /// The{ defined argument count.
    pub defined_arg_count: u32,
    /// The stack size.
    pub stack_size: u32,
    /// The closure count.
    pub closure_count: u32,
    /// The number of elements in the constant pool.
    pub constant_pool_size: u32,
    /// The function bytecode length.
    pub bytecode_len: u32,
    /// The number of locals.
    pub local_count: u32,
}

/// Closure variable information.
#[derive(Debug, Default, Copy, Clone)]
pub struct FunctionClosure {
    pub name: u32,
    pub index: u32,
    pub flags: u8,
}

/// Function local variable information.
#[derive(Debug, Default, Copy, Clone)]
pub struct FunctionLocal {
    pub name: u32,
    pub scope_level: u32,
    pub scope_next: u32,
    pub flags: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct DebugInfo<'a> {
    filename: u32,
    lineno: u32,
    colno: u32,
    line_debug_reader: BinaryReader<'a>,
    col_debug_reader: BinaryReader<'a>,
}

impl<'a> DebugInfo<'a> {
    /// Create a new [DebugInfo].
    pub fn new(
        filename: u32,
        lineno: u32,
        colno: u32,
        line_debug_reader: BinaryReader<'a>,
        col_debug_reader: BinaryReader<'a>,
    ) -> Self {
        Self {
            filename,
            lineno,
            colno,
            line_debug_reader,
            col_debug_reader,
        }
    }
}

/// Bytecode operators reader.
pub struct OperatorReader<'a> {
    /// The underlying binary reader.
    reader: BinaryReader<'a>,
}

impl<'a> OperatorReader<'a> {
    pub fn new(reader: BinaryReader<'a>) -> Self {
        Self { reader }
    }

    /// Read the next operator.
    pub fn read(&mut self) -> Result<Opcode> {
        use Opcode::*;
        let byte = self.reader.read_u8()?;
        let op = match byte {
            0 => Invalid,
            1 => PushI32 {
                value: i32::try_from(self.reader.read_u32()?)?,
            },
            2 => PushConst {
                index: self.reader.read_u32()?,
            },
            3 => FClosure {
                index: self.reader.read_u32()?,
            },
            4 => PushAtomValue {
                val: self.reader.read_u32()?,
            },
            5 => PrivateSymbol {
                val: self.reader.read_u32()?,
            },
            6 => Undefined,
            7 => Null,
            8 => PushThis,
            9 => PushFalse,
            10 => PushTrue,
            11 => Object,
            12 => SpecialObject {
                argument: self.reader.read_u8()? as i32,
            },
            13 => Rest {
                first: self.reader.read_u16()?,
            },
            14 => Drop,
            15 => Nip,
            16 => Nip1,
            17 => Dup,
            18 => Dup1,
            19 => Dup2,
            20 => Dup3,
            21 => Insert2,
            22 => Insert3,
            23 => Insert4,
            24 => Perm3,
            25 => Perm4,
            26 => Perm5,
            27 => Swap,
            28 => Swap2,
            29 => Rot3L,
            30 => Rot3R,
            31 => Rot4L,
            32 => Rot5L,
            33 => CallConstructor {
                argc: self.reader.read_u16()?,
            },
            34 => Call {
                argc: self.reader.read_u16()?,
            },
            35 => TailCall {
                argc: self.reader.read_u16()?,
            },
            36 => CallMethod {
                argc: self.reader.read_u16()?,
            },
            37 => TailCallMethod {
                argc: self.reader.read_u16()?,
            },
            38 => ArrayFrom {
                argc: self.reader.read_u16()?,
            },
            39 => Apply {
                magic: self.reader.read_u16()?,
            },
            40 => Return,
            41 => ReturnUndef,
            42 => CheckCtorReturn,
            43 => CheckCtor,
            44 => CheckBrand,
            45 => AddBrand,
            46 => ReturnAsync,
            47 => Throw,
            48 => {
                let atom = self.reader.read_u32()?;
                let ty = self.reader.read_u8()?;
                ThrowError { atom, ty }
            }
            49 => {
                let argc = self.reader.read_u16()?;
                let scope = self.reader.read_u16()? - 1;
                Eval { scope, argc }
            }
            50 => ApplyEval {
                scope: self.reader.read_u16()? - 1,
            },
            51 => Regexp,
            52 => GetSuper,
            53 => Import,
            54 => CheckVar {
                atom: self.reader.read_u32()?,
            },
            55 => GetVarUndef {
                atom: self.reader.read_u32()?,
            },
            56 => GetVar {
                atom: self.reader.read_u32()?,
            },
            57 => PutVar {
                atom: self.reader.read_u32()?,
            },
            58 => PutVarInit {
                atom: self.reader.read_u32()?,
            },
            59 => PutVarStrict {
                atom: self.reader.read_u32()?,
            },
            60 => GetRefValue,
            61 => PutRefValue,
            62 | 63 => {
                let atom = self.reader.read_u32()?;
                let flags = self.reader.read_u8()?;
                if byte == 62 {
                    DefineVar { flags, atom }
                } else {
                    CheckDefineVar { flags, atom }
                }
            }
            64 => {
                let atom = self.reader.read_u32()?;
                let flags = self.reader.read_u8()?;
                DefineFunc { flags, atom }
            }
            65 => GetField {
                atom: self.reader.read_u32()?,
            },
            66 => GetField2 {
                atom: self.reader.read_u32()?,
            },
            67 => PutField {
                atom: self.reader.read_u32()?,
            },
            68 => GetPrivateField,
            69 => PutPrivateField,
            70 => DefinePrivateField,
            71 => GetArrayEl,
            72 => GetArrayEl2,
            73 => PutArrayEl,
            74 => GetSuperValue,
            75 => PutSuperValue,
            76 => DefineField {
                atom: self.reader.read_u32()?,
            },
            77 => SetName {
                atom: self.reader.read_u32()?,
            },
            78 => SetNameComputed,
            79 => SetProto,
            80 => SetHomeObject,
            81 => DefineArrayEl,
            82 => Append,
            83 => CopyDataProperties {
                mask: self.reader.read_u8()?,
            },
            84 => {
                let atom = self.reader.read_u32()?;
                let flags = self.reader.read_u8()?;
                DefineMethod { atom, flags }
            }
            85 => DefineMethodComputed {
                flags: self.reader.read_u8()?,
            },
            86 | 87 => {
                let atom = self.reader.read_u32()?;
                let flags = self.reader.read_u8()?;
                if byte == 86 {
                    DefineClass { atom, flags }
                } else {
                    DefineClassComputed { atom, flags }
                }
            }
            88 => GetLoc {
                index: self.reader.read_u16()?,
            },
            89 => PutLoc {
                index: self.reader.read_u16()?,
            },
            90 => SetLoc {
                index: self.reader.read_u16()?,
            },
            91 => GetArg {
                index: self.reader.read_u16()?,
            },
            92 => PutArg {
                index: self.reader.read_u16()?,
            },
            93 => SetArg {
                index: self.reader.read_u16()?,
            },
            94 => GetVarRef {
                index: self.reader.read_u16()?,
            },
            95 => PutVarRef {
                index: self.reader.read_u16()?,
            },
            96 => SetVarRef {
                index: self.reader.read_u16()?,
            },
            97 => SetLocUninit {
                index: self.reader.read_u16()?,
            },
            98 => GetLocCheck {
                index: self.reader.read_u16()?,
            },
            99 => PutLocCheck {
                index: self.reader.read_u16()?,
            },
            100 => PutLocCheckInit {
                index: self.reader.read_u16()?,
            },
            101 => GetLocCheckThis {
                index: self.reader.read_u16()?,
            },
            102 => GetVarRefCheck {
                index: self.reader.read_u16()?,
            },
            103 => PutVarRefCheck {
                index: self.reader.read_u16()?,
            },
            104 => PutVarRefCheckInit {
                index: self.reader.read_u16()?,
            },
            105 => CloseLoc {
                index: self.reader.read_u16()?,
            },
            106 => IfFalse {
                offset: self.reader.read_u32()?,
            },
            107 => IfTrue {
                offset: self.reader.read_u32()?,
            },
            108 => GoTo {
                offset: self.reader.read_u32()?,
            },
            109 => Catch {
                diff: self.reader.read_u32()?,
            },
            110 => GoSub {
                diff: self.reader.read_u32()?,
            },
            111 => Ret,
            112 => NipCatch,
            113 => ToObject,
            114 => ToPropKey,
            115 => ToPropKey2,
            116 | 117 | 118 | 119 | 120 | 121 => {
                let atom = self.reader.read_u32()?;
                let diff = self.reader.read_u32()?;
                let is_with = self.reader.read_u8()?;
                match byte {
                    116 => WithGetVar {
                        atom,
                        diff,
                        is_with,
                    },
                    117 => WithPutVar {
                        atom,
                        diff,
                        is_with,
                    },
                    118 => WithDeleteVar {
                        atom,
                        diff,
                        is_with,
                    },
                    119 => WithMakeRef {
                        atom,
                        diff,
                        is_with,
                    },
                    120 => WithGetRef {
                        atom,
                        diff,
                        is_with,
                    },
                    121 => WithGetRefUndef {
                        atom,
                        diff,
                        is_with,
                    },
                    _ => unreachable!(),
                }
            }
            122 => {
                let atom = self.reader.read_u32()?;
                let idx = self.reader.read_u16()?;
                MakeLocRef { atom, idx }
            }
            123 => {
                let atom = self.reader.read_u32()?;
                let idx = self.reader.read_u16()?;
                MakeArgRef { atom, idx }
            }
            124 => {
                let atom = self.reader.read_u32()?;
                let idx = self.reader.read_u16()?;
                MakeVarRefRef { atom, idx }
            }
            125 => MakeVarRef {
                atom: self.reader.read_u32()?,
            },
            126 => ForInStart,
            127 => ForOfStart,
            128 => ForAwaitOfStart,
            129 => ForInNext,
            130 => ForOfNext {
                offset: self.reader.read_u8()?,
            },
            131 => IteratorCheckObject,
            132 => IteratorGetValueDone,
            133 => IteratorClose,
            134 => IteratorNext,
            135 => IteratorCall {
                flags: self.reader.read_u8()?,
            },
            136 => InitialYield,
            137 => Yield,
            138 => YieldStar,
            139 => AsyncYieldStar,
            140 => Await,
            141 => Neg,
            142 => Plus,
            143 => Dec,
            144 => Inc,
            145 => PostDec,
            146 => PostInc,
            147 => DecLoc {
                index: self.reader.read_u8()?,
            },
            148 => IncLoc {
                index: self.reader.read_u8()?,
            },
            149 => AddLoc {
                index: self.reader.read_u8()?,
            },
            150 => Not,
            151 => LNot,
            152 => TypeOf,
            153 => Delete,
            154 => DeleteVar {
                atom: self.reader.read_u32()?,
            },
            155 => Mul,
            156 => Div,
            157 => Mod,
            158 => Add,
            159 => Sub,
            160 => Pow,
            161 => Shl,
            162 => Sar,
            163 => Shr,
            164 => Lt,
            165 => Lte,
            166 => Gt,
            167 => Gte,
            168 => InstanceOf,
            169 => In,
            170 => Eq,
            171 => Neq,
            172 => StrictEq,
            173 => StrictNeq,
            174 => And,
            175 => Xor,
            176 => Or,
            177 => UndefOrNull,
            178 => PrivateIn,
            179 => MulPow10,
            180 => MathMod,
            181 => Nop,
            182 => PushMinus1,
            183 => Push0,
            184 => Push1,
            185 => Push2,
            186 => Push3,
            187 => Push4,
            188 => Push5,
            189 => Push6,
            190 => Push7,
            191 => PushI8 {
                val: self.reader.read_u8()? as i8,
            },
            192 => PushI16 {
                val: self.reader.read_u16()? as i16,
            },
            193 => PushConst8 {
                index: self.reader.read_u8()?,
            },
            194 => FClosure8 {
                index: self.reader.read_u8()?,
            },
            195 => PushEmptyString,
            196 => GetLoc8 {
                index: self.reader.read_u8()?,
            },
            197 => PutLoc8 {
                index: self.reader.read_u8()?,
            },
            198 => SetLoc8 {
                index: self.reader.read_u8()?,
            },
            199 => GetLoc0,
            200 => GetLoc1,
            201 => GetLoc2,
            202 => GetLoc3,
            203 => PutLoc0,
            204 => PutLoc1,
            205 => PutLoc2,
            206 => PutLoc3,
            207 => SetLoc0,
            208 => SetLoc1,
            209 => SetLoc2,
            210 => SetLoc3,
            211 => GetArg0,
            212 => GetArg1,
            213 => GetArg2,
            214 => GetArg3,
            215 => PutArg0,
            216 => PutArg1,
            217 => PutArg2,
            218 => PutArg3,
            219 => SetArg0,
            220 => SetArg1,
            221 => SetArg2,
            222 => SetArg3,
            223 => GetVarRef0,
            224 => GetVarRef1,
            225 => GetVarRef2,
            226 => GetVarRef3,
            227 => PutVarRef0,
            228 => PutVarRef1,
            229 => PutVarRef2,
            230 => PutVarRef3,
            231 => SetVarRef0,
            232 => SetVarRef1,
            233 => SetVarRef2,
            234 => SetVarRef3,
            235 => GetLength,
            236 => IfFalse8 {
                alternate_offset: self.reader.read_u8()?,
            },
            237 => IfTrue8 {
                offset: self.reader.read_u8()?,
            },
            238 => GoTo8 {
                offset: self.reader.read_u8()?,
            },
            239 => GoTo16 {
                offset: self.reader.read_u16()?,
            },
            240 => Call0,
            241 => Call1,
            242 => Call2,
            243 => Call3,
            244 => IsUndefined,
            245 => IsNull,
            246 => TypeOfIsUndefined,
            247 => TypeOfIsFunction,
            x => bail!("Unsupported opcode {x}"),
        };
        Ok(op)
    }

    /// Is the reader done?.
    pub fn done(&self) -> bool {
        self.reader.offset >= self.reader.data().len()
    }
}

/// A function section.
#[derive(Debug, Clone)]
pub struct FunctionSection<'a> {
    /// The function section header.
    header: FunctionSectionHeader,
    /// The parsed local variables.
    locals: Vec<FunctionLocal>,
    /// The locals reader.
    locals_reader: BinaryReader<'a>,
    /// The parsed closures.
    closures: Vec<FunctionClosure>,
    /// The closures reader.
    closures_reader: BinaryReader<'a>,
    /// The parsed opcodes.
    operators: Vec<Opcode>,
    /// The operators reader.
    operators_reader: BinaryReader<'a>,
    /// The function debug information.
    debug: Option<DebugInfo<'a>>,
}

impl<'a> FunctionSection<'a> {
    /// Create a new [FunctionSection].
    pub(crate) fn new(
        header: FunctionSectionHeader,
        locals: Vec<FunctionLocal>,
        locals_reader: BinaryReader<'a>,
        closures: Vec<FunctionClosure>,
        closures_reader: BinaryReader<'a>,
        operators_reader: BinaryReader<'a>,
        debug: Option<DebugInfo<'a>>,
    ) -> Self {
        let mut operators = vec![];
        let mut op_reader = OperatorReader::new(operators_reader);
        while !op_reader.done() {
            if let Ok(op) = op_reader.read() {
                operators.push(op);
            }
        }
        Self {
            header,
            locals,
            locals_reader,
            closures,
            closures_reader,
            operators,
            operators_reader,
            debug,
        }
    }

    /// Returns the function section header.
    pub fn header(&self) -> &FunctionSectionHeader {
        &self.header
    }

    /// Get an operators reader.
    pub fn operators_reader(&self) -> OperatorReader {
        OperatorReader::new(self.operators_reader)
    }
}
