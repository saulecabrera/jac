use javy_plugin_api::{
    Config, import_namespace,
    javy::{
        Runtime,
        quickjs::{Ctx, Value},
    },
};

use std::ptr::NonNull;

wit_bindgen::generate!({ world: "jacrt", generate_all });

fn config() -> Config {
    Config::default()
}

fn modify_runtime(runtime: Runtime) -> Runtime {
    runtime
}

struct Component;

impl Guest for Component {
    fn closure(
        context: u32,
        name: String,
        id: u32,
        argc: u32,
        data_ptr: u32,
        data_len: u32,
    ) -> u64 {
        let _ = context;
        let _ = name;
        let _ = id;
        let _ = argc;
        let _ = data_ptr;
        let _ = data_len;
        unsafe {
            let v = Value::new_undefined(Ctx::from_raw(NonNull::new(context as _).unwrap()));
            v.as_raw() as u64
        }
    }

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
