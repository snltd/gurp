use common::types::ExitCode;
use embed::helpers;
use terminal_size::{Width as TermWidth, terminal_size};

pub fn run(resource_type: &str) -> ExitCode {
    let term_width: usize = terminal_size()
        .map(|(TermWidth(w), _)| w as usize)
        .unwrap_or(80);

    helpers::run_command_and_exit(&indoc::formatdoc! { r#"
        (setdyn :term-width {term_width})
        (print (help-for "{resource_type}"))
        "#
    })
}
