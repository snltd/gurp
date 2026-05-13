//! Functions which load user config

use camino::Utf8Path;
use common::info;
use common::types::{ApplyClientOpts, ApplyOpts, CompileError, JsonConfig};
use embed::compiler;
use std::{env, fs};
use util::info as util_info;
use util::{http, json};

/// Get content from either a remote server or a local file. Used for precompiled JSON
/// and jimages
pub(crate) fn load(
    path: Option<&Utf8Path>,
    client_opts: &ApplyClientOpts,
    format: &str,
) -> Result<Vec<u8>, CompileError> {
    if let Some(path) = path {
        if !path.exists() {
            return Err(CompileError::FileNotFound(path.to_owned()));
        }
        tracing::debug!("reading {format} config from {path}");
        fs::read(path).map_err(CompileError::Io)
    } else if let Some(server) = &client_opts.server {
        tracing::debug!("requesting JSON config from {server}");
        http::config_from_server(server, &client_hostname(client_opts)?, format)
    } else {
        Err(CompileError::Other(anyhow::anyhow!(
            "no precompiled JSON path or server config"
        )))
    }
}

/// Turn a snippet supplied with --exec into runnable config
pub(crate) fn from_snippet(snippet: &str) -> Result<String, CompileError> {
    let cwd = env::current_dir()
        .map_err(CompileError::Io)?
        .to_string_lossy()
        .to_string();

    Ok(indoc::formatdoc! { r#"
        (setdyn *syspath* "{cwd}")
        (setdyn :gurp-config-root "{cwd}")

        (host "gurp-runner"
            {snippet})

        (to-json (machine-config))"#})
}

fn client_hostname(client_opts: &ApplyClientOpts) -> Result<String, CompileError> {
    if let Some(user_set_hostname) = &client_opts.hostname {
        Ok(user_set_hostname.clone())
    } else {
        util_info::my_hostname().map_err(CompileError::Other)
    }
}

pub(crate) fn compile(
    path: Option<&Utf8Path>,
    opts: &ApplyOpts,
) -> Result<JsonConfig, CompileError> {
    let json_config = if opts.precompiled {
        let raw = load(path, &opts.client, "json")?;
        String::from_utf8(raw).map_err(|e| CompileError::Other(e.into()))
    } else {
        let mut json_compiler =
            compiler::ConfigCompiler::new(&opts.vm, opts.destroy, opts.output.clone())?;

        if let Some(path) = path {
            json_compiler.janet_file(path, true)
        } else if opts.image {
            let raw = load(path, &opts.client, "jimage")?;
            json_compiler.janet_image(&raw, opts.client.server.as_deref())
        } else if let Some(snippet) = &opts.exec {
            json_compiler.janet_snippet(&from_snippet(snippet)?)
        } else {
            Err(CompileError::Other(anyhow::anyhow!(
                "fell through all apply config options"
            )))
        }
    }?;

    // Now we have a JSON config

    if opts.output.dump_configs {
        let formatted_json = json::pretty(&json_config).map_err(CompileError::Other)?;

        println!(
            "{}",
            info::dump_config(
                &formatted_json,
                Some("Compiled (JSON) config"),
                &opts.output
            )
        );
    }

    Ok(json_config)
}
