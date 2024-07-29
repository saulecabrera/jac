use std::any::type_name;

use crate::{readers::BinaryReader, JsModule};
use anyhow::{bail, Result};

/// A QuickJS operator code.
#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Opcode {
    /// A marker, never emitted.
    Invalid = 0,
    /// Push an `i32` value.
    PushI32 {
        value: i32,
    },
    /// Push a constant value.
    PushConst {
        /// The index of the constant in the constant pool.
        index: u32,
    },
    /// Push a function closure value.
    FClosure {
        /// The index of the closure in the constant pool.
        index: u32,
    },
    /// Push an atom constant.
    PushAtomValue {
        /// The immediate value of the atom.
        atom: u32,
    },
    /// Push a private symbol from an atom immediate.
    PrivateSymbol {
        /// The immediate value of the atom.
        atom: u32,
    },
    /// Push undefined value.
    Undefined,
    /// Push a null value.
    Null,
    /// Push the current object.
    PushThis,
    /// Push a false constant.
    PushFalse,
    /// Puhs a true constant.
    PushTrue,
    /// Push a new object.
    Object,
    /// Push a special object.
    SpecialObject {
        /// The special object argument.
        argument: i32,
    },
    // TODO: Verify this.
    /// Rest arguments.
    Rest {
        /// The first argument.
        first: u16,
    },
    /// Drop the top value.
    Drop,
    /// Drop the second top value.
    Nip,
    /// Drop the third top value.
    Nip1,
    /// Duplicate the top value, pushing the new value at the stack top.
    Dup,
    /// Similar to [Opcode::Dup] but puts the new value in the second top most
    /// position.
    Dup1,
    /// Duplicate the top two values, pushing the new values at the stack top.
    Dup2,
    /// Duplicate the top three values pushing the values at the stack top.
    Dup3,

    // TODO: Skipping comments for now.
    Insert2,
    Insert3,
    Insert4,
    Perm3,
    Perm4,
    Perm5,
    Swap,
    Swap2,
    Rot3L,
    Rot3R,
    Rot4L,
    Rot5L,

    CallConstructor {
        argc: u16,
    },
    Call {
        argc: u16,
    },
    TailCall {
        argc: u16,
    },
    CallMethod {
        argc: u16,
    },
    TailCallMethod {
        argc: u16,
    },

    ArrayFrom {
        argc: u16,
    },
    Apply {
        magic: u16,
    },
    Return,
    ReturnUndef,
    CheckCtorReturn,
    CheckCtor,
    CheckBrand,
    AddBrand,
    ReturnAsync,
    Throw,
    ThrowError {
        ty: u8,
        atom: u32,
    },
    Eval {
        scope: u16,
        argc: u16,
    },
    ApplyEval {
        scope: u16,
    },
    Regexp,

    GetSuper,
    Import,

    CheckVar {
        atom: u32,
    },
    GetVarUndef {
        atom: u32,
    },
    GetVar {
        atom: u32,
    },
    PutVar {
        atom: u32,
    },
    PutVarInit {
        atom: u32,
    },
    PutVarStrict {
        atom: u32,
    },
    GetRefValue,
    PutRefValue,
    DefineVar {
        flags: u8,
        atom: u32,
    },
    CheckDefineVar {
        flags: u8,
        atom: u32,
    },
    DefineFunc {
        flags: u8,
        atom: u32,
    },
    GetField {
        atom: u32,
    },
    GetField2 {
        atom: u32,
    },
    PutField {
        atom: u32,
    },
    GetPrivateField,
    PutPrivateField,
    DefinePrivateField,
    GetArrayEl,
    GetArrayEl2,
    PutArrayEl,
    GetSuperValue,
    PutSuperValue,
    DefineField {
        atom: u32,
    },
    SetName {
        atom: u32,
    },
    SetNameComputed,
    SetProto,
    SetHomeObject,
    DefineArrayEl,
    Append,
    CopyDataProperties {
        mask: u8,
    },
    DefineMethod {
        atom: u32,
        flags: u8,
    },
    DefineMethodComputed {
        flags: u8,
    },
    DefineClass {
        flags: u8,
        atom: u32,
    },
    DefineClassComputed {
        flags: u8,
        atom: u32,
    },
    GetLoc {
        // index to local variable list (after arg list)
        index: u16,
    },
    PutLoc {
        index: u16,
    },
    SetLoc {
        index: u16,
    },
    GetArg {
        // index to arg list
        index: u16,
    },
    PutArg {
        index: u16,
    },
    SetArg {
        index: u16,
    },
    GetVarRef {
        // index to the closures list
        index: u16,
    },
    PutVarRef {
        index: u16,
    },
    SetVarRef {
        index: u16,
    },
    SetLocUninit {
        index: u16,
    },
    GetLocCheck {
        index: u16,
    },
    PutLocCheck {
        index: u16,
    },
    PutLocCheckInit {
        index: u16,
    },
    GetLocCheckThis {
        index: u16,
    },
    GetVarRefCheck {
        index: u16,
    },
    PutVarRefCheck {
        index: u16,
    },
    PutVarRefCheckInit {
        index: u16,
    },
    CloseLoc {
        // TODO: figure out what this is
        index: u16,
    },
    IfFalse {
        // amount of forward jump bytes
        offset: i32,
    },
    IfTrue {
        offset: i32,
    },
    GoTo {
        offset: i32,
    },
    Catch {
        diff: u32,
    },
    GoSub {
        diff: u32,
    },
    Ret,
    NipCatch,
    ToObject,
    ToPropKey,
    ToPropKey2,
    WithGetVar {
        atom: u32,
        diff: u32,
        is_with: u8,
    },
    WithPutVar {
        atom: u32,
        diff: u32,
        is_with: u8,
    },
    WithDeleteVar {
        atom: u32,
        diff: u32,
        is_with: u8,
    },
    WithMakeRef {
        atom: u32,
        diff: u32,
        is_with: u8,
    },
    WithGetRef {
        atom: u32,
        diff: u32,
        is_with: u8,
    },
    WithGetRefUndef {
        atom: u32,
        diff: u32,
        is_with: u8,
    },
    MakeLocRef {
        atom: u32,
        idx: u16,
    },
    MakeArgRef {
        atom: u32,
        idx: u16,
    },
    MakeVarRefRef {
        atom: u32,
        idx: u16,
    },
    MakeVarRef {
        atom: u32,
    },
    ForInStart,
    ForOfStart,
    ForAwaitOfStart,
    ForInNext,
    ForOfNext {
        offset: i8,
    },
    IteratorCheckObject,
    IteratorGetValueDone,
    IteratorClose,
    IteratorNext,
    IteratorCall {
        flags: u8,
    },
    InitialYield,
    Yield,
    YieldStar,
    AsyncYieldStar,
    Await,
    Neg,
    Plus,
    Dec,
    Inc,
    PostDec,
    PostInc,
    DecLoc {
        index: u8,
    },
    IncLoc {
        index: u8,
    },
    AddLoc {
        index: u8,
    },
    Not,
    LNot,
    TypeOf,
    Delete,
    DeleteVar {
        atom: u32,
    },
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Pow,
    Shl,
    Sar,
    Shr,
    Lt,
    Lte,
    Gt,
    Gte,
    InstanceOf,
    In,
    Eq,
    Neq,
    StrictEq,
    StrictNeq,
    And,
    Xor,
    Or,
    UndefOrNull,
    PrivateIn,
    MulPow10,
    MathMod,
    // Short opcodes.
    Nop,
    PushMinus1,
    Push0,
    Push1,
    Push2,
    Push3,
    Push4,
    Push5,
    Push6,
    Push7,
    PushI8 {
        val: i8,
    },
    PushI16 {
        val: i16,
    },
    PushConst8 {
        index: u8,
    },
    FClosure8 {
        index: u8,
    },
    PushEmptyString,
    GetLoc8 {
        index: u8,
    },
    PutLoc8 {
        index: u8,
    },
    SetLoc8 {
        index: u8,
    },
    GetLoc0,
    GetLoc1,
    GetLoc2,
    GetLoc3,
    PutLoc0,
    PutLoc1,
    PutLoc2,
    PutLoc3,
    SetLoc0,
    SetLoc1,
    SetLoc2,
    SetLoc3,
    GetArg0,
    GetArg1,
    GetArg2,
    GetArg3,
    PutArg0,
    PutArg1,
    PutArg2,
    PutArg3,
    SetArg0,
    SetArg1,
    SetArg2,
    SetArg3,
    GetVarRef0,
    GetVarRef1,
    GetVarRef2,
    GetVarRef3,
    PutVarRef0,
    PutVarRef1,
    PutVarRef2,
    PutVarRef3,
    SetVarRef0,
    SetVarRef1,
    SetVarRef2,
    SetVarRef3,
    GetLength,
    IfFalse8 {
        offset: i8,
    },
    IfTrue8 {
        offset: i8,
    },
    GoTo8 {
        offset: i8,
    },
    GoTo16 {
        offset: i16,
    },
    Call0,
    Call1,
    Call2,
    Call3,
    IsUndefined,
    IsNull,
    TypeOfIsUndefined,
    TypeOfIsFunction,
}

