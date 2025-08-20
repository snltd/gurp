use camino::Utf8PathBuf;
use common::types::ApplyOpts;
use common::types::ExitCode;
use janet_int::helpers as janet_helpers;
use janet_int::reader;
use janetrs::env::CFunOptions;

pub fn run(host_file: &Utf8PathBuf, format: Option<&str>, opts: &ApplyOpts) -> ExitCode {
    let host_config = match reader::read_and_enrich_host_config(host_file, format, opts) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("reader error: {}", e);
            return 1;
        }
    };

    let mut client = janet_helpers::janet_client();

    if let Some(format) = format
        && format == "json"
    {
        client.add_c_fn(CFunOptions::new(c"encode", janet_helpers::encode_c));
    }

    match client.run(host_config) {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("Janet execution error: {}", e);
            1
        }
    }
}
