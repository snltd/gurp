use crate::common::constants::{GURP_DEFAULTS, GURP_LIB};
use crate::common::types::Opts;
use crate::debug;
use anyhow::{Context, anyhow};
use camino::Utf8PathBuf;

// Wherein we read and prep the user-supplied Janet code

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
pub fn read_and_enrich_host_config(
    host_file_path: &Utf8PathBuf,
    gurp_lib_path: &Option<Utf8PathBuf>,
    opts: &Opts,
) -> anyhow::Result<String> {
    let janet_host_config = std::fs::read_to_string(host_file_path)?;
    debug!(
        opts,
        "reader/enrich", "Reading host config from {}", host_file_path
    );
    let qualified_path = host_file_path.canonicalize_utf8()?;

    let host_config_dir = qualified_path
        .parent()
        .context(format!("cannot find parent of {}", host_file_path))?;

    let gurp_lib = match gurp_lib_path {
        Some(path) => &load_lib_from_disk(path, opts)?,
        None => GURP_LIB,
    };

    let mut ret = format!("(setdyn *syspath* \"{}\")\n\n", host_config_dir);
    ret.push_str(GURP_DEFAULTS);
    ret.push_str(
        gurp_lib
            .lines()
            .skip(1)
            .map(|s| format!("{}\n", s).to_owned())
            .collect::<String>()
            .as_str(),
    );
    ret.push('\n');
    ret.push_str(&janet_host_config);
    ret.push_str("\n(run-machine-configuration (machine-config))");
    Ok(ret)
}

pub fn format_janet_listing(janet_code: &str) -> String {
    let mut ret = "-".repeat(80);
    ret.push('\n');
    janet_code
        .lines()
        .enumerate()
        .for_each(|(i, l)| ret.push_str(&format!("{:>5} | {}\n", i + 1, l)));
    ret.push_str("-".repeat(80).as_str());
    ret.push('\n');
    ret
}

fn load_lib_from_disk(lib_path: &Utf8PathBuf, opts: &Opts) -> anyhow::Result<String> {
    if lib_path.exists() {
        debug!(
            opts,
            "reader/load", "Injecting gurp lib '{}' in user Janet", lib_path
        );
        Ok(std::fs::read_to_string(lib_path)?)
    } else {
        Err(anyhow!(format!("Could not find gurp lib at {}", lib_path)))
    }
}
