use crate::client;
use anyhow::Context;
use camino::Utf8Path;
use common::constants::SERVER_PORT;
use common::info;
use common::types::{ApplyOutputOpts, ApplyVmOpts, CompileError, JsonConfig};
use janetrs::client::JanetClient;
use janetrs::env::DefOptions;
use janetrs::{Janet, JanetString, TaggedJanet};
use std::env;

<<<<<<< Updated upstream
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
=======
/// A JsonCompiler turns Janet config into a JSON string
pub struct ConfigCompiler {
    client: JanetClient,
    output_opts: ApplyOutputOpts,
}

impl ConfigCompiler {
    pub fn new(
        vm_opts: &ApplyVmOpts,
        destroy_everything_you_touch: bool,
        output_opts: ApplyOutputOpts,
    ) -> Result<Self, CompileError> {
        let client =
            client::gurp(vm_opts, destroy_everything_you_touch).map_err(CompileError::Other)?;

        Ok(Self {
            client,
            output_opts,
        })
    }

    // Get a string by compiling a local Janet file (and its dependencies)
    pub fn janet_file(&self, path: &Utf8Path, to_json: bool) -> Result<JsonConfig, CompileError> {
        if !path.exists() {
            return Err(CompileError::FileNotFound(path.to_owned()));
>>>>>>> Stashed changes
        }

        let host_file = path
            .canonicalize_utf8()
            .with_context(|| format!("failed to canonicalize {path}"))
            .map_err(CompileError::Other)?;

<<<<<<< Updated upstream
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
=======
        let config_dir = host_file
            .parent()
            .context("cannot get parent of config file")
            .map_err(CompileError::Other)?;

        let final_cmd = if to_json {
            "(to-json (machine-config))"
        } else {
            "(machine-config)"
        };
>>>>>>> Stashed changes

        let janet_instructions = indoc::formatdoc! { r#"
            (setdyn *syspath* "{config_dir}")
            (setdyn :gurp-config-root "{config_dir}")
            (merge-module (curenv) (dofile "{host_file}" :env (curenv)) "" true)
            {final_cmd}"#};

<<<<<<< Updated upstream
// Get a JSON string from a pre-compiled file on disk
fn local_json_to_json(path: &Utf8Path) -> Result<JsonConfig, CompileError> {
    if !path.exists() {
        return Err(CompileError::FileNotFound(path.to_owned()));
=======
        if self.output_opts.dump_configs {
            println!(
                "{}",
                info::dump_config(&janet_instructions, Some("Janet config"), &self.output_opts)
            );
        }

        self.compile_to_string(&janet_instructions)
>>>>>>> Stashed changes
    }

    // Get a string by compiling a snippet of Janet
    pub fn janet_snippet(&self, janet_snippet: &str) -> Result<JsonConfig, CompileError> {
        let cwd = env::current_dir()
            .map_err(CompileError::Io)?
            .to_string_lossy()
            .to_string();

<<<<<<< Updated upstream
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
    local_janet(host_file, opts, "(to-json (machine-config))")
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
=======
        let janet_instructions = indoc::formatdoc! { r#"
>>>>>>> Stashed changes
            (setdyn *syspath* "{cwd}")
            (setdyn :gurp-config-root "{cwd}")

            (host "gurp-runner"
                {janet_snippet})

            (to-json (machine-config))"#};

<<<<<<< Updated upstream
    if opts.dump_config {
        println!(
            "{}",
            info::dump_config(&janet_instructions, Some("Janet config"), opts)
        );
=======
        if self.output_opts.dump_configs {
            println!(
                "{}",
                info::dump_config(&janet_instructions, Some("Janet config"), &self.output_opts)
            );
        }

        self.compile_to_string(&janet_instructions)
>>>>>>> Stashed changes
    }

    pub fn janet_image(
        &mut self,
        raw_image: &[u8],
        server: Option<&str>,
    ) -> Result<JsonConfig, CompileError> {
        let jstr = JanetString::new(raw_image);
        let janet_val = Janet::string(jstr);
        self.client
            .add_def(DefOptions::new("*user-image*", janet_val));

        let server = if let Some(server) = server {
            format!("\n(setdyn :server-name \"{server}:{SERVER_PORT}\")")
        } else {
            String::new()
        };

<<<<<<< Updated upstream
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

        (merge-module (curenv) (dofile "{host_file}" :env (curenv)) "" true)
        {final_cmd}"#};

    if opts.dump_config {
        println!(
            "{}",
            info::dump_config(&janet_instructions, Some("Janet config"), opts)
        );
    }

    compile_to_string(&client, &janet_instructions)
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
=======
        let janet_instructions = indoc::formatdoc! { r#"
            (merge-module (curenv) (load-image *user-image*) "" true)
            {server}
            (to-json (eval '(machine-config)))
>>>>>>> Stashed changes
    "#};

        self.compile_to_string(&janet_instructions)
    }

<<<<<<< Updated upstream
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
        (merge-module build-env (dofile "{host_file}" :env build-env) "" true)
        (make-image build-env)
        "#};

    if opts.dump_config {
        println!(
            "{}",
            info::dump_config(&janet_instructions, Some("Janet to compile image"), opts)
        );
=======
    // Wrapper for compile() for when we want to get a JSON string
    fn compile_to_string(&self, code: &str) -> Result<JsonConfig, CompileError> {
        compile(&self.client, code)
            .and_then(|bytes| String::from_utf8(bytes).map_err(|e| CompileError::Compile(e.into())))
>>>>>>> Stashed changes
    }
}
//
// Compile to a Vec<u8>, which can hold a jimage or be converted to a string, which we do
// if we expect JSON output.
fn compile(client: &JanetClient, code: &str) -> Result<Vec<u8>, CompileError> {
    tracing::debug!("evaluating Janet config");

    match client.run(code) {
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

/// Used in server mode to create a Janet jimage of a machine config
pub fn to_jimage(path: &Utf8Path) -> Result<Vec<u8>, CompileError> {
    if !path.exists() {
        return Err(CompileError::FileNotFound(path.to_owned()));
    }

    let host_file = path.canonicalize_utf8().map_err(CompileError::Io)?;
    let host_config_dir = host_file
        .parent()
        .context("cannot get host config dir")
        .map_err(CompileError::Other)?;

<<<<<<< Updated upstream
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
=======
    let client =
        client::gurp(&ApplyVmOpts::default(), false).map_err(CompileError::ClientCreate)?;

    let janet_instructions = indoc::formatdoc! { r#"
        (def build-env (make-env (fiber/getenv (fiber/root))))
        (set (build-env *syspath*) "{host_config_dir}")
        (setdyn :gurp-config-root "{host_config_dir}")
        (merge-module build-env (dofile "{host_file}" :env build-env) "" true)
        (make-image build-env)
        "#};

    compile(&client, &janet_instructions)
>>>>>>> Stashed changes
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tester::fixture;
<<<<<<< Updated upstream
=======
    use common::types::ApplyVmOpts;
    use pretty_assertions::assert_eq;
    use std::fs;

    fn test_compiler() -> ConfigCompiler {
        ConfigCompiler::new(&ApplyVmOpts::default(), false, ApplyOutputOpts::default()).unwrap()
    }
>>>>>>> Stashed changes

    #[test]
    fn test_janet_file() {
        assert_eq!(
            r#"{"metadata":{"name":"test"},"resources":{"ensure":{"file":[{"_id":"/basenode/file/_tmp_tester","content":"blah","group":"root","mode":"0644","name":"/tmp/tester","owner":"root","role":"basenode"}]},"remove":{}}}"#,
            test_compiler()
                .janet_file(&fixture("basic_config.janet"), true)
                .unwrap()
        );
    }

    #[test]
    fn test_janet_snippet() {
        assert_eq!(
            r#"{"metadata":{"name":"gurp-runner"},"resources":{"ensure":{"directory":[{"_id":"/NO-ROLE/directory/_tmp_test1","group":"root","mode":"0755","name":"/tmp/test1","owner":"root","role":"NO-ROLE"}]},"remove":{}}}"#,
            test_compiler()
                .janet_snippet(r#"(directory/ensure "/tmp/test1")"#)
                .unwrap()
        );
    }

    #[test]
    fn test_to_jimage() {
        let image = to_jimage(&fixture("basic_config.janet")).unwrap();
        assert!(image.len() > 100); // if it fails it's 10b long
    }

    #[test]
    fn test_jimage() {
        assert_eq!(
            r#"{"metadata":{"name":"test"},"resources":{"ensure":{"file":[{"_id":"/NO-ROLE/file/_tmp_tester","content":"blah","group":"root","mode":"0644","name":"/tmp/tester","owner":"root"}]},"remove":{}}}"#,
<<<<<<< Updated upstream
            local_jimage_to_json(&fixture("basic_image.jimage"), &ApplyOpts::default()).unwrap()
=======
            test_compiler()
                .janet_image(&fs::read(fixture("basic_image.jimage")).unwrap(), None)
                .unwrap()
>>>>>>> Stashed changes
        );
    }
}
