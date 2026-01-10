use common::types::ExitCode;
use embed::helpers as janet_helpers;

pub fn run(resource_type: &str) -> ExitCode {
    let client = match janet_helpers::gurp_client() {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("could not create gurp-specific Janet client: {e}");
            return 1;
        }
    };

    let janet_instruction = format!("(print (help-for \"{resource_type}\"))");

    match client.run(janet_instruction) {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("Janet execution error: {}", e);
            1
        }
    }
}
