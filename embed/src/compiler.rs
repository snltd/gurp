use crate::client;
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use common::constants::SERVER_PORT;
use common::info;
use common::types::{ApplyOutputOpts, ApplyVmOpts, CompileError, JsonConfig};
use janetrs::client::JanetClient;
use janetrs::env::DefOptions;
use janetrs::{Janet, JanetString, JanetStruct, TaggedJanet};
use std::env;

/// A JsonCompiler turns Janet config into a JSON string
pub struct ConfigCompiler {
    client: JanetClient,
    output_opts: ApplyOutputOpts,
    config_file: Option<Utf8PathBuf>,
    dyns: Option<Dyns>,
}

struct Dyns {
    syspath: String,
    gurp_config_root: String,
}

impl ConfigCompiler {
    pub fn new(
        vm_opts: &ApplyVmOpts,
        destroy_everything_you_touch: bool,
        output_opts: ApplyOutputOpts,
        config_path: Option<&Utf8Path>,
    ) -> Result<Self, CompileError> {
        let client =
            client::gurp(vm_opts, destroy_everything_you_touch).map_err(CompileError::Other)?;

        let dyns = if let Some(config_path) = config_path {
            let host_file = config_path
                .canonicalize_utf8()
                .with_context(|| format!("failed to canonicalize {config_path}"))
                .map_err(CompileError::Other)?;

            let config_dir = host_file
                .parent()
                .context("cannot get parent of config file")
                .map_err(CompileError::Other)?;

            Some(Dyns {
                syspath: config_dir.to_string(),
                gurp_config_root: config_dir.to_string(),
            })
        } else {
            None
        };

        Ok(Self {
            client,
            output_opts,
            config_file: config_path.map(|p| p.to_owned()),
            dyns,
        })
    }

    // Get a string by compiling a local Janet file (and its dependencies)
    pub fn janet_file(&self, path: &Utf8Path, to_json: bool) -> Result<JsonConfig, CompileError> {
        if !path.exists() {
            return Err(CompileError::FileNotFound(path.to_owned()));
        }

        let Dyns {
            syspath,
            gurp_config_root,
        } = self
            .dyns
            .as_ref()
            .ok_or_else(|| CompileError::Other(anyhow::anyhow!("no dyns set")))?;

        let config_file = self
            .config_file
            .as_ref()
            .ok_or_else(|| CompileError::Other(anyhow::anyhow!("no config file set")))?;

        let final_cmd = if to_json {
            "(to-json (eval '(machine-config)))"
        } else {
            "(eval '(machine-config))"
        };

        let janet_instructions = indoc::formatdoc! { r#"
            (merge-module (curenv) (dofile "{config_file}" :env (curenv)) "" true)
            {final_cmd}"#
        };

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
        let compiled_bytes = compile(&self.client, code, &self.dyns, &self.output_opts)?;

        String::from_utf8(compiled_bytes).map_err(|e| {
            CompileError::Other(anyhow::anyhow!(
                "cannot convert compiled bytes to JSON string: {e}"
            ))
        })
    }
}

