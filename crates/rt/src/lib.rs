pub type FnType = extern "C" fn(i32) -> i32;

#[no_mangle]
extern "C" fn call(f: FnType) {
    let result = f(0);
    println!("{}", result);
}
