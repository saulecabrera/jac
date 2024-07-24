//! QuickJS bytecode details.

use anyhow::{bail, ensure, Result};

// QuickJS manages several flavors of bytecode version, depending on what
// features the engine is compiled with. For the time being we assume `67`, which
// means that the engine is compiled with BIGNUM support.
pub const VERSION: u8 = 67;

/// Bytecode tags.
///
/// Each tag represents a value or a section in the bytecode.
#[derive(Debug)]
pub enum Tag {
    Null = 1,
    Undefined,
    False,
    True,
    I32,
    F64,
    String,
    Object,
    Array,
    BigInt,
    TemplateObject,
    FunctionBytecode,
    Module,
    TypedArray,
    ArrayBuffer,
    SharedArrayBuffer,
    Date,
    ObjectValue,
    ObjectRef,
    BigFloat,
    BigDecimal,
}

impl Tag {
    /// Maps an arbitrary byte to a [Tag].
    pub fn map_byte(byte: u8) -> Result<Tag> {
        Ok(match byte {
            12 => Tag::FunctionBytecode,
            13 => Tag::Module,
            _ => bail!("Unknown tag: {byte}"),
        })
    }
}

/// Extract the flag at the given index of the bitset.
///
/// #Safety
///
/// `T` should only be a primitive type.
/// TODO: Find a way to restrict this.
pub(crate) fn flag<T>(flags: u32, index: u32) -> u32 {
    let size = std::mem::size_of::<T>();
    (flags >> index) & ((1u32 << size) - 1)
}

/// Validates the bytecode version.
pub(crate) fn validate_version(version: u8) -> Result<u8> {
    let bc_version = VERSION;
    ensure!(
        version == bc_version,
        "Mismatched bytecode version. Found: {version}, expected: {bc_version}"
    );
    Ok(version)
}
