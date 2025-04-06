use crate::{bad_utf8, handle_result, wasm_byte_vec_t, wasmtime_error_t};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wasmtime_wat2wasm(
    wat: *const u8,
    wat_len: usize,
    ret: &mut wasm_byte_vec_t,
) -> Option<Box<wasmtime_error_t>> {
    let wat = crate::slice_from_raw_parts(wat, wat_len);
    let wat = match std::str::from_utf8(wat) {
        Ok(s) => s,
        Err(_) => return bad_utf8(),
    };
    handle_result(wat::parse_str(wat).map_err(|e| e.into()), |bytes| {
        ret.set_buffer(bytes)
    })
}


use crate::wasm_config_t;
use wasmtime_explorer;
use std::ffi::CStr;
use std::fs;
use std::path::Path;

#[unsafe(no_mangle)]
pub extern "C" fn wasmtime_explore(c: &wasm_config_t, wasm_path: *const std::os::raw::c_char) {

    let config = &c.config;
    
    // path
    if wasm_path.is_null() {
        eprintln!("Null pointer received");
        return;   
    }
    
    let c_str = unsafe { CStr::from_ptr(wasm_path) };
    let path_str = c_str.to_str().expect("failed to to_string");
    let input_path = Path::new(path_str);
    let output_path = format!("{}.explore.html", path_str);
    let output_path = Path::new(&output_path);
    
    let bytes = fs::read(input_path).expect("failed to read path");
    let output_file = std::fs::File::create(&output_path).expect("failed to create output_file");
    let mut output_file = std::io::BufWriter::new(output_file);

    let result = wasmtime_explorer::generate(
        config,
        // self.common.target.as_deref(),
        None,
        None,
        &bytes,
        &mut output_file,
    );

    match result {
        Ok(value) => println!("Exploration written to {}", output_path.display()),
        Err(e) => println!("failed to explore: {}", e),
    }
}