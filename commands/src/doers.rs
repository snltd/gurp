use common::types::ExitCode;
use embed::helpers as janet_helpers;

pub fn run() -> ExitCode {
    let client = match janet_helpers::gurp_client() {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("could not create gurp-specific Janet client: {e}");
            return 1;
        }
    };

    match client.run("(print (list-doers))") {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("Janet execution error: {}", e);
            1
        }
    }
}
