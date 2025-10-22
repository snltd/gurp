use camino::Utf8PathBuf;
use common::types::ApplyOpts;
use common::types::ExitCode;
use janet_int::helpers;

pub fn run(host_file: &Utf8PathBuf, format: Option<&str>, opts: &ApplyOpts) -> ExitCode {
    match helpers::compile_config(host_file, format, opts) {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("Janet execution error: {}", e);
            1
        }
    }
}
