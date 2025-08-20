use anyhow::{Context, bail};
use camino::{Utf8Path, Utf8PathBuf};
use common::types::ApplyOpts;

// Wherein we read and prep the user-supplied Janet code

pub fn read_and_enrich_host_config(
    path: &Utf8PathBuf,
    format: Option<&str>,
    opts: &ApplyOpts,
) -> anyhow::Result<String> {
    let janet_host_config = std::fs::read_to_string(path)?;
    tracing::debug!("reading host config from {}", path);
    let qualified_path = path.canonicalize_utf8()?;

    let host_config_dir = qualified_path
        .parent()
        .context(format!("cannot find parent of {path}"))?;

    let gurp_lib = match &opts.gurp_lib_path {
        Some(path) => &load_lib_from_disk(path)?,
        None => crate::constants::GURP_LIB,
    };

    janet_conf(&janet_host_config, host_config_dir, gurp_lib, format, opts)
}

// We can inject our own Janet code into what the user gives us, that way the user doesn't
// have to work out include paths, and doesn't even have to have the gurp library. You can
// define and configure a host with just the gurp executable and a single (albeit quite big)
// Janet file.
//
// The user may specify an on-disk library file, which takes precedence over the hardcoded one.
//
// By setting the syspath dynamic binding, we let the user `use` role and library files without
// having to supply their path.
//
pub fn janet_conf(
    config: &str,
    conf_root: &Utf8Path,
    gurp_lib: &str,
    format: Option<&str>,
    opts: &ApplyOpts,
) -> anyhow::Result<String> {
    let mut ret = format!("(setdyn *syspath* \"{conf_root}\")\n\n");
    ret.push_str(&format!("(setdyn :gurp-config-root \"{conf_root}\")\n\n"));
    ret.push_str(crate::constants::GURP_DEFAULTS);
    ret.push_str(gurp_lib);
    ret.push('\n');
    ret.push_str(config);

    if let Some(format) = format
        && opts.compile_only
    {
        match format {
            "janet" => {
                if opts.colour {
                    ret.push_str("\n(prinf \"%M\" (machine-config))");
                } else {
                    ret.push_str("\n(prinf \"%m\" (machine-config))");
                }
            }
            "json" => ret.push_str("\n(print (encode (machine-config)))"),
            _ => bail!("format must be 'janet' or 'json'"),
        }
    }

    Ok(ret)
}

fn load_lib_from_disk(lib_path: &Utf8PathBuf) -> anyhow::Result<String> {
    if lib_path.exists() {
        tracing::debug!("injecting gurp lib '{}' in user Janet", lib_path);
        Ok(std::fs::read_to_string(lib_path)?)
    } else {
        bail!("Could not find gurp lib at {}", lib_path)
    }
}
