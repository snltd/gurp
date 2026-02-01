use embed::helpers;
use std::process::ExitCode;

pub fn run() -> ExitCode {
    helpers::run_command_and_exit("(print (list-doers))")
}
