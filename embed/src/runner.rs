use super::client;
use common::types::ApplyVmOpts;
use std::process::ExitCode;

pub fn run_command_and_exit(janet_command: &str, vm_opts: &ApplyVmOpts) -> ExitCode {
    match client::gurp(vm_opts, false) {
        Ok(client) => match client.run(janet_command) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                tracing::error!("Janet execution error: {e:#}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            tracing::error!("could not create gurp-specific Janet client: {e:#}");
            ExitCode::FAILURE
        }
    }
}
