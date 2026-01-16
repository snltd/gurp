use common::types::ExitCode;
use embed::helpers;

pub fn run(resource_type: &str) -> ExitCode {
    helpers::run_command_and_exit(&format!("(print (help-for \"{resource_type}\"))"))
}
