use crate::client;
use anyhow::Context;
use camino::Utf8Path;
use common::constants::{CLIENT_API_VERSION, CLIENT_RETRIES, SERVER_PORT};
use common::info;
use common::types::{ApplyOpts, CompileError, JsonConfig};
use janetrs::client::JanetClient;
use janetrs::env::DefOptions;
use janetrs::{Janet, JanetString, TaggedJanet};
use std::time::Duration;
use std::{env, fs, thread};
use util::{http, info as util_info, json};

// When Janet compilation fails, perhaps because of a missing module or syntax error, the
// embedded interpreter issues a Janet panic. janetrs does not offer any way to catch this,
// so the error message is dumped to stderr. We would prefer to capture it and log it properly.
//
// To do this, we wrap fallible Janet function calls in (protect). From the docs:
// (protect & body)
// Evaluate expressions, while capturing any errors. Evaluates to a tuple
// of two elements. The first element is true if successful, false if an
// error, and the second is the return value or error.
//
// It's a bit messy, but I think it's the best we can do for the time being. A side effect is
// that we lose the stack trace, but I generally find it only muddies the waters in this
// situation, so that's fine.

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
    host_file: Option<&Utf8Path>,
    opts: &ApplyOpts,
) -> Result<String, CompileError> {
    let ret = if opts.image {
        tracing::debug!("applying precompiled jimage config");
        // case 4
        let image_file = host_file
            .context("No host file specified")
            .map_err(CompileError::Other)?;
        local_jimage_to_json(image_file, opts)?
    } else if opts.precompiled {
        tracing::debug!("applying precompiled JSON config");
        // case 2
        let host_file = host_file
            .context("No host file specified")
            .map_err(CompileError::Other)?;
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
        unreachable!("no configuration file specified")
    };

    Ok(ret)
}

// Get a JSON string from a Janet image on disk
pub fn local_jimage_to_json(path: &Utf8Path, opts: &ApplyOpts) -> Result<JsonConfig, CompileError> {
    if path.exists() {
        jimage_to_json(
            &fs::read(path).map_err(|e| CompileError::Other(e.into()))?,
            None,
            opts,
        )
    } else {
        Err(CompileError::FileNotFound(path.to_owned()))
    }
}

// Make a JSON string from a Janet image fetched from a remote server
fn remote_jimage_to_json(server: &str, opts: &ApplyOpts) -> Result<JsonConfig, CompileError> {
    let hostname = opts
        .hostname
        .clone()
        .map_or_else(util_info::my_hostname, Ok)
        .map_err(CompileError::Other)?;

    jimage_to_json(
        &fetch_from_server(server, &hostname, "jimage")
            .with_context(|| format!("failed to fetch jimage for {hostname} from {server}"))
            .map_err(CompileError::Network)?,
        Some(server),
        opts,
    )
}

// Get a JSON string from a pre-compiled file on disk
fn local_json_to_json(path: &Utf8Path) -> Result<JsonConfig, CompileError> {
    if !path.exists() {
        return Err(CompileError::FileNotFound(path.to_owned()));
    }

    fs::read_to_string(path).map_err(|e| CompileError::Other(e.into()))
}

// Get a JSON string from a remote server
fn remote_json_to_json(server: &str, opts: &ApplyOpts) -> Result<JsonConfig, CompileError> {
    let hostname = opts
        .hostname
        .clone()
        .map_or_else(util_info::my_hostname, Ok)
        .map_err(CompileError::Other)?;

    let host_config = String::from_utf8(
        fetch_from_server(server, &hostname, "json")
            .with_context(|| format!("failed to fetch JSON config for {hostname} from {server}"))
            .map_err(CompileError::Network)?,
    )
    .map_err(|e| CompileError::Other(e.into()))?;

    if opts.dump_config {
        let formatted_json = json::pretty(&host_config).map_err(CompileError::Other)?;

        println!(
            "{}",
            info::dump_config(&formatted_json, Some("Janet config"), opts)
        );
    }

    Ok(host_config)
}

