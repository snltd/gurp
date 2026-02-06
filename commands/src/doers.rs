use embed::helpers;
use std::io::IsTerminal;
use std::process::ExitCode;

pub fn run(no_colour: bool) -> ExitCode {
    if no_colour || !std::io::stdout().is_terminal() {
        helpers::run_command_and_exit("(print (strip-ansi (list-doers)))")
    } else {
        helpers::run_command_and_exit("(print (list-doers))")
    }
}
