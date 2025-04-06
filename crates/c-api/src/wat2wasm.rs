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

use wasmtime_explorer;

#[unsafe(no_mangle)]
pub extern "C" fn wasmtime_explore(c: &mut wasm_config_t) {
    self.common.init_logging()?;

    let mut config = self.common.config(None)?;

    let bytes =
        Cow::Owned(std::fs::read(&self.module).with_context(|| {
            format!("failed to read Wasm module: {}", self.module.display())
        })?);
    #[cfg(feature = "wat")]
    let bytes = wat::parse_bytes(&bytes).map_err(|mut e| {
        e.set_path(&self.module);
        e
    })?;

    let output = self
        .output
        .clone()
        .unwrap_or_else(|| self.module.with_extension("explore.html"));
    let output_file = std::fs::File::create(&output)
        .with_context(|| format!("failed to create file: {}", output.display()))?;
    let mut output_file = std::io::BufWriter::new(output_file);

    let clif_dir = if let Some(Strategy::Cranelift) | None = self.common.codegen.compiler {
        let clif_dir = tempdir()?;
        config.emit_clif(clif_dir.path());
        config.disable_cache(); // cache does not emit clif
        Some(clif_dir)
    } else {
        None
    };

    wasmtime_explorer::generate(
        &config,
        self.common.target.as_deref(),
        clif_dir.as_ref().map(|tmp_dir| tmp_dir.path()),
        &bytes,
        &mut output_file,
    )?;

    println!("Exploration written to {}", output.display());
}