use camino::Utf8PathBuf;
use common::types::{ApplyVmOpts, GlobalOpts};
use embed::runner;
use gurptel::init;
use std::io::IsTerminal;
use std::process::ExitCode;

pub fn run(no_colour: bool) -> ExitCode {
    let _ = init::init_telemetry("gurp", &GlobalOpts::default());
    let vm_opts = ApplyVmOpts::from_dir(&Utf8PathBuf::from("/")).expect("can't init vm_opts");

    if no_colour || !std::io::stdout().is_terminal() {
        runner::run_command_and_exit("(print (strip-ansi (list-doers)))", &vm_opts)
    } else {
        runner::run_command_and_exit("(print (list-doers))", &vm_opts)
    }
}
