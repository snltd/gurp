use camino::Utf8PathBuf;
use common::types::ApplyOpts;
use common::types::ExitCode;
use janet_int::{helpers, reader};
use janetrs::env::CFunOptions;

pub fn run(host_file: &Utf8PathBuf, format: Option<&str>, opts: &ApplyOpts) -> ExitCode {
    if let Ok(mut config) = reader::assembled_config(host_file, opts) {
        let mut client = helpers::janet_client();

        if let Some(format) = format {
            match format {
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
                    client.add_c_fn(CFunOptions::new(c"encode", helpers::encode_c));
                    tracing::debug!("injecting json print");
                    config.push_str("\n(print (encode (machine-config)))");
                }
                _ => {
                    tracing::error!("format must be 'janet' or 'json'");
                    return 1;
                }
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
