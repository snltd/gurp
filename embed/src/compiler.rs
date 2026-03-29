use crate::client;
use anyhow::{Context, bail, ensure};
use camino::Utf8PathBuf;
use common::constants::{CLIENT_API_VERSION, SERVER_PORT};
use common::info;
use common::types::{ApplyOpts, JsonConfig};
use janetrs::env::DefOptions;
use janetrs::{Janet, JanetString, TaggedJanet};
use std::time::Duration;
use std::{env, fs, thread};
use util::{http, info as util_info, json};

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
        .map_or_else(util_info::my_hostname, Ok)?;

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
        .map_or_else(util_info::my_hostname, Ok)?;

    let host_config = String::from_utf8(fetch_from_server(server, &hostname, "json")?)?;

    if opts.dump_config {
        let formatted_json = json::pretty(&host_config)?;

        println!(
            "{}",
            info::dump_config(&formatted_json, Some("Janet config"), opts)
        );
    }

    Ok(host_config)
}

pub fn local_janet_to_json(
    host_file: &Utf8PathBuf,
    opts: &ApplyOpts,
) -> anyhow::Result<JsonConfig> {
    local_janet(host_file, opts, "(to-json (machine-config))")
}

pub fn local_janet_to_janet(host_file: &Utf8PathBuf, opts: &ApplyOpts) -> anyhow::Result<String> {
    let format = if opts.colour { "%M" } else { "%m" };
    let compiled_janet = local_janet(
        host_file,
        opts,
        &format!(r#"(string/format "{format}" (machine-config))"#),
    )?;

    Ok(info::dump_config(&compiled_janet, None, opts))
}

// Get a string by compiling a snippet of Janet
pub fn raw_janet_to_json(janet_snippet: &str, opts: &ApplyOpts) -> anyhow::Result<JsonConfig> {
    let client = client::gurp()?;
    let cwd = env::current_dir()?.to_string_lossy().to_string();

    let destroyer = if opts.destroy {
        "(setdyn :destroy-everything-you-touch true)"
    } else {
        ""
    };

    let janet_instructions = indoc::formatdoc! { r#"
            (setdyn *syspath* "{cwd}")
            (setdyn :gurp-config-root "{cwd}")
            {destroyer}
            (host "gurp-runner"
            {janet_snippet})
            (to-json (machine-config))
        "#};

    if opts.dump_config {
        println!(
            "{}",
            info::dump_config(&janet_instructions, Some("Janet config"), opts)
        );
    }

    let janet_result = client.run(janet_instructions)?;
    Ok(janet_result.unwrap().to_string())
}

// Get a string by compiling a local Janet file (and its dependencies)
pub fn local_janet(
    host_file: &Utf8PathBuf,
    opts: &ApplyOpts,
    final_cmd: &str,
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

    let client = client::gurp()?;

    let destroyer = if opts.destroy {
        "(setdyn :destroy-everything-you-touch true)"
    } else {
        ""
    };

    let janet_instructions = indoc::formatdoc! { r#"
            (setdyn *syspath* "{config_dir}")
            (setdyn :gurp-config-root "{config_dir}")
            {destroyer}
            (merge-module (curenv) (dofile "{host_file}" :env (curenv)) "" true)
            {final_cmd}
        "#};

    if opts.dump_config {
        println!(
            "{}",
            info::dump_config(&janet_instructions, Some("Janet config"), opts)
        );
    }

    let janet_result = client.run(janet_instructions)?;
    Ok(janet_result.unwrap().to_string())
}

// We tell the server what we think it's called so it can build file resources we can find. This
// lets us use a raw IP address, DNS name, whatever.
fn fetch_precompiled_file(server: &str, hostname: &str, format: &str) -> anyhow::Result<Vec<u8>> {
    let url = format!(
        "http://{server}:{SERVER_PORT}/{CLIENT_API_VERSION}/config/{hostname}?server_name={server}&format={format}"
    );
    tracing::info!("fetching config from {url}");
    http::remote_file_to_memory(&url)
}

pub fn jimage_to_json(
    raw_image: &[u8],
    server: Option<&str>,
    opts: &ApplyOpts,
) -> anyhow::Result<JsonConfig> {
    let mut client = client::gurp()?;
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
    let client = client::gurp()?;

    let janet_instructions = indoc::formatdoc! { r#"
            (def build-env (make-env (fiber/getenv (fiber/root))))
            (set (build-env *syspath*) "{host_config_dir}")
            (setdyn :gurp-config-root "{host_config_dir}")
            (merge-module build-env (dofile "{host_file}" :env build-env) "" true)
            (make-image build-env)
        "#};

    if opts.dump_config {
        println!(
            "{}",
            info::dump_config(&janet_instructions, Some("Janet to compile image"), opts)
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
