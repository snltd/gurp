use embed::runner;
use std::io::IsTerminal;
use std::process::ExitCode;

pub fn run(no_colour: bool) -> ExitCode {
    if no_colour || !std::io::stdout().is_terminal() {
        runner::run_command_and_exit("(print (strip-ansi (list-doers)))")
    } else {
        runner::run_command_and_exit("(print (list-doers))")
    }
}
