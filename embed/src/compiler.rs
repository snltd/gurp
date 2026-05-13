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
        }

        let host_file = path
            .canonicalize_utf8()
            .with_context(|| format!("failed to canonicalize {path}"))
            .map_err(CompileError::Other)?;

        let config_dir = host_file
            .parent()
            .context("cannot get parent of config file")
            .map_err(CompileError::Other)?;

        let final_cmd = if to_json {
            "(to-json (machine-config))"
        } else {
            "(machine-config)"
        };

        let janet_instructions = indoc::formatdoc! { r#"
            (setdyn *syspath* "{config_dir}")
            (setdyn :gurp-config-root "{config_dir}")
            (merge-module (curenv) (dofile "{host_file}" :env (curenv)) "" true)
            {final_cmd}"#};

        if self.output_opts.dump_configs {
            println!(
                "{}",
                info::dump_config(&janet_instructions, Some("Janet config"), &self.output_opts)
            );
        }

        self.compile_to_string(&janet_instructions)
    }

    // Get a string by compiling a snippet of Janet
    pub fn janet_snippet(&self, janet_snippet: &str) -> Result<JsonConfig, CompileError> {
        let cwd = env::current_dir()
            .map_err(CompileError::Io)?
            .to_string_lossy()
            .to_string();

        let janet_instructions = indoc::formatdoc! { r#"
            (setdyn *syspath* "{cwd}")
            (setdyn :gurp-config-root "{cwd}")

            (host "gurp-runner"
                {janet_snippet})

            (to-json (machine-config))"#};

        if self.output_opts.dump_configs {
            println!(
                "{}",
                info::dump_config(&janet_instructions, Some("Janet config"), &self.output_opts)
            );
        }

        self.compile_to_string(&janet_instructions)
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

        let janet_instructions = indoc::formatdoc! { r#"
            (merge-module (curenv) (load-image *user-image*) "" true)
            {server}
            (to-json (eval '(machine-config)))
    "#};

        self.compile_to_string(&janet_instructions)
    }

    // Wrapper for compile() for when we want to get a JSON string
    fn compile_to_string(&self, code: &str) -> Result<JsonConfig, CompileError> {
        compile(&self.client, code)
            .and_then(|bytes| String::from_utf8(bytes).map_err(|e| CompileError::Compile(e.into())))
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tester::fixture;
    use common::types::ApplyVmOpts;
    use std::fs;

    fn test_compiler() -> ConfigCompiler {
        ConfigCompiler::new(&ApplyVmOpts::default(), false, ApplyOutputOpts::default()).unwrap()
    }

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
            test_compiler()
                .janet_image(&fs::read(fixture("basic_image.jimage")).unwrap(), None)
                .unwrap()
        );
    }
}
