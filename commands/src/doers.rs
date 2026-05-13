use common::types::ApplyVmOpts;
use embed::runner;
use std::io::IsTerminal;
use std::process::ExitCode;

pub fn run(no_colour: bool) -> ExitCode {
    let vm_opts = ApplyVmOpts::default();

    if no_colour || !std::io::stdout().is_terminal() {
        runner::run_command_and_exit("(print (strip-ansi (list-doers)))", &vm_opts)
    } else {
        runner::run_command_and_exit("(print (list-doers))", &vm_opts)
    }
}
