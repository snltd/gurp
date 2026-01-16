use crate::helpers as janet_helpers;
use anyhow::{Context, bail, ensure};
use camino::Utf8PathBuf;
use colored::Colorize;
use common::constants::SERVER_PORT;
use common::helpers;
use common::prelude::*;
use janetrs::env::DefOptions;
use janetrs::{Janet, JanetString, TaggedJanet};
use serde_json::Error;
use std::fs;
use std::thread;
use std::time::Duration;
use util::http;

// Config comes in various forms. It can be:
//   1. a local Janet config which is compiled to Janet using Gurp's built-in library.
//   2. a local JSON file compiled by this, or some other Gurp instance.
//   3. a JSON file compiled on and fetched from a remote server.
//   4. a local Janet Image file compiled by this, or some other Gurp instance.
//   5. a Janet image compiled on and fetched from a remote server.
//
// In cases 3 and 5 it's the responsibility of this code to fetch the file before applying it.
//
pub fn compile_to_json(
    host_file: Option<&Utf8PathBuf>,
    opts: &ApplyOpts,
) -> anyhow::Result<String> {
    let ret = if opts.image {
        tracing::debug!("applying precompiled jimage config");
        // case 4
        let image_file = host_file.context("No host file specified")?;
        local_jimage_to_json(image_file, opts)?
    } else if opts.precompiled {
        tracing::debug!("applying precompiled JSON config");
        // case 2
        let host_file = host_file.context("No host file specified")?;
        local_json_to_json(host_file)?
    } else if let Some(server) = opts.server.as_ref() {
        if opts.as_json {
            tracing::debug!("applying precompiled JSON config from server");
            // case 3
            remote_json_to_json(server, opts)?
        } else {
            tracing::debug!("applying precompiled jimage config from server");
            // case 5
            remote_jimage_to_json(server, opts)?
        }
    } else if let Some(host_file) = host_file {
        tracing::debug!("compiling and applying local Janet config");
        // case 1
        local_janet_to_json(host_file, opts)?
    } else {
        bail!("No configuration file specified")
    };

    Ok(ret)
}

pub fn display_error(e: Error, json: &str) -> anyhow::Result<()> {
    tracing::error!("deserializing error: {}", e);
    let formatted_json = formatted_json(json)?;
    let error_line = e.line();
    let json_lines: Vec<_> = formatted_json.lines().collect();
    let first_line = error_line.saturating_sub(30);
    let last_line = (error_line + 15).clamp(0, json_lines.len());

    for l in first_line..=last_line {
        let output_line = format!(" {:4} | {}", l + 1, json_lines.get(l).unwrap_or(&""));

        if l == error_line {
            println!("{}", output_line.bold());
        } else {
            println!("{output_line}");
        }
    }

    Ok(())
}

// Get a JSON string from a Janet image on disk
pub fn local_jimage_to_json(path: &Utf8PathBuf, opts: &ApplyOpts) -> anyhow::Result<JsonConfig> {
    ensure!(path.exists(), "Cannot find image file at {}", path);

    jimage_to_json(&fs::read(path)?, None, opts)
}

// Get a JSON string from a Janet image from a remote server
fn remote_jimage_to_json(server: &str, opts: &ApplyOpts) -> anyhow::Result<JsonConfig> {
    let hostname = opts
        .hostname
        .clone()
        .map_or_else(helpers::my_hostname, Ok)?;

    jimage_to_json(
        &fetch_from_server(server, &hostname, "jimage")?,
        Some(server),
        opts,
    )
}

// Get a JSON string from a pre-compiled file on disk
fn local_json_to_json(path: &Utf8PathBuf) -> anyhow::Result<JsonConfig> {
    ensure!(path.exists(), "Cannot find JSON file at {}", path);
    Ok(fs::read_to_string(path)?)
}

// Get a JSON string from a remote server
fn remote_json_to_json(server: &str, opts: &ApplyOpts) -> anyhow::Result<JsonConfig> {
    let hostname = opts
        .hostname
        .clone()
        .map_or_else(helpers::my_hostname, Ok)?;

    let host_config = String::from_utf8(fetch_from_server(server, &hostname, "json")?)?;

    if opts.dump_config {
        let formatted_json = helpers::pretty_json(&host_config)?;

        println!(
            "{}",
            helpers::dump_config(&formatted_json, "Janet config", opts)
        );
    }

    Ok(host_config)
}

// Get a JSON string by compiling a local Janet file (and its dependencies)
pub fn local_janet_to_json(
    host_file: &Utf8PathBuf,
    opts: &ApplyOpts,
) -> anyhow::Result<JsonConfig> {
    ensure!(
        host_file.exists(),
        "Cannot find host config file at {}",
        host_file
    );

    let host_file = host_file.canonicalize_utf8()?;

    let config_dir = host_file
        .parent()
        .context("cannot get parent of config file")?;

    let client = janet_helpers::gurp_client()?;

    let mut janet_instructions = String::new();

    janet_instructions.push_str(&format!("(setdyn *syspath* \"{config_dir}\")\n"));
    janet_instructions.push_str(&format!("(setdyn :gurp-config-root\"{config_dir}\")\n"));

    if opts.destroy {
        janet_instructions.push_str("(setdyn :destroy-everything-you-touch true)\n");
    }

    janet_instructions.push_str(&format!(
        "(merge-module (curenv) (dofile \"{host_file}\" :env (curenv)) \"\" true)\n"
    ));

    janet_instructions.push_str("(to-json (machine-config))\n");

    if opts.dump_config {
        println!(
            "{}",
            helpers::dump_config(&janet_instructions, "Janet config", opts)
        );
    }

    let janet_result = client.run(janet_instructions)?;
    Ok(janet_result.unwrap().to_string())
}