pub fn local_janet_to_json(
    host_file: &Utf8Path,
    opts: &ApplyOpts,
) -> Result<JsonConfig, CompileError> {
    local_janet(
        host_file,
        opts,
        indoc::indoc! { r#"
          (def cmd-result (protect (eval '(machine-config))))

          (if (cmd-result 0)
            (to-json (cmd-result 1))
            (buffer/push (buffer "ERR:") (string (cmd-result 1))))"#,
        },
    )
}

pub fn local_janet_to_janet(
    host_file: &Utf8Path,
    opts: &ApplyOpts,
) -> Result<String, CompileError> {
    let format = if opts.colour { "%M" } else { "%m" };
    let compiled_janet = local_janet(
        host_file,
        opts,
        &format!(r#"(string/format "{format}" (machine-config))"#),
    )?;

    Ok(info::dump_config(&compiled_janet, None, opts))
}

// Get a string by compiling a snippet of Janet
pub fn raw_janet_to_json(
    janet_snippet: &str,
    opts: &ApplyOpts,
) -> Result<JsonConfig, CompileError> {
    let client = client::gurp().map_err(CompileError::ClientCreate)?;
    let cwd = env::current_dir()
        .map_err(CompileError::Io)?
        .to_string_lossy()
        .to_string();

    let destroyer = destroyer_string(opts);

    let janet_instructions = indoc::formatdoc! { r#"
            (setdyn *syspath* "{cwd}")
            (setdyn :gurp-config-root "{cwd}")
            {destroyer}

            (host "gurp-runner"
                {janet_snippet})

            (to-json (machine-config))"#};

    if opts.dump_config {
        println!(
            "{}",
            info::dump_config(&janet_instructions, Some("Janet config"), opts)
        );
    }

    compile_to_string(&client, &janet_instructions, false)
}

// Get a string by compiling a local Janet file (and its dependencies)
pub fn local_janet(
    path: &Utf8Path,
    opts: &ApplyOpts,
    final_cmd: &str,
) -> Result<JsonConfig, CompileError> {
    if !path.exists() {
        return Err(CompileError::FileNotFound(path.to_owned()));
    }

    let host_file = path
        .canonicalize_utf8()
        .with_context(|| format!("failed to canonicalize {path}"))
        .map_err(CompileError::Other)?;

    let config_dir = host_file
        .parent()
        .context("cannot get parent of config file")
        .map_err(CompileError::Other)?;

    let client = client::gurp().map_err(CompileError::ClientCreate)?;
    let destroyer = destroyer_string(opts);

    let janet_instructions = indoc::formatdoc! { r#"
        (setdyn *syspath* "{config_dir}")
        (setdyn :gurp-config-root "{config_dir}")
        {destroyer}

        (def load-result
          (protect
            (merge-module (curenv) (dofile "{host_file}" :env (curenv)) "" true)))

        (if (load-result 0)
          (do
            {final_cmd})
          (buffer/push (buffer "ERR:") (string (load-result 1))))"#};

    if opts.dump_config {
        println!(
            "{}",
            info::dump_config(&janet_instructions, Some("Janet config"), opts)
        );
    }

    compile_to_string(&client, &janet_instructions, true)
}

pub fn jimage_to_json(
    raw_image: &[u8],
    server: Option<&str>,
    opts: &ApplyOpts,
) -> Result<JsonConfig, CompileError> {
    let mut client = client::gurp().map_err(CompileError::ClientCreate)?;
    let jstr = JanetString::new(raw_image);
    let janet_val = Janet::string(jstr);
    client.add_def(DefOptions::new("*user-image*", janet_val));

    let destroyer = destroyer_string(opts);

    let server = if let Some(server) = server {
        format!("\n(setdyn :server-name \"{server}:{SERVER_PORT}\")")
    } else {
        String::new()
    };

    let janet_instructions = indoc::formatdoc! { r#"
        (merge-module (curenv) (load-image *user-image*) "" true)
        {destroyer}
        {server}
        (to-json (eval '(machine-config)))
    "#};

    compile_to_string(&client, &janet_instructions, true)
}

/// Returns a Janet jimage of the user's config
pub fn local_janet_to_jimage(path: &Utf8Path, opts: &ApplyOpts) -> Result<Vec<u8>, CompileError> {
    if !path.exists() {
        return Err(CompileError::FileNotFound(path.to_owned()));
    }

    let host_file = path.canonicalize_utf8().map_err(CompileError::Io)?;
    let host_config_dir = host_file
        .parent()
        .context("cannot get host config dir")
        .map_err(CompileError::Other)?;
    let client = client::gurp().map_err(CompileError::ClientCreate)?;

    let janet_instructions = indoc::formatdoc! { r#"
        (def build-env (make-env (fiber/getenv (fiber/root))))
        (set (build-env *syspath*) "{host_config_dir}")
        (setdyn :gurp-config-root "{host_config_dir}")

        (def load-result
          (protect
            (merge-module build-env (dofile "{host_file}" :env build-env) "" true)))

        (if (load-result 0)
          (make-image build-env)
          (buffer/push (buffer "ERR:") (string (load-result 1))))"#};

    if opts.dump_config {
        println!(
            "{}",
            info::dump_config(&janet_instructions, Some("Janet to compile image"), opts)
        );
    }

    compile(&client, &janet_instructions, true)
}

// Compile to a Vec<u8>, which can hold a jimage or be converted to a string, which we do
// if we expect JSON output.
fn compile(client: &JanetClient, code: &str, wrap: bool) -> Result<Vec<u8>, CompileError> {
    tracing::debug!("evaluating Janet config");

    // A Janet panic in the run phase will dump the error to stderr. We'd prefer to capture
    // it and write it to the logs, hence the protect, which catches errors and converts into
    // true/false.
    //
    let to_run = if wrap {
        &indoc::formatdoc! {
        "(match
            (protect (do
                {code}))
            [true result] result
            [false err] (buffer (string err)))"
        }
    } else {
        code
    };

    match client.run(to_run) {
        Ok(buf) => match buf.unwrap() {
            TaggedJanet::String(s) => Ok(s.bytes().collect()),
            TaggedJanet::Buffer(buf) => {
                let bytes = buf.as_bytes();
                if bytes.starts_with(b"ERR:") {
                    let msg = String::from_utf8_lossy(&bytes[4..]).into_owned();
                    Err(CompileError::Compile(anyhow::anyhow!(msg)))
                } else {
                    Ok(bytes.to_vec())
                }
            } //
            _ => Err(CompileError::Compile(anyhow::anyhow!(
                "Janet eval returned unexpected type: expected Buffer, got {buf:?}"
            ))),
        },
        Err(e) => {
            let err_desc = match e {
                janetrs::client::Error::AlreadyInit => "AlreadyInit",
                janetrs::client::Error::CompileError => "CompileError",
                janetrs::client::Error::EnvNotInit => "EnvNotInit",
                janetrs::client::Error::ParseError => "ParseError",
                janetrs::client::Error::RunError => "RunError",
                _ => "<UNKNOWN>", // Should be impossible
            };

            tracing::error!("error compiling Janet config: {err_desc}");
            Err(CompileError::Compile(e.into()))
        }
    }
}

// Wrapper for compile() for when we want to get a JSON string
fn compile_to_string(
    client: &JanetClient,
    code: &str,
    wrap: bool,
) -> Result<JsonConfig, CompileError> {
    compile(client, code, wrap)
        .and_then(|bytes| String::from_utf8(bytes).map_err(|e| CompileError::Compile(e.into())))
}

fn fetch_from_server(server: &str, hostname: &str, format: &str) -> Result<Vec<u8>, CompileError> {
    let mut tries = 1;
    let mut err: Option<anyhow::Error> = None;

    while tries < CLIENT_RETRIES {
        tracing::debug!("try {tries}/{CLIENT_RETRIES}");

        match fetch_precompiled_file(server, hostname, format) {
            Ok(resp) => {
                return Ok(resp);
            }
            Err(e) => {
                tracing::error!("error calling remote server: {e}");
                tracing::info!("sleeping for retry");
                thread::sleep(Duration::from_secs(tries * tries));
                tries += 1;
                err = Some(e.into());
            }
        }
    }

    Err(CompileError::Network(err.unwrap()))
}

fn destroyer_string(opts: &ApplyOpts) -> String {
    if opts.destroy {
        tracing::debug!("enabling destroy-everything-you-touch");
        "(setdyn :destroy-everything-you-touch true)".to_owned()
    } else {
        String::new()
    }
}

// We tell the server what we think it's called so it can build file resources we can find. This
// lets us use a raw IP address, DNS name, whatever.
fn fetch_precompiled_file(
    server: &str,
    hostname: &str,
    format: &str,
) -> Result<Vec<u8>, CompileError> {
    let url = format!(
        "http://{server}:{SERVER_PORT}/{CLIENT_API_VERSION}/config/{hostname}?server_name={server}&format={format}"
    );
    tracing::info!("fetching config from {url}");
    http::remote_file_to_memory(&url).map_err(CompileError::Network)
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
