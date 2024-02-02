//! JAC - The Javy Ahead-of-Time Compiler.

// NOTE:
// * `JS_ReadObject` -> to read bytecode as a value.
//   * `JS_ReadObjectAtoms`
//   * `JS_ReadObjectRec`
// * `JS_EvalFunction` -> to evaluate the value as a function.

/// Compile QuickJS bytecode ahead-of-time to WebAssembly.
pub fn compile(bytes: &[u8]) -> Vec<u8> {
    vec![]
}