impl Opcode {
    pub fn from_reader(reader: &mut BinaryReader<'_>) -> Result<(u32, Opcode)> {
        use Opcode::*;
        let pc = reader.offset as u32;
        let byte = reader.read_u8()?;
        let op = match byte {
            0 => Invalid,
            1 => PushI32 {
                value: i32::try_from(reader.read_u32()?)?,
            },
            2 => PushConst {
                index: reader.read_u32()?,
            },
            3 => FClosure {
                index: reader.read_u32()?,
            },
            4 => PushAtomValue {
                atom: reader.read_u32()?,
            },
            5 => PrivateSymbol {
                atom: reader.read_u32()?,
            },
            6 => Undefined,
            7 => Null,
            8 => PushThis,
            9 => PushFalse,
            10 => PushTrue,
            11 => Object,
            12 => SpecialObject {
                argument: reader.read_u8()? as i32,
            },
            13 => Rest {
                first: reader.read_u16()?,
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
                argc: reader.read_u16()?,
            },
            34 => Call {
                argc: reader.read_u16()?,
            },
            35 => TailCall {
                argc: reader.read_u16()?,
            },
            36 => CallMethod {
                argc: reader.read_u16()?,
            },
            37 => TailCallMethod {
                argc: reader.read_u16()?,
            },
            38 => ArrayFrom {
                argc: reader.read_u16()?,
            },
            39 => Apply {
                magic: reader.read_u16()?,
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
                let atom = reader.read_u32()?;
                let ty = reader.read_u8()?;
                ThrowError { atom, ty }
            }
            49 => {
                let argc = reader.read_u16()?;
                let scope = reader.read_u16()? - 1;
                Eval { scope, argc }
            }
            50 => ApplyEval {
                scope: reader.read_u16()? - 1,
            },
            51 => Regexp,
            52 => GetSuper,
            53 => Import,
            54 => CheckVar {
                atom: reader.read_u32()?,
            },
            55 => GetVarUndef {
                atom: reader.read_u32()?,
            },
            56 => GetVar {
                atom: reader.read_u32()?,
            },
            57 => PutVar {
                atom: reader.read_u32()?,
            },
            58 => PutVarInit {
                atom: reader.read_u32()?,
            },
            59 => PutVarStrict {
                atom: reader.read_u32()?,
            },
            60 => GetRefValue,
            61 => PutRefValue,
            62 | 63 => {
                let atom = reader.read_u32()?;
                let flags = reader.read_u8()?;
                if byte == 62 {
                    DefineVar { flags, atom }
                } else {
                    CheckDefineVar { flags, atom }
                }
            }
            64 => {
                let atom = reader.read_u32()?;
                let flags = reader.read_u8()?;
                DefineFunc { flags, atom }
            }
            65 => GetField {
                atom: reader.read_u32()?,
            },
            66 => GetField2 {
                atom: reader.read_u32()?,
            },
            67 => PutField {
                atom: reader.read_u32()?,
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
                atom: reader.read_u32()?,
            },
            77 => SetName {
                atom: reader.read_u32()?,
            },
            78 => SetNameComputed,
            79 => SetProto,
            80 => SetHomeObject,
            81 => DefineArrayEl,
            82 => Append,
            83 => CopyDataProperties {
                mask: reader.read_u8()?,
            },
            84 => {
                let atom = reader.read_u32()?;
                let flags = reader.read_u8()?;
                DefineMethod { atom, flags }
            }
            85 => DefineMethodComputed {
                flags: reader.read_u8()?,
            },
            86 | 87 => {
                let atom = reader.read_u32()?;
                let flags = reader.read_u8()?;
                if byte == 86 {
                    DefineClass { atom, flags }
                } else {
                    DefineClassComputed { atom, flags }
                }
            }
            88 => GetLoc {
                index: reader.read_u16()?,
            },
            89 => PutLoc {
                index: reader.read_u16()?,
            },
            90 => SetLoc {
                index: reader.read_u16()?,
            },
            91 => GetArg {
                index: reader.read_u16()?,
            },
            92 => PutArg {
                index: reader.read_u16()?,
            },
            93 => SetArg {
                index: reader.read_u16()?,
            },
            94 => GetVarRef {
                index: reader.read_u16()?,
            },
            95 => PutVarRef {
                index: reader.read_u16()?,
            },
            96 => SetVarRef {
                index: reader.read_u16()?,
            },
            97 => SetLocUninit {
                index: reader.read_u16()?,
            },
            98 => GetLocCheck {
                index: reader.read_u16()?,
            },
            99 => PutLocCheck {
                index: reader.read_u16()?,
            },
            100 => PutLocCheckInit {
                index: reader.read_u16()?,
            },
            101 => GetLocCheckThis {
                index: reader.read_u16()?,
            },
            102 => GetVarRefCheck {
                index: reader.read_u16()?,
            },
            103 => PutVarRefCheck {
                index: reader.read_u16()?,
            },
            104 => PutVarRefCheckInit {
                index: reader.read_u16()?,
            },
            105 => CloseLoc {
                index: reader.read_u16()?,
            },
            106 => IfFalse {
                offset: reader.read_u32()? as i32,
            },
            107 => IfTrue {
                offset: reader.read_u32()? as i32,
            },
            108 => GoTo {
                offset: reader.read_u32()? as i32,
            },
            109 => Catch {
                diff: reader.read_u32()?,
            },
            110 => GoSub {
                diff: reader.read_u32()?,
            },
            111 => Ret,
            112 => NipCatch,
            113 => ToObject,
            114 => ToPropKey,
            115 => ToPropKey2,
            116..=121 => {
                let atom = reader.read_u32()?;
                let diff = reader.read_u32()?;
                let is_with = reader.read_u8()?;
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
                let atom = reader.read_u32()?;
                let idx = reader.read_u16()?;
                MakeLocRef { atom, idx }
            }
            123 => {
                let atom = reader.read_u32()?;
                let idx = reader.read_u16()?;
                MakeArgRef { atom, idx }
            }
            124 => {
                let atom = reader.read_u32()?;
                let idx = reader.read_u16()?;
                MakeVarRefRef { atom, idx }
            }
            125 => MakeVarRef {
                atom: reader.read_u32()?,
            },
            126 => ForInStart,
            127 => ForOfStart,
            128 => ForAwaitOfStart,
            129 => ForInNext,
            130 => ForOfNext {
                offset: reader.read_u8()? as i8,
            },
            131 => IteratorCheckObject,
            132 => IteratorGetValueDone,
            133 => IteratorClose,
            134 => IteratorNext,
            135 => IteratorCall {
                flags: reader.read_u8()?,
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
                index: reader.read_u8()?,
            },
            148 => IncLoc {
                index: reader.read_u8()?,
            },
            149 => AddLoc {
                index: reader.read_u8()?,
            },
            150 => Not,
            151 => LNot,
            152 => TypeOf,
            153 => Delete,
            154 => DeleteVar {
                atom: reader.read_u32()?,
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
                val: reader.read_u8()? as i8,
            },
            192 => PushI16 {
                val: reader.read_u16()? as i16,
            },
            193 => PushConst8 {
                index: reader.read_u8()?,
            },
            194 => FClosure8 {
                index: reader.read_u8()?,
            },
            195 => PushEmptyString,
            196 => GetLoc8 {
                index: reader.read_u8()?,
            },
            197 => PutLoc8 {
                index: reader.read_u8()?,
            },
            198 => SetLoc8 {
                index: reader.read_u8()?,
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
                offset: reader.read_u8()? as i8,
            },
            237 => IfTrue8 {
                offset: reader.read_u8()? as i8,
            },
            238 => GoTo8 {
                offset: reader.read_u8()? as i8,
            },
            239 => GoTo16 {
                offset: reader.read_u16()? as i16,
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
        Ok((pc, op))
    }

    pub fn name_from_byte(byte: u8) -> String {
        match byte {
            0 => "Invalid",
            1 => "PushI32",
            2 => "PushConst",
            3 => "FClosure",
            4 => "PushAtomValue",
            5 => "PrivateSymbol",
            6 => "Undefined",
            7 => "Null",
            8 => "PushThis",
            9 => "PushFalse",
            10 => "PushTrue",
            11 => "Object",
            12 => "SpecialObject",
            13 => "Rest",
            14 => "Drop",
            15 => "Nip",
            16 => "Nip1",
            17 => "Dup",
            18 => "Dup1",
            19 => "Dup2",
            20 => "Dup3",
            21 => "Insert2",
            22 => "Insert3",
            23 => "Insert4",
            24 => "Perm3",
            25 => "Perm4",
            26 => "Perm5",
            27 => "Swap",
            28 => "Swap2",
            29 => "Rot3L",
            30 => "Rot3R",
            31 => "Rot4L",
            32 => "Rot5L",
            33 => "CallConstructor",
            34 => "Call",
            35 => "TailCall",
            36 => "CallMethod",
            37 => "TailCallMethod",
            38 => "ArrayFrom",
            39 => "Apply",
            40 => "Return",
            41 => "ReturnUndef",
            42 => "CheckCtorReturn",
            43 => "CheckCtor",
            44 => "CheckBrand",
            45 => "AddBrand",
            46 => "ReturnAsync",
            47 => "Throw",
            48 => "ThrowError",
            49 => "Eval",
            50 => "ApplyEval",
            51 => "Regexp",
            52 => "GetSuper",
            53 => "Import",
            54 => "CheckVar",
            55 => "GetVarUndef",
            56 => "GetVar",
            57 => "PutVar",
            58 => "PutVarInit",
            59 => "PutVarStrict",
            60 => "GetRefValue",
            61 => "PutRefValue",
            62 => "DefineVar",
            63 => "CheckDefineVar",
            64 => "DefineFunc",
            65 => "GetField",
            66 => "GetField2",
            67 => "PutField",
            68 => "GetPrivateField",
            69 => "PutPrivateField",
            70 => "DefinePrivateField",
            71 => "GetArrayEl",
            72 => "GetArrayEl2",
            73 => "PutArrayEl",
            74 => "GetSuperValue",
            75 => "PutSuperValue",
            76 => "DefineField",
            77 => "SetName",
            78 => "SetNameComputed",
            79 => "SetProto",
            80 => "SetHomeObject",
            81 => "DefineArrayEl",
            82 => "Append",
            83 => "CopyDataProperties",
            84 => "DefineMethod",
            85 => "DefineMethodComputed",
            86 => "DefineClass",
            87 => "DefineClassComputed",
            88 => "GetLoc",
            89 => "PutLoc",
            90 => "SetLoc",
            91 => "GetArg",
            92 => "PutArg",
            93 => "SetArg",
            94 => "GetVarRef",
            95 => "PutVarRef",
            96 => "SetVarRef",
            97 => "SetLocUninit",
            98 => "GetLocCheck",
            99 => "PutLocCheck",
            100 => "PutLocCheckInit",
            101 => "GetLocCheckThis",
            102 => "GetVarRefCheck",
            103 => "PutVarRefCheck",
            104 => "PutVarRefCheckInit",
            105 => "CloseLoc",
            106 => "IfFalse",
            107 => "IfTrue",
            108 => "GoTo",
            109 => "Catch",
            110 => "GoSub",
            111 => "Ret",
            112 => "NipCatch",
            113 => "ToObject",
            114 => "ToPropKey",
            115 => "ToPropKey2",
            116 => "WithGetVar",
            117 => "WithPutVar",
            118 => "WithDeleteVar",
            119 => "WithMakeRef",
            120 => "WithGetRef",
            121 => "WithGetRefUndef",
            122 => "MakeLocRef",
            123 => "MakeArgRef",
            124 => "MakeVarRefRef",
            125 => "MakeVarRef",
            126 => "ForInStart",
            127 => "ForOfStart",
            128 => "ForAwaitOfStart",
            129 => "ForInNext",
            130 => "ForOfNext",
            131 => "IteratorCheckObject",
            132 => "IteratorGetValueDone",
            133 => "IteratorClose",
            134 => "IteratorNext",
            135 => "IteratorCall",
            136 => "InitialYield",
            137 => "Yield",
            138 => "YieldStar",
            139 => "AsyncYieldStar",
            140 => "Await",
            141 => "Neg",
            142 => "Plus",
            143 => "Dec",
            144 => "Inc",
            145 => "PostDec",
            146 => "PostInc",
            147 => "DecLoc",
            148 => "IncLoc",
            149 => "AddLoc",
            150 => "Not",
            151 => "LNot",
            152 => "TypeOf",
            153 => "Delete",
            154 => "DeleteVar",
            155 => "Mul",
            156 => "Div",
            157 => "Mod",
            158 => "Add",
            159 => "Sub",
            160 => "Pow",
            161 => "Shl",
            162 => "Sar",
            163 => "Shr",
            164 => "Lt",
            165 => "Lte",
            166 => "Gt",
            167 => "Gte",
            168 => "InstanceOf",
            169 => "In",
            170 => "Eq",
            171 => "Neq",
            172 => "StrictEq",
            173 => "StrictNeq",
            174 => "And",
            175 => "Xor",
            176 => "Or",
            177 => "UndefOrNull",
            178 => "PrivateIn",
            179 => "MulPow10",
            180 => "MathMod",
            181 => "Nop",
            182 => "PushMinus1",
            183 => "Push0",
            184 => "Push1",
            185 => "Push2",
            186 => "Push3",
            187 => "Push4",
            188 => "Push5",
            189 => "Push6",
            190 => "Push7",
            191 => "PushI8",
            192 => "PushI16",
            193 => "PushConst8",
            194 => "FClosure8",
            195 => "PushEmptyString",
            196 => "GetLoc8",
            197 => "PutLoc8",
            198 => "SetLoc8",
            199 => "GetLoc0",
            200 => "GetLoc1",
            201 => "GetLoc2",
            202 => "GetLoc3",
            203 => "PutLoc0",
            204 => "PutLoc1",
            205 => "PutLoc2",
            206 => "PutLoc3",
            207 => "SetLoc0",
            208 => "SetLoc1",
            209 => "SetLoc2",
            210 => "SetLoc3",
            211 => "GetArg0",
            212 => "GetArg1",
            213 => "GetArg2",
            214 => "GetArg3",
            215 => "PutArg0",
            216 => "PutArg1",
            217 => "PutArg2",
            218 => "PutArg3",
            219 => "SetArg0",
            220 => "SetArg1",
            221 => "SetArg2",
            222 => "SetArg3",
            223 => "GetVarRef0",
            224 => "GetVarRef1",
            225 => "GetVarRef2",
            226 => "GetVarRef3",
            227 => "PutVarRef0",
            228 => "PutVarRef1",
            229 => "PutVarRef2",
            230 => "PutVarRef3",
            231 => "SetVarRef0",
            232 => "SetVarRef1",
            233 => "SetVarRef2",
            234 => "SetVarRef3",
            235 => "GetLength",
            236 => "IfFalse8",
            237 => "IfTrue8",
            238 => "GoTo8",
            239 => "GoTo16",
            240 => "Call0",
            241 => "Call1",
            242 => "Call2",
            243 => "Call3",
            244 => "IsUndefined",
            245 => "IsNull",
            246 => "TypeOfIsUndefined",
            247 => "TypeOfIsFunction",
            _ => "Unknown",
        }
        .to_string()
    }

    /// returns the canonical name of the opcode, using the given js module definitions.
    pub fn report(&self, pc: u32, fn_idx: u32, js_module: &JsModule) -> String {
        use Opcode::*;
        format!(
            "{}: {}",
            pc,
            match self {
                FClosure { index } => js_module
                    .get_fn_name(*index + fn_idx + 1)
                    .map_or(format!("{:?}", self), |name| {
                        format!("FClosure {{ {} }}", name)
                    }),
                PushAtomValue { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PushAtomValue {{ {} }}", name)
                    }),
                PrivateSymbol { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PrivateSymbol {{ {} }}", name)
                    }),
                ThrowError { ty, atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("ThrowError {{ ty: {} {} }}", ty, name)
                    }),
                CheckVar { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("CheckVar {{ {} }}", name)
                    }),
                GetVarUndef { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetVarUndef {{ {} }}", name)
                    }),
                GetVar { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetVar {{ {} }}", name)
                    }),
                PutVar { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutVar {{ {} }}", name)
                    }),
                PutVarInit { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutVarInit {{ {} }}", name)
                    }),
                PutVarStrict { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutVarStrict {{ {} }}", name)
                    }),
                DefineVar { flags, atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("DefineVar {{ flags: {} {} }}", flags, name)
                    }),
                CheckDefineVar { flags, atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("CheckDefineVar {{ flags: {} {} }}", flags, name)
                    }),
                DefineFunc { flags, atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("DefineFunc {{ flags: {} {} }}", flags, name)
                    }),
                GetField { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetField {{ {} }}", name)
                    }),
                GetField2 { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetField2 {{ {} }}", name)
                    }),
                PutField { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutField {{ {} }}", name)
                    }),
                DefineField { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("DefineField {{ {} }}", name)
                    }),
                SetName { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("SetName {{ {} }}", name)
                    }),
                DefineMethod { atom, flags } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("DefineMethod {{ {} {} }}", name, flags)
                    }),
                DefineMethodComputed { flags } => format!("DefineMethodComputed {{ {} }}", flags),
                DefineClass { flags, atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("DefineClass {{ flags: {} {} }}", flags, name)
                    }),
                DefineClassComputed { flags, atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("DefineClassComputed {{ flags: {} {} }}", flags, name)
                    }),
                GetLoc { index } => js_module
                    .get_fn_loc_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetLoc {{ {} }}", name)
                    }),
                PutLoc { index } => js_module
                    .get_fn_loc_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutLoc {{ {} }}", name)
                    }),
                SetLoc { index } => js_module
                    .get_fn_loc_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("SetLoc {{ {} }}", name)
                    }),
                GetArg { index } => js_module
                    .get_fn_arg_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetArg {{ {} }}", name)
                    }),
                PutArg { index } => js_module
                    .get_fn_arg_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutArg {{ {} }}", name)
                    }),
                SetArg { index } => js_module
                    .get_fn_arg_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("SetArg {{ {} }}", name)
                    }),
                GetVarRef { index } => js_module
                    .get_fn_closure_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetVarRef {{ {} }}", name)
                    }),
                PutVarRef { index } => js_module
                    .get_fn_closure_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutVarRef {{ {} }}", name)
                    }),
                SetVarRef { index } => js_module
                    .get_fn_closure_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("SetVarRef {{ {} }}", name)
                    }),
                SetLocUninit { index } => js_module
                    .get_fn_loc_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("SetLocUninit {{ {} }}", name)
                    }),
                GetLocCheck { index } => js_module
                    .get_fn_loc_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetLocCheck {{ {} }}", name)
                    }),
                PutLocCheck { index } => js_module
                    .get_fn_loc_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutLocCheck {{ {} }}", name)
                    }),
                PutLocCheckInit { index } => js_module
                    .get_fn_loc_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutLocCheckInit {{ {} }}", name)
                    }),
                GetLocCheckThis { index } => js_module
                    .get_fn_loc_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetLocCheckThis {{ {} }}", name)
                    }),
                GetVarRefCheck { index } => js_module
                    .get_fn_closure_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetVarRefCheck {{ {} }}", name)
                    }),
                PutVarRefCheck { index } => js_module
                    .get_fn_closure_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutVarRefCheck {{ {} }}", name)
                    }),
                PutVarRefCheckInit { index } => js_module
                    .get_fn_closure_name(fn_idx, *index)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutVarRefCheckInit {{ {} }}", name)
                    }),
                WithGetVar {
                    atom,
                    diff,
                    is_with,
                } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!(
                            "WithGetVar {{ {} diff: {} is_with: {} }}",
                            name, diff, is_with
                        )
                    }),
                WithPutVar {
                    atom,
                    diff,
                    is_with,
                } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!(
                            "WithPutVar {{ {} diff: {} is_with: {} }}",
                            name, diff, is_with
                        )
                    }),
                WithDeleteVar {
                    atom,
                    diff,
                    is_with,
                } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!(
                            "WithDeleteVar {{ {} diff: {} is_with: {} }}",
                            name, diff, is_with
                        )
                    }),
                WithMakeRef {
                    atom,
                    diff,
                    is_with,
                } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!(
                            "WithMakeRef {{ {} diff: {} is_with: {} }}",
                            name, diff, is_with
                        )
                    }),
                WithGetRef {
                    atom,
                    diff,
                    is_with,
                } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!(
                            "WithGetRef {{ {} diff: {} is_with: {} }}",
                            name, diff, is_with
                        )
                    }),
                WithGetRefUndef {
                    atom,
                    diff,
                    is_with,
                } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!(
                            "WithGetRefUndef {{ {} diff: {} is_with: {} }}",
                            name, diff, is_with
                        )
                    }),
                MakeLocRef { atom, idx } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("MakeLocRef {{ {} idx: {} }}", name, idx)
                    }),
                MakeArgRef { atom, idx } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("MakeArgRef {{ {} idx: {} }}", name, idx)
                    }),
                MakeVarRefRef { atom, idx } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("MakeVarRefRef {{ {} idx: {} }}", name, idx)
                    }),
                MakeVarRef { atom } => js_module
                    .get_atom_name(*atom)
                    .map_or(format!("{:?}", self), |name| {
                        format!("MakeVarRef {{ {} }}", name)
                    }),
                DecLoc { index } => js_module
                    .get_fn_loc_name(fn_idx, *index as u16)
                    .map_or(format!("{:?}", self), |name| {
                        format!("DecLoc {{ {} }}", name)
                    }),
                IncLoc { index } => js_module
                    .get_fn_loc_name(fn_idx, *index as u16)
                    .map_or(format!("{:?}", self), |name| {
                        format!("IncLoc {{ {} }}", name)
                    }),
                AddLoc { index } => js_module
                    .get_fn_loc_name(fn_idx, *index as u16)
                    .map_or(format!("{:?}", self), |name| {
                        format!("AddLoc {{ {} }}", name)
                    }),
                FClosure8 { index } => js_module
                    .get_fn_name(*index as u32 + fn_idx + 1)
                    .map_or(format!("{:?}", self), |name| {
                        format!("FClosure8 {{ {} }}", name)
                    }),
                GetLoc8 { index } => js_module
                    .get_fn_loc_name(fn_idx, *index as u16)
                    .map_or(format!("{:?}", self), |name| {
                        format!("GetLoc8 {{ {} }}", name)
                    }),
                PutLoc8 { index } => js_module
                    .get_fn_loc_name(fn_idx, *index as u16)
                    .map_or(format!("{:?}", self), |name| {
                        format!("PutLoc8 {{ {} }}", name)
                    }),
                SetLoc8 { index } => js_module
                    .get_fn_loc_name(fn_idx, *index as u16)
                    .map_or(format!("{:?}", self), |name| {
                        format!("SetLoc8 {{ {} }}", name)
                    }),
                GetLoc0 | GetLoc1 | GetLoc2 | GetLoc3 => {
                    let index = self.discriminant() - 199 as u8;
                    js_module
                        .get_fn_loc_name(fn_idx, index as u16)
                        .map_or(format!("{:?}", self), |name| {
                            format!("GetLoc{} {{ {} }}", index, name)
                        })
                }
                PutLoc0 | PutLoc1 | PutLoc2 | PutLoc3 => {
                    let index = self.discriminant() - 203 as u8;
                    js_module
                        .get_fn_loc_name(fn_idx, index as u16)
                        .map_or(format!("{:?}", self), |name| {
                            format!("PutLoc{} {{ {} }}", index, name)
                        })
                }
                SetLoc0 | SetLoc1 | SetLoc2 | SetLoc3 => {
                    let index = self.discriminant() - 207 as u8;
                    js_module
                        .get_fn_loc_name(fn_idx, index as u16)
                        .map_or(format!("{:?}", self), |name| {
                            format!("SetLoc{} {{ {} }}", index, name)
                        })
                }
                GetArg0 | GetArg1 | GetArg2 | GetArg3 => {
                    let index = self.discriminant() - 211 as u8;
                    js_module
                        .get_fn_arg_name(fn_idx, index as u16)
                        .map_or(format!("{:?}", self), |name| {
                            format!("GetArg{} {{ {} }}", index, name)
                        })
                }
                PutArg0 | PutArg1 | PutArg2 | PutArg3 => {
                    let index = self.discriminant() - 215 as u8;
                    js_module
                        .get_fn_arg_name(fn_idx, index as u16)
                        .map_or(format!("{:?}", self), |name| {
                            format!("PutArg{} {{ {} }}", index, name)
                        })
                }
                SetArg0 | SetArg1 | SetArg2 | SetArg3 => {
                    let index = self.discriminant() - 219 as u8;
                    js_module
                        .get_fn_arg_name(fn_idx, index as u16)
                        .map_or(format!("{:?}", self), |name| {
                            format!("SetArg{} {{ {} }}", index, name)
                        })
                }
                GetVarRef0 | GetVarRef1 | GetVarRef2 | GetVarRef3 => {
                    let index = self.discriminant() - 223 as u8;
                    js_module
                        .get_fn_closure_name(fn_idx, index as u16)
                        .map_or(format!("{:?}", self), |name| {
                            format!("GetVarRef{} {{ {} }}", index, name)
                        })
                }
                PutVarRef0 | PutVarRef1 | PutVarRef2 | PutVarRef3 => {
                    let index = self.discriminant() - 227 as u8;
                    js_module
                        .get_fn_closure_name(fn_idx, index as u16)
                        .map_or(format!("{:?}", self), |name| {
                            format!("PutVarRef{} {{ {} }}", index, name)
                        })
                }
                SetVarRef0 | SetVarRef1 | SetVarRef2 | SetVarRef3 => {
                    let index = self.discriminant() - 231 as u8;
                    js_module
                        .get_fn_closure_name(fn_idx, index as u16)
                        .map_or(format!("{:?}", self), |name| {
                            format!("SetVarRef{} {{ {} }}", index, name)
                        })
                }
                _ => format!("{:?}", self),
            }
        )
    }

    pub fn discriminant(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

fn variant_name<T>(_: &T) -> &'static str {
    let full_name = type_name::<T>();
    full_name.split("::").last().unwrap()
}
