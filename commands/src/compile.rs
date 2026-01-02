use camino::Utf8PathBuf;
use common::types::ExitCode;
use common::types::{ApplyOpts, CompileOpts};
use janet_int::{helpers, reader};
use janetrs::env::CFunOptions;
use std::fs;

pub fn run(host_file: &Utf8PathBuf, c_opts: &CompileOpts, opts: &ApplyOpts) -> ExitCode {
    if let Ok(mut config) = reader::assembled_config(host_file, opts) {
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
            "jimage" => {
                if let Some(path) = &c_opts.output_file {
                    // client.add_c_fn(CFunOptions::new(c"encode-to-jimage", helpers::encode_c));
                    // tracing::debug!("injecting jimage");
                    fs::write("/tmp/x.janet", &config)
                        .expect("Should be able to write to `/foo/tmp`");

                    // let builder_config = indoc::formatdoc! { "
                    // (-string ``````
                    // {config}
                    // ``````)
                    // (spit \"{path}\" (make-image (curenv)))
                    // "
                    // };

                    config = format!(
                        "(merge-module (curenv)  (dofile \"/tmp/x.janet\") \"\" true)\n
                        (spit \"{path}\" (make-image (curenv)))"
                    );
                } else {
                    tracing::error!("writing an image requires an output path");
                    return 2;
                }
            }
            _ => {
                tracing::error!("format must be 'janet', 'jimage', or 'json'");
                return 1;
            }
        }

        println!("{config}");

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
