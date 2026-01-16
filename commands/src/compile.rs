use camino::Utf8PathBuf;
use common::types::{ApplyOpts, CompileOpts, ExitCode};
use embed::compiler;
use std::fs;

pub fn run(host_file: &Utf8PathBuf, c_opts: &CompileOpts, opts: &ApplyOpts) -> ExitCode {
    match c_opts.format.as_str() {
        "json" => compile_json(host_file, c_opts, opts),
        "jimage" => compile_jimage(host_file, c_opts, opts),
        _ => {
            tracing::error!("format must be json or jimage");
            1
        }
    }
}

fn compile_json(host_file: &Utf8PathBuf, c_opts: &CompileOpts, opts: &ApplyOpts) -> ExitCode {
    let json = match compiler::local_janet_to_json(host_file, opts) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("error compiling janet->JSON: {e}");
            return 1;
        }
    };

    if let Some(out_file) = &c_opts.output_file {
        if let Err(e) = fs::write(out_file, json) {
            tracing::error!("error writing JSON to {out_file}: {e}");
            return 2;
        }
        tracing::info!("wrote JSON to {out_file}");
    } else {
        println!("{json}");
    }

    0
}

fn compile_jimage(host_file: &Utf8PathBuf, c_opts: &CompileOpts, opts: &ApplyOpts) -> ExitCode {
    let output_path = match &c_opts.output_file {
        Some(path) => path,
        None => {
            tracing::error!("writing an image requires an output path");
            return 2;
        }
    };

    let image_data = match compiler::local_janet_to_jimage(host_file, opts) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("error compiling image file: {e}");
            return 5;
        }
    };

    if let Err(e) = fs::write(output_path, image_data) {
        tracing::error!("error writing image file: {e}");
        return 4;
    }

    tracing::info!("wrote image file to '{output_path}'");
    0
}
