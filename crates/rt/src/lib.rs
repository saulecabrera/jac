// Single-threaded scenario.
#![allow(static_mut_refs)]

use anyhow::anyhow;
use javy_plugin_api::{
    Config, import_namespace,
    javy::{
        Runtime,
        quickjs::{Ctx, Value, qjs},
    },
};

use std::ptr::NonNull;
use std::cell::OnceCell;

wit_bindgen::generate!({ world: "jacrt", generate_all });

static mut RT: OnceCell<Runtime> = OnceCell::new();

fn config() -> Config {
    Config::default()
}

fn modify_runtime(runtime: Runtime) -> Runtime {
    runtime
}


fn unwrap_runtime() -> &'static Runtime {
    unsafe { RT.get().expect("Runtime not initialized") }
}

struct Component;

impl Guest for Component {
    fn init() {
	let runtime = Runtime::default();
	unsafe {
	   RT
		.set(runtime)
		.map_err(|_| anyhow!("Could not pre-initialize javy::Runtime"))
		.unwrap()
	}
    }
    fn closure(
        id: u32,
        argc: u32,
        data_ptr: u32,
        data_len: u32,
    ) -> u64 {

	let rt = unwrap_runtime();
	rt.context()
	    .with(|cx| {
		unsafe {
		    qjs::JS_NewCFunctionData(
			cx.as_raw().as_ptr(),
			Some(closure),
			argc as _,
			id as _,
			data_len as _,
			data_ptr as _,
		    )
		}
	    })
	
    }

    // These methods are not used in the context of `jac` -- they are
    // here simply to satisfy the interface.
    fn invoke(bytecode: Vec<u8>, function: Option<String>) {
        javy_plugin_api::invoke(&bytecode, function.as_deref()).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::abort();
        })
    }

    fn compile_src(src: Vec<u8>) -> Result<Vec<u8>, String> {
        javy_plugin_api::compile_src(&src).map_err(|e| e.to_string())
    }

    fn initialize_runtime() {
        javy_plugin_api::initialize_runtime(config, modify_runtime).unwrap()
    }
}

import_namespace!("jacrt");
export!(Component);

unsafe extern "C" fn closure(
    context: *mut qjs::JSContext,
    _this: qjs::JSValue,
    _argc: i32,
    _argv: *mut qjs::JSValue,
    _magic: i32,
    _data: *mut qjs::JSValue,
) -> qjs::JSValue {
    println!("Invoked!");
    unsafe { Value::new_undefined(Ctx::from_raw(NonNull::new(context as _).unwrap())).as_raw() }
}
