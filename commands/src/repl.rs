use common::types::ExitCode;
use embed::helpers;

pub fn run() -> ExitCode {
    helpers::run_command_and_exit("(repl)")
}
