//! Functions which load user config

use anyhow::Context;
use camino::Utf8Path;
use common::info;
use common::types::{ApplyClientOpts, ApplyOpts, ApplyVmOpts, CompileError, JsonConfig};
use embed::compiler;
use std::fs;
use util::info as util_info;
use util::{file, http, json};

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
    Ok(indoc::formatdoc! { r#"
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
        let config_dir = if let Some(path) = path {
            if !path.exists() {
                return Err(CompileError::FileNotFound(path.to_path_buf()));
            }

            let host_file = path
                .canonicalize_utf8()
                .with_context(|| format!("failed to canonicalize host file at {path}"))
                .map_err(CompileError::Other)?;

            let config_dir = host_file
                .parent()
                .with_context(|| format!("cannot get parent of host file at {path}"))
                .map_err(CompileError::Other)?;

            config_dir.to_owned()
        } else {
            file::current_dir().map_err(CompileError::Other)?
        };

        let syspath = match &opts.syspath {
            Some(path) => path
                .canonicalize_utf8()
                .with_context(|| format!("failed to canonicalize syspath {path}"))
                .map_err(CompileError::Other)?,
            None => config_dir.clone(),
        };

        let gurp_config_root = match &opts.gurp_config_root {
            Some(path) => path
                .canonicalize_utf8()
                .with_context(|| format!("failed to canonicalize gurp_config_root {path}"))
                .map_err(CompileError::Other)?,
            None => config_dir.clone(),
        };

        let vm_opts = ApplyVmOpts {
            syspath,
            gurp_config_root,
            ..opts.vm.clone()
        };

        let mut json_compiler = compiler::ConfigCompiler::new(&vm_opts, opts.output.clone())?;

        if let Some(path) = path
            && !opts.image
        {
            // local Janet config
            json_compiler.janet_file(path, true)
        } else if let Some(snippet) = &opts.exec {
            // local snippet
            json_compiler.janet_snippet(&from_snippet(snippet)?)
        } else {
            // local or remote Janet image
            let raw = load(path, &opts.client, "jimage")?;
            json_compiler.janet_image(&raw, opts.client.server.as_deref(), &opts.vm)
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

#[cfg(test)]
mod test {
    // TODO test failure modes
}
