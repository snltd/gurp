use crate::doers::directory;
use crate::doers::types::Resource;
use crate::utils::janet_helpers as j;
use crate::utils::janet_helpers::{JanetExt, JanetTableExt};
use crate::utils::types::Opts;
use crate::{debug, verbose};
use anyhow::{Context, anyhow};
use camino::Utf8PathBuf;
use janetrs::{Janet, client::JanetClient, env::CFunOptions};
use janetrs::{JanetKeyword, JanetString, TaggedJanet};
use serde_json::Value;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static OPTIONS: RefCell<Option<Opts>> = RefCell::new(None);
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
type HostResources = HashMap<ResourceType, Vec<Resource>>;

// Read the host file the user gives us, and execute it with our embedded Janet interpreter. This
// generates a big Janet Table, with these keys:
//
//   :metadata   For now just has the name of the machine in it.
//   :resources  A Table whose keys are resource types, e.g. :file or :service and whose value
//               are Arrays of those resources. Each resource is defined as a Janet Table.
//
// The final function call passes this big Table to (run-machine-configuration), which is a Janet
// CFunction defined
// in this file, and mapped to machine_config_handler(). Because it's a Janet function it has to receive
// and return a janetrs::Janet. So it calls out to other functions to do all the actual work,
// returning success or failure.
//
// Janet is dynamically typed, and Rust is not. To reduce friction, I (at least for now) convert
// :resources into JSON.
//
// With vecs of properly typed resources, we can construct a dependency graph, check it
// looks valid, then apply the resources in order. Some resource types, say packages, can be
// grouped together
// into a single action.

pub fn do_it(host_file: &Utf8PathBuf, opts: &Opts) -> anyhow::Result<bool> {
    OPTIONS.with(|o| {
        *o.borrow_mut() = Some(Opts {
            debug: opts.debug,
            noop: opts.noop,
            module_dirs: opts.module_dirs.clone(),
            verbose: opts.verbose,
        });
    });

    let host_config = prep_host_config(host_file, opts)?;

    debug!(
        opts,
        "Janet host config follows:\n{}\n{}{}",
        "-".repeat(80),
        host_config,
        "-".repeat(80),
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

    // Override the default include path
    Ok(format!("(setdyn *syspath* \"{}\")", host_config_dir))
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

    debug!(opts, "Extracting Janet config table");

    let config_table = match janet_config {
        TaggedJanet::Table(table) => table,
        other => {
            eprintln!("Expected Janet table, got {}", other);
            return Janet::from(false);
        }
    };

    debug!(opts, "Extracting Janet metadata");

    let janet_metadata = match config_table.get(Janet::from(":metadata")) {
        Some(md) => md,
        None => {
            eprintln!("Host config has no metadata");
            return Janet::from(false);
        }
    };

    debug!(opts, "Extracting Janet resources");

    let janet_resources = match config_table.get(Janet::from(":resources")) {
        Some(md) => md,
        None => {
            eprintln!("Host config has no resources");
            return Janet::from(false);
        }
    };

    match janet_to_rust_config(janet_metadata, janet_resources, &opts) {
        Ok(config) => Janet::from(true),
        Err(e) => {
            eprintln!("Failed to generate Rust config: {}", e);
            Janet::from(false)
        }
    }
}

fn janet_to_rust_metadata(janet_metadata: &Janet, opts: &Opts) -> anyhow::Result<HostMetadata> {
    debug!(opts, "Extracting Janet metadata table");
    let rust_metadata = match janet_metadata.unwrap() {
        TaggedJanet::Table(table) => {
            if let Some(name) = table.get(JanetKeyword::from("name")) {
                HostMetadata {
                    name: name.unwrap().to_string(),
                }
            } else {
                return Err(anyhow!("Did not find 'name' in host metadata"));
            }
        }
        _ => {
            return Err(anyhow!("Expected metadata to be Janet table"));
        }
    };

    Ok(rust_metadata)
}

fn janet_to_rust_resources(janet_resources: &Janet, opts: &Opts) -> anyhow::Result<HostResources> {
    debug!(opts, "Extracting Janet resource table");
    let resources = janet_resources.extract_table()?;

    for (resource_type, resource_list) in resources {
        match resource_type.unwrap().to_string().as_str() {
            ":directories" => {
                let x = directory::unpack_list(&resource_list)?;

                println!("{:?}", x);
            }
            other => {
                eprintln!("{} resources are not supported", other);
            }
        }
    }
    todo!()
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
    use janetrs::JanetTable;

    #[test]
    fn test_janet_to_rust_metadata_good() {
        init_janet();

        let good_janet_metadata =
            Janet::wrap(JanetTable::builder(1).put("name", "test_name").finalize());

        assert_eq!(
            HostMetadata {
                name: "test_name".to_owned(),
            },
            janet_to_rust_metadata(&good_janet_metadata).unwrap()
        );

        let bad_janet_metadata = Janet::wrap(
            JanetTable::builder(1)
                .put("unknown", "test_name")
                .finalize(),
        );

        assert!(janet_to_rust_metadata(&bad_janet_metadata).is_err());
    }

    fn init_janet() {
        unsafe {
            janetrs::lowlevel::janet_init();
        }
    }
}
