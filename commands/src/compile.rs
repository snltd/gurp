use camino::Utf8PathBuf;
use common::types::ApplyOpts;
use common::types::ExitCode;
use janet_int::helpers as janet_helpers;
use janet_int::reader;

pub fn run(host_file: &Utf8PathBuf, opts: &ApplyOpts) -> ExitCode {
    let host_config = match reader::read_and_enrich_host_config(host_file, opts) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("reader error: {}", e);
            return 1;
        }
    };

    let client = janet_helpers::janet_client();

    match client.run(host_config) {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("Janet execution error: {}", e);
            1
        }
    }
}
