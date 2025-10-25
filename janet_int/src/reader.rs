use anyhow::{Context, bail};
use camino::{Utf8Path, Utf8PathBuf};
use common::constants::SERVER_PORT;
use common::types::ApplyOpts;

// Wherein we read and prep the user-supplied Janet code
//
// We can inject our own Janet code into what the user gives us, that way the user doesn't
// have to work out include paths, and doesn't even have to have the gurp library. You can
// define and configure a host with just the gurp executable and a single (albeit quite big)
// Janet file.

// The assembled config is a big chunk of text which bundles the user's top-level configuration
// with a Gurp library and some bindings that library requires.
//
pub fn assembled_config(path: &Utf8PathBuf, opts: &ApplyOpts) -> anyhow::Result<String> {
    let host_config = users_janet_config(path)?;

    assemble(&host_config, path, opts)
}

// Broken out because it needs to be called directly by tests
//
pub fn assemble(host_conf: &str, path: &Utf8PathBuf, opts: &ApplyOpts) -> anyhow::Result<String> {
    let host_config_dir = path
        .parent()
        .context(format!("cannot find parent of {path}"))?;

    let mut conf = String::new();

    conf.push_str(&dynamic_bindings(host_config_dir, path));
    conf.push_str(&server_bindings(
        opts.server_name.as_deref(),
        opts.client_name.as_deref(),
    ));
    conf.push_str(default_values());
    conf.push_str(&gurp_lib(opts.gurp_lib_path.as_ref())?);
    conf.push('\n');
    conf.push_str(host_conf);

    Ok(conf)
}

// Janet dynamic bindings. These go at the top of the file and are referred to by various
// library functions.
//
// By setting the `syspath`, we let the user `use`/`import` role and library files without
// having to supply their path.
fn dynamic_bindings(host_config_dir: &Utf8Path, config_file: &Utf8PathBuf) -> String {
    indoc::formatdoc! { r#"
        (setdyn *syspath* "{host_config_dir}")
        (setdyn :gurp-config-root "{host_config_dir}")
        (setdyn :config-file "{config_file}")
        (setdyn *syspath* "{host_config_dir}")

        "# }
}

// When running in server mode the front-end converts local file references to HTTP file references.
// This dyn tells the front-end what the client thinks the server is called.
fn server_bindings(server_name: Option<&str>, client_name: Option<&str>) -> String {
    if let Some(server_name) = server_name
        && let Some(client_name) = client_name
    {
        indoc::formatdoc! { r#"
            (setdyn :server-name "{server_name}:{SERVER_PORT}")
            (setdyn :client-name "{client_name}")
        "# }
    } else {
        indoc::formatdoc! { r#"
            (setdyn :server-name nil)
            (setdyn :client-name nil)
        "# }
    }
}

// Hardcoded default values. At some point we'll let the user add their own.
fn default_values() -> &'static str {
    crate::constants::GURP_DEFAULTS
}

// Raw text of the user's top-level config file. Janet will sort out all the uses and imports
fn users_janet_config(path: &Utf8PathBuf) -> anyhow::Result<String> {
    let path = path.canonicalize_utf8()?;

    tracing::debug!("reading host config from {}", path);

    let janet_host_config = std::fs::read_to_string(&path)?;
    Ok(janet_host_config)
}

// The Gurp library. The user may specify an on-disk library file, which takes precedence
// over the hardcoded one.
fn gurp_lib(gurp_lib_path: Option<&Utf8PathBuf>) -> anyhow::Result<String> {
    let lib_as_string = if let Some(lib_path) = gurp_lib_path {
        if lib_path.exists() {
            tracing::debug!("using Gurp lib at {lib_path}");
            std::fs::read_to_string(lib_path)?
        } else {
            bail!("no Gurp lib at {}", lib_path)
        }
    } else {
        tracing::debug!("using built-in Gurp lib");
        crate::constants::GURP_LIB.to_owned()
    };

    Ok(lib_as_string)
}
