use common::types::ExitCode;
use janet_int::helpers as janet_helpers;

pub fn run(resource_type: &str) -> ExitCode {
    let client = janet_helpers::janet_client();

    let mut janet = janet_int::constants::GURP_DEFAULTS.to_owned();
    janet.push('\n');
    janet.push_str(janet_int::constants::GURP_LIB);
    janet.push_str(&format!("\n(print (help-for \"{resource_type}\"))"));

    match client.run(janet) {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("Janet execution error: {}", e);
            1
        }
    }
}
