use common::types::ApplyVmOpts;
use embed::runner;
use std::io::IsTerminal;
use std::process::ExitCode;
use terminal_size::{Width as TermWidth, terminal_size};

pub fn run(resource_type: &str, no_colour: bool) -> ExitCode {
    let term_width: usize = terminal_size()
        .map(|(TermWidth(w), _)| w as usize)
        .unwrap_or(80);

    let modifier = if no_colour || !std::io::stdout().is_terminal() {
        "strip-ansi"
    } else {
        "identity"
    };

    runner::run_command_and_exit(
        &indoc::formatdoc! { r#"
        (setdyn :term-width {term_width})
        (print ({modifier} (help-for "{resource_type}")))
        "#
        },
        &ApplyVmOpts::default(),
    )
}
