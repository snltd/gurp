use crate::debug;
use crate::doers::directory;
use crate::doers::types::Resource;
use crate::utils::janet_helpers as j;
use crate::utils::janet_helpers::JanetExt;
use crate::utils::types::Opts;
use anyhow::{Context, anyhow};
use camino::Utf8PathBuf;
use janetrs::{Janet, env::CFunOptions};
use janetrs::{JanetKeyword, TaggedJanet};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static OPTIONS: RefCell<Option<Opts>> = const { RefCell::new(None) };
}

#[derive(Debug)]
struct HostConfig {
    metadata: HostMetadata,
    resources: HostResources,
}

#[derive(Debug, PartialEq, Eq)]
struct HostMetadata {
    name: String,
}

type ResourceType = String;

// I keep changing my mind whether this should be a hash or a vec. We'll see what fits the
// problem best.
type HostResources = HashMap<ResourceType, Vec<Resource>>;

// Read the host file the user gives us, and execute it with our embedded Janet interpreter. This
// generates a big Janet Struct, with these keys:
//
//   :metadata   For now just has the name of the machine in it.
//   :resources  A Structwhose keys are resource types, e.g. :file or :service and whose value
//               are Arrays of those resources. Each resource is defined as a Janet Struct.
//
// The final function call passes this big Structto (run-machine-configuration), which is a Janet
// CFunction defined
// in this file, and mapped to machine_config_handler(). Because it's a Janet function it has to receive
// and return a janetrs::Janet. So it calls out to other functions to do all the actual work,
// returning success or failure.
//
// With vecs of properly typed resources, we can construct a dependency graph, check it
// looks valid, then apply the resources in order. Some resource types, say packages, can be
// grouped together
// into a single action.

pub fn dump_janet(janet_code: &String) -> String {
    let mut ret = "-".repeat(80);
    ret.push('\n');
    janet_code
        .lines()
        .enumerate()
        .for_each(|(i, l)| ret.push_str(&format!("{:>5} | {}\n", i + 1, l)));
    // janet_code,
    ret.push_str("-".repeat(80).as_str());
    ret.push('\n');
    ret
}
pub fn do_it(host_file: &Utf8PathBuf, opts: &Opts) -> anyhow::Result<bool> {
    OPTIONS.with(|o| {
        *o.borrow_mut() = Some(Opts {
            debug: opts.debug,
            noop: opts.noop,
            verbose: opts.verbose,
        });
    });

    let host_config = prep_host_config(host_file, opts)?;

    debug!(
        opts,
        "Janet host config follows:\n{}",
        dump_janet(&host_config)
    );

    let mut client = j::janet_client();

    client.add_c_fn(CFunOptions::new(
        c"run-machine-configuration",
        machine_config_handler_c,
    ));

    match client.run(host_config) {
        Ok(_) => Ok(true),
        Err(e) => {
            eprintln!("TODO: handle errors properly");
            Err(anyhow!(e))
        }
    }
}

fn janet_insert(host_file: &Utf8PathBuf) -> anyhow::Result<String> {
    // We can inject our own Janet code into what the user gives us, to reduce boilerplate.
    let host_config_dir = host_file
        .parent()
        .context(format!("cannot find parent of {}", host_file))?;

    // TODO at some point this will be baked into the binary, but over-rideable via a flag. It's
    // still very much in flux though, so we'll just read it off disk every time for now.
    //
    let gurp_lib_path =
        Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("janet_src/lib/gurp.janet");

    if !gurp_lib_path.exists() {
        panic!("Could not find gurp lib at {}", gurp_lib_path);
    }

    let gurp_lib = std::fs::read_to_string(gurp_lib_path)?;

    // Override the default include path, and drop the lib into the given file.
    Ok(format!(
        "(setdyn *syspath* \"{}\")\n\n{}\n",
        host_config_dir, gurp_lib,
    ))
}

fn prep_host_config(host_file_path: &Utf8PathBuf, opts: &Opts) -> anyhow::Result<String> {
    let janet_host_config = std::fs::read_to_string(host_file_path)?;
    debug!(opts, "Reading host config from {}", host_file_path);
    let qualified_path = host_file_path.canonicalize_utf8()?;
    Ok(format!(
        "{}\n{}",
        janet_insert(&qualified_path)?,
        janet_host_config
    ))
}

