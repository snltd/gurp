use camino::Utf8PathBuf;
use common::types::{ApplyOpts, CompileOpts, ExitCode};
use embed::compiler;
use std::fs;

pub fn run(host_file: &Utf8PathBuf, c_opts: &CompileOpts, opts: &ApplyOpts) -> ExitCode {
    match c_opts.format.as_str() {
        "json" => match compiler::local_janet_to_jason(host_file, opts) {
            Ok(json) => {
                if let Some(out_file) = &c_opts.output_file {
                    match fs::write(out_file, json) {
                        Ok(_) => {
                            tracing::info!("wrote JSON to {out_file}");
                            0
                        }
                        Err(e) => {
                            tracing::error!("error writing JSON to {out_file}: {e}");
                            2
                        }
                    }
                } else {
                    println!("{json}");
                    0
                }
            }
            Err(e) => {
                tracing::error!("error compiling janet->JSON: {e}");
                1
            }
        },
        "jimage" => {
            if let Some(path) = &c_opts.output_file {
                match compiler::local_janet_to_jimage(host_file, opts) {
                    Ok(image_data) => match fs::write(path, image_data) {
                        Ok(_) => {
                            tracing::info!("wrote image file to '{path}'");
                            0
                        }
                        Err(e) => {
                            tracing::error!("error writing image file: {e}");
                            4
                        }
                    },
                    Err(e) => {
                        tracing::error!("error compiling image file: {e}");
                        5
                    }
                }
            } else {
                tracing::error!("writing an image requires an output path");
                2
            }
        }
        _ => {
            tracing::error!("format must be json or jimage");
            1
        }
    }
}
