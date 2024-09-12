//! JAC - The Javy Ahead-of-Time Compiler.
use anyhow::Result;
use quickpars::{Parser, Payload};

// NOTE:
// * `JS_EvalFunction` -> to evaluate the value as a function. The function
// evaluation, when the `JS_EVAL_FLAG_COMPILE_ONLY` is set, it returns a value
// tagged with `JS_TAG_FUNCTION_BYTECODE`. It also returns the bytecode instead
// of evaluating the function inline.
//
//   Note that when writing the final bytecode object, `JS_TAG_FUNCTION_BYTECOE`
//   gets transformed to `BC_TAG_FUNCTION_BYTECODE`, which is what gets used
//   when reading the bytedode (see below).
//
// * After gathering the bytecode, we use `JS_ReadObject` to read the bytecode
// as `JSValue. This is like the process of instantiation.
//   The sequence (incomplete at least right now), is:
//   * `JS_ReadObjectAtoms`
//   * `JS_ReadObjectRec`
//   * `JS_ReadFunctionTag` (setup locals, etc)
//      * `JS_ReadFunctionBytecode` (setup more atoms?)
//
// * After having "instantiated" the bytecode. We can evaluate a particular
// function by:
//    * using `JS_Eval`, and then `JS_EvalThis` and finally `JS_EvalInternal`,
//      which ends up in `__JS_EvalInternal`
//    * Those functions read flags, and sets things up and then
//    `JS_EvalFunctionInternal` gets evaluated.
//    * Finally `JS_CallInternal` does the bulk of the work.
//

// For the compiler to cooperate with the runtime, multiple things need to
// happen:
//
// 1. At runtime, we still need to go through `JS_ReadObject` to "instantiate"
//    the current object through bytecode. This will intern atoms and prepare
//    a bunch of state.
// 2. At compile time, we don't need to process the entire bytecode: assuming
//    that at runtime we'll go through `JS_ReadObject`, we can "skip" to the
//    function section.
// 3. The driver: with this approach the current dynamically linked module, will
//    be replaced by the code that we'll generate via LLVM. This code will
//    contain some code to decide which functions we need to invoke, depending
//    if we have a compiled version or not present.

/// Main entry point to compile QuickJS bytecode ahead-of-time to WebAssembly.
pub fn compile(bytes: &[u8]) -> Result<Vec<u8>> {
    let module = Parser::parse_buffer_sync(bytes)?;
    println!("{:#?}", &module);

    Ok(vec![])
}
