/// A QuickJS operator code.
#[derive(Debug, Clone)]
pub enum Opcode {
    /// A marker, never emitted.
    Invalid,
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
        val: u32,
    },
    /// Push a private symbol from an atom immediate.
    PrivateSymbol {
        /// The immediate value of the atom.
        val: u32,
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
    GetVar {
        atom: u32,
    },
    GetVarUndef {
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
        index: u16,
    },
    PutLoc {
        index: u16,
    },
    SetLoc {
        index: u16,
    },
    GetArg {
        index: u16,
    },
    PutArg {
        index: u16,
    },
    SetArg {
        index: u16,
    },
    GetVarRef {
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
        index: u16,
    },
    IfFalse {
        offset: u32,
    },
    IfTrue {
        offset: u32,
    },
    GoTo {
        offset: u32,
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
        offset: u8,
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
        alternate_offset: u8,
    },
    IfTrue8 {
        offset: u8,
    },
    GoTo8 {
        offset: u8,
    },
    GoTo16 {
        offset: u16,
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
