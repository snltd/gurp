use embed::runner;
use std::process::ExitCode;

pub fn run() -> ExitCode {
    runner::run_command_and_exit("(repl)")
}
