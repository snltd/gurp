use common::types::ApplyVmOpts;
use embed::runner;
use std::process::ExitCode;

pub fn run(opts: &ApplyVmOpts) -> ExitCode {
    runner::run_command_and_exit("(repl)", opts)
}
