use crate::{client, janet_cfuncs};
use anyhow::{Context, bail};
use janetrs::TaggedJanet;
use janetrs::env::CFunOptions;
use std::process::ExitCode;

pub fn run_command_and_exit(janet_command: &str) -> ExitCode {
    match client::gurp() {
        Ok(client) => match client.run(janet_command) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                tracing::error!("Janet execution error: {e:#}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            tracing::error!("could not create gurp-specific Janet client: {e:#}");
            ExitCode::FAILURE
        }
    }
}

pub fn run_config(host_config: &str) -> anyhow::Result<String> {
    let mut client = client::vanilla();
    client.add_c_fn(CFunOptions::new(c"to_json", janet_cfuncs::to_json_c));
    let json_wrapped_host_config = format!("{host_config}\n(to-json (machine-config))");
    let json_config = client
        .run(json_wrapped_host_config)
        .context("failed to run config")?;

    let json = match json_config.unwrap() {
        TaggedJanet::String(buf) => buf.to_string(),
        other => bail!("expected JSON config as Janet::String; got {}", other),
    };

    Ok(json)
}
