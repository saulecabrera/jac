//! JAC - The Javy Ahead-of-Time Compiler.

// NOTE:
// * `JS_EvalFunction` -> to evaluate the value as a function.
//   The function evaluation, when the `JS_EVAL_FLAG_COMPILE_ONLY` is set, it
//   returns a value tagged with `JS_TAG_FUNCTION_BYTECODE`. It also returns the
//   bytecode instead of evaluating the function inline.
//
// * After gathering the bytecode, we use `JS_ReadObject` to read the bytecode as `JSValue`.
//   The sequence (incomplete at least right now), is:
//   * `JS_ReadObjectAtoms`
//   * `JS_ReadObjectRec`
//   * `JS_ReadFunctionTag` (setup locals, etc)
//      * `JS_ReadFunctionBytecode` (setup more atoms?)

/// Compile QuickJS bytecode ahead-of-time to WebAssembly.
pub fn compile(bytes: &[u8]) -> Vec<u8> {
    vec![]
}