// We tell the server what we think it's called so it can build file resources we can find. This
// lets us use a raw IP address, DNS name, whatever.
fn fetch_precompiled_file(server: &str, hostname: &str, format: &str) -> anyhow::Result<Vec<u8>> {
    let url = format!(
        "http://{server}:{SERVER_PORT}/config/{hostname}?server_name={server}&format={format}"
    );
    tracing::info!("fetching config from {url}");
    http::remote_file_to_memory(&url)
}

pub fn jimage_to_json(
    raw_image: &[u8],
    server: Option<&str>,
    opts: &ApplyOpts,
) -> anyhow::Result<JsonConfig> {
    let mut client = janet_helpers::gurp_client()?;
    let jstr = JanetString::new(raw_image);
    let janet_val = Janet::string(jstr);
    client.add_def(DefOptions::new("*user-image*", janet_val));
    let mut janet_instructions = String::new();

    janet_instructions.push_str(r#"(merge-module (curenv) (load-image *user-image*) "" true)"#);

    if opts.destroy {
        janet_instructions.push_str("\n(setdyn :destroy-everything-you-touch true)");
    }

    if let Some(server) = server {
        janet_instructions.push_str(&format!(
            "\n(setdyn :server-name \"{server}:{SERVER_PORT}\")"
        ));
    }

    janet_instructions.push_str("\n(to-json (machine-config))");

    let janet_result = client.run(janet_instructions)?;
    Ok(janet_result.unwrap().to_string())
}

fn formatted_json(raw_json: &str) -> anyhow::Result<String> {
    match helpers::pretty_json(raw_json) {
        Ok(json) => Ok(json),
        Err(e) => {
            tracing::error!("JSON processing error: {}", e);
            tracing::error!(raw_json);
            bail!("END");
        }
    }
}

fn fetch_from_server(server: &str, hostname: &str, format: &str) -> anyhow::Result<Vec<u8>> {
    let mut tries = 1;

    while tries < 5 {
        tracing::debug!("try {tries} of 5");
        match fetch_precompiled_file(server, hostname, format) {
            Ok(resp) => {
                return Ok(resp);
            }
            Err(e) => {
                tracing::error!("error calling remote server: {e}");
                tracing::info!("sleeping for retry");
                thread::sleep(Duration::from_secs(tries * tries));
                tries += 1;
            }
        }
    }

    bail!("failed to get config from server");
}

/// Returns a Janet jimage of the user's config
pub fn local_janet_to_jimage(host_file: &Utf8PathBuf, opts: &ApplyOpts) -> anyhow::Result<Vec<u8>> {
    ensure!(
        host_file.exists(),
        "Cannot find host config file at {}",
        host_file
    );

    let host_file = host_file.canonicalize_utf8()?;
    let host_config_dir = host_file.parent().context("cannot get host config dir")?;
    let client = janet_helpers::gurp_client()?;

    let mut janet_instructions = String::new();

    janet_instructions.push_str("(def build-env (make-env (fiber/getenv (fiber/root))))\n");
    janet_instructions.push_str(&format!(
        "(set (build-env *syspath*) \"{host_config_dir}\")\n"
    ));
    janet_instructions.push_str(&format!(
        "(setdyn :gurp-config-root \"{host_config_dir}\")\n"
    ));
    janet_instructions.push_str(&format!(
        "(merge-module build-env (dofile \"{host_file}\" :env build-env) \"\" true)\n"
    ));
    janet_instructions.push_str("(make-image build-env)\n");

    if opts.dump_config {
        println!(
            "{}",
            helpers::dump_config(&janet_instructions, "Janet to compile image", opts)
        );
    }

    let result = client.run(janet_instructions)?;

    match result.unwrap() {
        TaggedJanet::Buffer(buf) => Ok(buf.as_bytes().to_vec()),
        _ => bail!("did not get image buffer"),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tester::fixture;

    #[test]
    fn test_local_janet_to_jimage() {
        let image =
            local_janet_to_jimage(&fixture("basic_config.janet"), &ApplyOpts::default()).unwrap();
        println!("{}", image.len());

        assert!(image.len() > 100); // if it fails it's 10b long
    }

    #[test]
    fn test_local_janet_to_json() {
        assert_eq!(
            r#"{"metadata":{"name":"test"},"resources":{"ensure":{"file":[{"_id":"/basenode/file/_tmp_tester","content":"blah","group":"root","mode":"0644","name":"/tmp/tester","owner":"root","role":"basenode"}]},"remove":{}}}"#,
            local_janet_to_json(&fixture("basic_config.janet"), &ApplyOpts::default()).unwrap()
        );
    }

    #[test]
    fn test_local_json_to_json() {
        assert_eq!(
            r#"{"a":1,"b":"string","c":[1,2,3]}"#,
            local_json_to_json(&fixture("json_input.json"))
                .unwrap()
                .trim()
        );
    }

    #[test]
    fn test_local_jimage_to_json() {
        assert_eq!(
            r#"{"metadata":{"name":"test"},"resources":{"ensure":{"file":[{"_id":"/NO-ROLE/file/_tmp_tester","content":"blah","group":"root","mode":"0644","name":"/tmp/tester","owner":"root"}]},"remove":{}}}"#,
            local_jimage_to_json(&fixture("basic_image.jimage"), &ApplyOpts::default()).unwrap()
        );
    }
}
