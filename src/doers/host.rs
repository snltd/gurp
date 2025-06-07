use crate::common::constants::ONE_RESOURCE_ONE_ERROR;
use crate::common::traits::{Apply, HasId};
use crate::common::types::{ApplySummary, HostConfig, Opts};
use crate::utils::{janet_helpers, parser, reader};
use crate::{debug, error, info};
use anyhow::anyhow;
use camino::Utf8PathBuf;
use colored::Colorize;
use janetrs::{Janet, TaggedJanet, env::CFunOptions};
use std::cell::RefCell;

thread_local! {
    static OPTIONS: RefCell<Option<Opts>> = const { RefCell::new(None) };
}

// Read the host file the user gives us, and execute it with our embedded Janet interpreter. This
// generates a big Janet Struct, with these keys:
//
//   :metadata   For now just has the name of the machine in it.
//   :resources  A Structwhose keys are resource types, e.g. :file or :service and whose value
//               are Arrays of those resources. Each resource is defined as a Janet Struct.
//
// The final function call passes this big Struct to (run-machine-configuration), which is a Janet
// CFunction defined
// in this file, and mapped to machine_config_handler(). Because it's a Janet function it has to receive
// and return a janetrs::Janet. So it calls out to other functions to do all the actual work,
// returning success or failure.
//
// With vecs of properly typed resources, we can construct a dependency graph, check it
// looks valid, then apply the resources in order. Some resource types, say packages, can be
// grouped together
// into a single action.

// This is the entry point from main
pub fn apply(host_file: &Utf8PathBuf, opts: &Opts) -> anyhow::Result<Janet> {
    debug!(opts, "host-apply", "Stashing opts object");

    OPTIONS.with(|o| {
        *o.borrow_mut() = Some(Opts {
            debug: opts.debug,
            noop: opts.noop,
            verbose: opts.verbose,
            gurp_lib_path: opts.gurp_lib_path.clone(),
        });
    });

    let host_config = reader::read_and_enrich_host_config(host_file, opts)?;

    debug!(
        opts,
        "host-apply",
        "Janet host config follows:\n{}",
        reader::format_janet_listing(&host_config)
    );

    let mut client = janet_helpers::janet_client(opts);

    client.add_c_fn(CFunOptions::new(
        c"run-machine-configuration",
        machine_config_handler_c,
    ));

    // Compile the Janet and kick off the machine configuration by calling the handler defined above.
    match client.run(host_config) {
        // Here we return from doing all the work of configuring the host
        Ok(summary) => Ok(summary),
        Err(e) => {
            println!("Returning ERR from host apply");
            Err(anyhow!(e))
        }
    }
}

#[janetrs::janet_fn(arity(fix(1)))]
fn machine_config_handler(janet_config: &mut [Janet]) -> Janet {
    let config_elements = janet_config.len() as i32;

    if config_elements != 1 {
        error!(
            opts,
            "handler", "expected single host configuration element, got {}", config_elements
        );
        return Janet::from(false);
    }

    let opts = OPTIONS
        .with(|o| o.borrow().clone())
        .expect("Failed to recover options");

    let janet_config = &janet_config[0].unwrap();

    debug!(opts, "handler", "extracting Janet config struct");

    let config_struct = match janet_config {
        TaggedJanet::Struct(data) => data,
        other => {
            error!(opts, "handler", "expected Janet struct, got {}", other);
            return Janet::from(false);
        }
    };

    debug!(opts, "handler", "extracting Janet metadata");

    let janet_metadata = match config_struct.get(Janet::from(":metadata")) {
        Some(md) => md,
        None => {
            error!(opts, "handler", "host config has no metadata");
            return Janet::from(false);
        }
    };

    debug!(opts, "handler", "extracting Janet resources");

    let janet_resources = match config_struct.get(Janet::from(":resources")) {
        Some(md) => md,
        None => {
            error!(opts, "handler", "host config has no resources");
            return Janet::from(false);
        }
    };

    match parser::parse_config(janet_metadata, janet_resources, &opts) {
        Ok(config) => match ensure_and_remove(&config, &opts) {
            // You'd think a JanetAbstract would be the right thing here, but it gets very
            // complicated. The struct is simple enough to do this.
            Ok(summary) => janet_helpers::wrap_summary(&summary),
            Err(e) => {
                error!(opts, "handler", "running ensure_and_remove: {}", e);
                Janet::from(false)
            }
        },
        Err(e) => {
            error!(opts, "handler", "failed to generate Rust config: {}", e);
            Janet::from(false)
        }
    }
}

fn ensure_and_remove(config: &HostConfig, opts: &Opts) -> anyhow::Result<ApplySummary> {
    info!(opts, "Configuring host '{}'", config.metadata.name);

    let ensure_order = &["pkg", "user", "directory", "file"];

    let mut summary_total = ApplySummary {
        resources: 0,
        changes: 0,
        errors: 0,
    };

    for resource_type in ensure_order {
        if let Some(resources) = config.resources.ensure.get(*resource_type) {
            for resource in resources {
                match resource.apply(opts) {
                    Ok(summary) => summary_total = summary_total + summary,
                    Err(e) => {
                        error!(
                            opts,
                            "ensure/remove",
                            "could not ensure {}: {}",
                            resource.id(),
                            e
                        );
                        summary_total = summary_total + ONE_RESOURCE_ONE_ERROR;
                    }
                }
            }
        } else {
            debug!(
                opts,
                "ensure/remove", "No {} resources to ensure", resource_type
            );
        }
    }

    let remove_order = &["file", "directory", "user", "pkg"];

    for resource_type in remove_order {
        if let Some(resources) = config.resources.remove.get(*resource_type) {
            for resource in resources {
                match resource.apply(opts) {
                    Ok(summary) => summary_total = summary_total + summary,
                    Err(e) => {
                        error!(
                            opts,
                            "ensure/remove", "could not remove {}: {}", resource_type, e
                        );
                        summary_total = summary_total + ONE_RESOURCE_ONE_ERROR;
                    }
                }
            }
        } else {
            debug!(
                opts,
                "ensure/remove", "No {} resources to remove", resource_type
            );
        }
    }

    Ok(summary_total)
}
