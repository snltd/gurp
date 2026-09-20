use super::doers;
use std::process::ExitCode;

pub fn run(no_colour: bool) -> ExitCode {
    doers::execute_janet_info_cmd("list-facts", no_colour)
}