#[janetrs::janet_fn(arity(fix(1)))]
fn machine_config_handler(janet_config: &mut [Janet]) -> Janet {
    let config_elements = janet_config.len() as i32;

    if config_elements != 1 {
        eprintln!(
            "Expected single host configuration element, got {}",
            config_elements
        );
        return Janet::from(false);
    }

    let opts = OPTIONS
        .with(|o| o.borrow().clone())
        .expect("Failed to recover options");

    let janet_config = &janet_config[0].unwrap();

    debug!(opts, "Extracting Janet config struct");

    let config_struct = match janet_config {
        TaggedJanet::Struct(data) => data,
        other => {
            eprintln!("Expected Janet struct, got {}", other);
            return Janet::from(false);
        }
    };

    debug!(opts, "Extracting Janet metadata");

    let janet_metadata = match config_struct.get(Janet::from(":metadata")) {
        Some(md) => md,
        None => {
            eprintln!("Host config has no metadata");
            return Janet::from(false);
        }
    };

    debug!(opts, "Extracting Janet resources");

    let janet_resources = match config_struct.get(Janet::from(":resources")) {
        Some(md) => md,
        None => {
            eprintln!("Host config has no resources");
            return Janet::from(false);
        }
    };

    match janet_to_rust_config(janet_metadata, janet_resources, &opts) {
        Ok(_config) => Janet::from(true),
        Err(e) => {
            eprintln!("Failed to generate Rust config: {}", e);
            Janet::from(false)
        }
    }
}

fn janet_to_rust_metadata(janet_metadata: &Janet, opts: &Opts) -> anyhow::Result<HostMetadata> {
    debug!(opts, "Extracting Janet metadata struct");
    let rust_metadata = match janet_metadata.unwrap() {
        TaggedJanet::Struct(metadata) => {
            if let Some(name) = metadata.get(JanetKeyword::from("name")) {
                HostMetadata {
                    name: name.unwrap().to_string(),
                }
            } else {
                return Err(anyhow!("Did not find 'name' in host metadata"));
            }
        }
        _ => {
            return Err(anyhow!("Expected metadata to be Janet struct"));
        }
    };

    Ok(rust_metadata)
}

fn janet_to_rust_resources(janet_resources: &Janet, opts: &Opts) -> anyhow::Result<HostResources> {
    let mut ret = HashMap::new();

    debug!(opts, "Extracting Janet resource struct");
    let resources = janet_resources.extract_struct()?;
    debug!(opts, "Found {} resource types", resources.len());

    for (resource_type, resource_list) in resources {
        match resource_type.unwrap().to_string().as_str() {
            ":directories" => {
                let directories = directory::unpack_list(&resource_list)?;
                debug!(opts, "Found {} directory resource(s) ", directories.len());
                ret.insert("directories".to_owned(), directories);
            }
            other => {
                eprintln!("{} resources are not supported", other);
            }
        }
    }

    println!("{:?}", ret);

    Ok(ret)
}

fn janet_to_rust_config(
    janet_metadata: &Janet,
    janet_resources: &Janet,
    opts: &Opts,
) -> anyhow::Result<HostConfig> {
    Ok(HostConfig {
        metadata: janet_to_rust_metadata(janet_metadata, opts)?,
        resources: janet_to_rust_resources(janet_resources, opts)?,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::defopts;
    use janetrs::structs;

    #[test]
    fn test_janet_to_rust_metadata_good() {
        init_janet();

        let good_janet_metadata = Janet::wrap(structs! { ":name" => "test_name"});

        println!("{:?}", good_janet_metadata);

        assert_eq!(
            HostMetadata {
                name: "test_name".to_owned(),
            },
            janet_to_rust_metadata(&good_janet_metadata, &defopts()).unwrap()
        );

        let bad_janet_metadata = Janet::wrap(structs! { ":unknown" => "test_name" });
        assert!(janet_to_rust_metadata(&bad_janet_metadata, &defopts()).is_err());
    }

    fn init_janet() {
        unsafe {
            janetrs::lowlevel::janet_init();
        }
    }

    #[test]
    fn test_janet_to_rust_resources() {}
}
