use camino::Utf8PathBuf;
use common::types::ExitCode;
use common::types::{ApplyOpts, CompileOpts};
use embed::{helpers, reader};
use janetrs::env::CFunOptions;
use std::fs;

pub fn run(host_file: &Utf8PathBuf, c_opts: &CompileOpts, opts: &ApplyOpts) -> ExitCode {
    if c_opts.format == "jimage" {
        if let Some(path) = &c_opts.output_file {
            match helpers::compile_to_image(host_file, opts) {
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
                    3
                }
            }
        } else {
            tracing::error!("writing an image requires an output path");
            2
        }
    } else if let Ok(mut config) = reader::assembled_config(host_file, opts) {
        let mut client = helpers::janet_client();
        match c_opts.format.as_str() {
            "janet" => {
                if opts.colour {
                    tracing::debug!("injecting colour prinf");
                    config.push_str("\n(prinf \"%M\" (machine-config))");
                } else {
                    tracing::debug!("injecting non-colour prinf");
                    config.push_str("\n(prinf \"%m\" (machine-config))");
                }
            }
            "json" => {
                client.add_c_fn(CFunOptions::new(c"encode-to-json", helpers::encode_c));
                tracing::debug!("injecting json print");
                config.push_str("\n(print (encode-to-json (machine-config)))");
            }
            _ => {
                tracing::error!("format must be 'janet', 'jimage', or 'json'");
                return 1;
            }
        }

        match client.run(config) {
            Ok(_) => 0,
            Err(e) => {
                tracing::error!("Janet execution error: {e}");
                1
            }
        }
    } else {
        tracing::error!("Could not load config {host_file}");
        1
    }
}
