use common::types::ExitCode;
use embed::helpers as janet_helpers;

pub fn run(resource_type: &str) -> ExitCode {
    let client = janet_helpers::janet_client();

    let mut janet = embed::constants::GURP_DEFAULTS.to_owned();
    janet.push('\n');
    janet.push_str(embed::constants::GURP_LIB);
    janet.push_str(&format!("\n(print (help-for \"{resource_type}\"))"));

    match client.run(janet) {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("Janet execution error: {}", e);
            1
        }
    }
}
