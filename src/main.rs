#![allow(dead_code, unused_imports, unused_must_use)]

extern crate libc;

use drop::{quickjs_sys::resolver, quickjs_sys::transpiler, Context, Runtime, *};
use once_cell::sync::Lazy;
use std::{ffi::CString, sync::Mutex};

fn args_parse() -> (String, Vec<String>) {
    use argparse::ArgumentParser;
    let mut file_path = String::new();
    let mut rest_args: Vec<String> = vec![];
    {
        let mut arg_parser = ArgumentParser::new();
        arg_parser
            .refer(&mut file_path)
            .add_argument(
                "file",
                argparse::Store,
                "input script (*.[cm][ts|js][x] or *.zrc)",
            )
            .required();
        arg_parser.refer(&mut rest_args).add_argument(
            "args",
            argparse::List,
            "additional arguments for runtime",
        );
        arg_parser.parse_args_or_exit();
    }
    (file_path, rest_args)
}

static mut RT: Lazy<Mutex<Runtime>> = Lazy::new(|| {
    let rt = Runtime::new();
    Mutex::new(rt)
});

static mut CTX: Lazy<Mutex<Context>> = Lazy::new(|| unsafe {
    let mut rt = RT.lock().unwrap();
    let ctx = rt.new_context();
    Mutex::new(ctx)
});

extern "C" fn exit() {
    unsafe {
        let mut ctx = CTX.lock().unwrap();
        ctx.js_loop().unwrap();
        // TODO: this needs to check for errors and propagate the exit code
        ctx.eval_global_str("import('process').then(p=>p.emit('beforeExit'))".into());
        ctx.js_loop().unwrap();
    }
}

fn main() {
    unsafe {
        let mut ctx = CTX.lock().unwrap();
        let (file_path, mut rest_arg) = args_parse();
        let entrypoint =
            resolver::import(&file_path).expect(format!("file not found: {}", &file_path).as_str());
        let code = String::from_utf8(entrypoint)
            .expect(format!("invalid format: {}", &file_path).as_str());
        rest_arg.insert(0, file_path.clone());
        ctx.put_args(rest_arg);
        ctx.eval_global_str(include_str!("./main.js").into());
        ctx.promise_loop_poll();
        ctx.eval_global_str(format!("globalThis.__filename = '{}'", &file_path).into());
        ctx.eval_module_str(code, &file_path);
        ctx.js_loop().unwrap();
        libc::atexit(exit);
    }
}
