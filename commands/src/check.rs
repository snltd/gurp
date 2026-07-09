use anyhow::Context;
use camino::Utf8Path;
use common::types::ApplyVmOpts;
use embed::client;
use std::process::ExitCode;

pub fn run(path: &Utf8Path) -> ExitCode {
    match check_config(path) {
        Ok(_) => {
            println!("checked successfully");
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("check errored: {e}");
            ExitCode::FAILURE
        }
    }
}

fn check_config(path: &Utf8Path) -> anyhow::Result<bool> {
    let path = path
        .canonicalize_utf8()
        .with_context(|| format!("cannot canonicalize path for {path}"))?;

    let parent = path
        .parent()
        .with_context(|| format!("cannot get parent of {path}"))?;

    let client = client::gurp(&ApplyVmOpts::default(), false)?;

    // flycheck always returns nil, so the easiest thing to do is pass the
    // exit flag and let the interpreter exit for us
    let janet_command = indoc::formatdoc! {
            r#"
            (setdyn *syspath* "{parent}")
            (flycheck "{path}" :exit true)
            "#
    };

    client.run(janet_command)?;

    Ok(true)
}
