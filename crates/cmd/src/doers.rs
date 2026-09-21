use camino::Utf8PathBuf;
use common::types::{ApplyVmOpts, GlobalOpts};
use embed::runner;
use gurptel::init;
use std::io::IsTerminal;
use std::process::ExitCode;

pub fn execute_janet_info_cmd(fn_name: &str, no_colour: bool) -> ExitCode {
    let _ = init::init_telemetry("gurp", &GlobalOpts::default());
    let vm_opts = ApplyVmOpts::from_dir(&Utf8PathBuf::from("/")).expect("can't init vm_opts");

    let run_cmd = if no_colour || !std::io::stdout().is_terminal() {
        format!("(print (strip-ansi ({fn_name})))")
    } else {
        format!("(print ({fn_name}))")
    };

    runner::run_command_and_exit(&run_cmd, &vm_opts)
}

pub fn run(no_colour: bool) -> ExitCode {
    execute_janet_info_cmd("list-doers", no_colour)
}
