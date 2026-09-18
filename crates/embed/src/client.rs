use super::janet_cfuncs;
use anyhow::Context;
use common::constants::SANDBOX_FORBIDDEN_CAPABILITIES;
use common::types::ApplyVmOpts;
use janetrs::client::JanetClient;
use janetrs::env::CFunOptions;

/// Returns a standard Janet client, with no Gurp library.
pub fn vanilla() -> JanetClient {
    tracing::debug!("Initialising vanilla Janet client");
    JanetClient::init_with_default_env().expect("Failed to create Janet client")
}

/// Returns a Janet client with the Gurp library in the root environment. Also includes
/// (to-json) which turns any suitable Janet object into JSON.
pub fn gurp(vm_opts: &ApplyVmOpts) -> anyhow::Result<JanetClient> {
    let mut client = vanilla();
    tracing::debug!("enriching Janet client");

    let c_syspath = vm_opts
        .syspath
        .canonicalize_utf8()
        .with_context(|| format!("cannot canonicalize syspath: {}", vm_opts.syspath))?;

    let c_config_root = vm_opts
        .gurp_config_root
        .canonicalize_utf8()
        .with_context(|| {
            format!(
                "cannot canonicalize gurp_config_root: {}",
                vm_opts.gurp_config_root
            )
        })?;

    client.add_c_fn(CFunOptions::new(
        c"gurp-library",
        janet_cfuncs::gurp_library_c,
    ));

    client.add_c_fn(CFunOptions::new(c"to-json", janet_cfuncs::to_json_c));

    let mut janet_instructions = indoc::formatdoc! {r#"
        (setdyn *syspath* "{c_syspath}")
        (setdyn :gurp-config-root "{c_config_root}")
        (merge-module (fiber/getenv (fiber/root)) (load-image (gurp-library)) "" true)
        "#
    };

    client.add_c_fn(CFunOptions::new(
        c"run-safe-cmd",
        janet_cfuncs::run_safe_cmd_c,
    ));

    client.add_c_fn(CFunOptions::new(c"run-cmd", janet_cfuncs::run_cmd_c));

    janet_instructions.push_str(&format!(
        "\n(sandbox {})\n",
        SANDBOX_FORBIDDEN_CAPABILITIES.join(" ")
    ));

    if vm_opts.destroy_everything_you_touch {
        janet_instructions.push_str(&destroyer_string());
    }

    if vm_opts.define.is_empty() {
        janet_instructions.push_str(r#"(setdyn :gurp-user-defs {})"#);
    } else {
        janet_instructions.push_str(&define_string(vm_opts));
    }

    tracing::debug!("creating new Janet client with Gurp environment");
    client.run(janet_instructions)?;
    tracing::debug!("successfully created Gurp client");

    Ok(client)
}

fn destroyer_string() -> String {
    tracing::debug!("enabling destroy-everything-you-touch");
    "(setdyn :destroy-everything-you-touch true)".to_owned()
}

/// Builds a Janet struct from the contents of opts.define, which is a Vec<String>. Values which
/// contain an '=', are split on that char, with the first part becoming a struct key (keyword)
/// and the second becoming the corresponding value (string). If there is no '=', the whole value
/// becomes a key (keyword) and the value is set to true (boolean).
pub fn define_string(vmopts: &ApplyVmOpts) -> String {
    tracing::debug!("setting gurp-user-defs");

    let bindings = vmopts
        .define
        .iter()
        .filter_map(|d| {
            let mut chunks = d.splitn(2, '=');

            if let Some(key) = chunks.next() {
                let value = if let Some(v) = chunks.next() {
                    &format!("\"{v}\"")
                } else {
                    "true"
                };

                Some(format!("(keyword \"{key}\") {value}"))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    if bindings.is_empty() {
        String::new()
    } else {
        format!(r#"(setdyn :gurp-user-defs (struct {bindings}))"#)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert;
    use tester::test_vm_opts;

    #[test]
    fn test_vanilla_client() {
        let client = vanilla();
        assert_eq!(3, convert::janet_to_json(&client.run("(+ 1 2)").unwrap()));
    }

    #[test]
    fn test_gurp_client() {
        let client = gurp(&test_vm_opts()).unwrap();
        assert_eq!(3, convert::janet_to_json(&client.run("(+ 1 2)").unwrap()));

        assert_eq!(
            "/path/to/file",
            convert::janet_to_json(&client.run(r#"(pathcat "path" "to" "file")"#).unwrap())
        );

        assert_eq!(
            r#"{"a":123}"#,
            convert::janet_to_json(&client.run(r#"(to-json {:a 123})"#).unwrap())
        );
    }

    #[test]
    fn test_define_string() {
        let opts = ApplyVmOpts {
            define: vec!["boolean".to_owned(), "key=value".to_owned()],
            ..Default::default()
        };

        assert_eq!(
            r#"(setdyn :gurp-user-defs (struct (keyword "boolean") true (keyword "key") "value"))"#,
            define_string(&opts)
        );
    }
}