/// Trying to compile invalid Janet causes a Janet panic, with a stack trace
/// dumped to stderr. We want to capture the error so we can act accordingly,
/// and the stack trace so we can pass it back to a client from a server.
///
/// To do this we wrap the Janet so it runs in a fiber. Said fiber receives
/// the current environment as `outer-env` (which is resumed after the Janet
/// runs)
///
/// If the fiber errors,
/// `debug/stacktrace` puts the trace into a buffer (using
/// `with-dyns` to `:err`).The buffer is prefixed with
///
/// If the fiber completes without error, the wrapped code's own result is
/// returned unchanged.
///
fn wrapped_config(code: &str, dyns: &Option<Dyns>) -> String {
    let fib_block = if let Some(dyns) = dyns {
        let Dyns {
            syspath,
            gurp_config_root,
        } = dyns;

        &indoc::formatdoc! { r#"
            (def fib
              (fiber/new
                (fn []
                  (with-dyns [*syspath* "{syspath}"
                              :gurp-config-root "{gurp_config_root}"]
                      {code}))
                      :e
                      outer-env))"#}
    } else {
        "(def fib (fiber/new (fn [] {code} :e outer-env)))"
    };

    indoc::formatdoc! { r#"
    (def outer-env (curenv))

    {fib_block}

    (def result (resume fib))

    (if (= (fiber/status fib) :error)
      (let [buf @""]
        (with-dyns [:err buf] (debug/stacktrace fib result ""))
        (struct :error (string result) :trace (string buf)))
      result)"# 
    }
}

// Errors are wrapped now. They come in a struct with keys
// :error and :trace.
fn destructure_wrapped_error(st: JanetStruct) -> CompileError {
    let jerror = match st.get_owned(":error") {
        Some(err_msg) => err_msg.to_string(),
        None => {
            return CompileError::Other(anyhow::anyhow!(
                "could not get error field from Janet result struct"
            ));
        }
    };

    let jtrace = match st.get_owned(":trace") {
        Some(trace) => trace
            .to_string()
            .split('\n')
            .map(|s| s.to_owned())
            .collect(),
        None => {
            return CompileError::Other(anyhow::anyhow!(
                "could not get trace field from Janet result struct"
            ));
        }
    };

    CompileError::Compile {
        message: jerror,
        trace: jtrace,
    }
}

// Compile to a Vec<u8>, which can hold a jimage or be converted to a string, which we do
// if we expect JSON output.
fn compile(
    client: &JanetClient,
    code: &str,
    dyns: &Option<Dyns>,
    output_opts: &ApplyOutputOpts,
) -> Result<Vec<u8>, CompileError> {
    tracing::debug!("evaluating Janet config");
    let wrapped_code = wrapped_config(code, dyns);

    if output_opts.dump_configs {
        println!(
            "{}",
            info::dump_config(&wrapped_code, Some("Janet config"), output_opts)
        );
    }

    match client.run(&wrapped_code) {
        Ok(buf) => match buf.unwrap() {
            // Successful compilation to JSON gives us a JSON String
            TaggedJanet::String(str) => Ok(str.bytes().collect()),
            // Successful compilation to an image gives us a Buffer
            TaggedJanet::Buffer(buf) => {
                let bytes = buf.as_bytes();
                if bytes.starts_with(b"ERR:") {
                    let msg = String::from_utf8_lossy(&bytes[4..]).into_owned();
                    Err(CompileError::Other(anyhow::anyhow!(msg)))
                } else {
                    Ok(bytes.to_vec())
                }
            }
            // A compilation error gives us a struct
            TaggedJanet::Struct(jstruct) => Err(destructure_wrapped_error(jstruct)),
            // We shouldn't see anything else
            _ => Err(CompileError::Other(anyhow::anyhow!(
                "Janet eval returned unexpected type: expected String or Struct, got {buf:?}"
            ))),
        },
        Err(e) => {
            let err_desc = match e {
                janetrs::client::Error::AlreadyInit => "AlreadyInit",
                janetrs::client::Error::CompileError => "CompileError",
                janetrs::client::Error::EnvNotInit => "EnvNotInit",
                janetrs::client::Error::ParseError => "ParseError",
                janetrs::client::Error::RunError => "RunError",
                _ => "<UNKNOWN>",
            };

            tracing::error!("unhandled error compiling Janet config: {err_desc}");
            Err(CompileError::Other(e.into()))
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

    compile(
        &client,
        &janet_instructions,
        &Some(Dyns {
            syspath: host_config_dir.to_string(),
            gurp_config_root: host_config_dir.to_string(),
        }),
        &ApplyOutputOpts::default(),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tester::fixture;
    use camino_tempfile::NamedUtf8TempFile;
    use common::types::ApplyVmOpts;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_janet_file() {
        assert_eq!(
            r#"{"control-data":{},"metadata":{"name":"test"},"resources":{"ensure":{"file":[{"_id":"/basenode/file/_tmp_tester","content":"blah","group":"root","mode":"0644","name":"/tmp/tester","owner":"root","role":"basenode"}]},"remove":{}}}"#,
            test_compiler()
                .janet_file(&fixture("basic_config.janet"), true)
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

    #[test]
    fn test_file_is_bad_janet() {
        let mut file = NamedUtf8TempFile::new().unwrap();
        write!(file, "(unknown-function)").unwrap();

        let err = test_compiler().janet_file(file.path(), true).unwrap_err();

        match err {
            CompileError::Compile { message, trace } => {
                assert!(message.contains("compile error: unknown symbol unknown-function"));
                assert!(!trace.is_empty());
            }
            other => {
                panic!("expected CompileError::Compile, got {other:?}");
            }
        }
    }

    #[test]
    fn test_file_is_not_even_janet() {
        let err = test_compiler()
            .janet_file("/etc/passwd".into(), true)
            .unwrap_err();

        match err {
            CompileError::Compile { message, trace } => {
                assert!(message.contains("parse error"));
                assert!(!trace.is_empty());
            }
            other => {
                panic!("expected CompileError::Compile, got {other:?}");
            }
        }
    }

    #[test]
    fn test_janet_snippet() {
        assert_eq!(
            r#"{"control-data":{},"metadata":{"name":"gurp-runner"},"resources":{"ensure":{"directory":[{"_id":"/NO-ROLE/directory/_tmp_test1","group":"root","mode":"0755","name":"/tmp/test1","owner":"root","role":"NO-ROLE"}]},"remove":{}}}"#,
            test_compiler()
                .janet_snippet(r#"(directory/ensure "/tmp/test1")"#)
                .unwrap()
        );
    }

    // #[test]
    // fn test_snippet_is_not_even_janet() {
    //     let err = test_compiler().janet_snippet("123abc").unwrap_err();

    //     // match err {
    //     //     CompileError::Compile { message, trace } => {
    //     //         assert_eq!("merp", message);
    //     //     }
    //     //     other => {
    //     //         panic!("expected CompileError::Compile, got {other:?}");
    //     //     }
    //     // }
    // }

    fn test_compiler() -> ConfigCompiler {
        ConfigCompiler::new(
            &ApplyVmOpts::default(),
            false,
            ApplyOutputOpts::default(),
            None,
        )
        .unwrap()
    }
}
