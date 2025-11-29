use common::types::ExitCode;
use janet_int::helpers as janet_helpers;

pub fn run() -> ExitCode {
    let client = janet_helpers::janet_client();

    let mut janet = janet_int::constants::GURP_DEFAULTS.to_owned();
    janet.push('\n');
    janet.push_str(janet_int::constants::GURP_LIB);
    janet.push_str("(each r (sort (keys resource-ensure-keys)) (print r))");

    match client.run(janet) {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("Janet execution error: {}", e);
            1
        }
    }
}
