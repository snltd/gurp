use crate::common::constants::ONE_RESOURCE_ONE_ERROR;
use crate::common::traits::{Apply, HasId};
use crate::common::types::{ApplyContext, ApplySummary, ChangedIds, HostConfig, Opts};
use crate::debug;
use crate::utils::{janet_helpers, parser, reader};
use anyhow::anyhow;
use camino::Utf8PathBuf;
use janetrs::{Janet, TaggedJanet, env::CFunOptions};
use std::cell::RefCell;
use std::collections::HashSet;

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
pub fn apply(
    host_file: &Utf8PathBuf,
    gurp_lib_path: &Option<Utf8PathBuf>,
    opts: &Opts,
) -> anyhow::Result<Janet> {
    tracing::debug!("Stashing opts object");

    OPTIONS.with(|o| {
        *o.borrow_mut() = Some(Opts {
            debug: opts.debug,
            noop: opts.noop,
            verbose: opts.verbose,
            no_colour: opts.no_colour,
        });
    });

    let host_config = reader::read_and_enrich_host_config(host_file, gurp_lib_path, opts, false)?;

    debug!(
        opts,
        "host-apply",
        "Janet host config follows:\n{}",
        reader::format_janet_listing(&host_config)
    );

    let mut client = janet_helpers::janet_client();

    // client.add_c_fn(CFunOptions::new(
    //     c"run-machine-configuration",
    //     machine_config_handler_c,
    // ));

    // Compile the Janet and kick off the machine configuration by calling the handler defined above.
    let json = client.run(host_config);
    println!("{:?}", json.unwrap().unwrap());
    //     // Here we return from doing all the work of configuring the host
    //     Ok(summary) => Ok(summary),
    //     Err(e) => {
    //         tracing::debug!("returning err from host apply");
    //         Err(anyhow!(e))
    //     }
    // }
    todo!()
}
// #[janetrs::janet_fn(arity(fix(1)))]
// fn machine_config_handler(janet_config: &mut [Janet]) -> Janet {
//     janet_config.to_owned()
// }

// #[janetrs::janet_fn(arity(fix(1)))]
// fn _machine_config_handler(janet_config: &mut [Janet]) -> Janet {
//     let config_elements = janet_config.len() as i32;

//     if config_elements != 1 {
//         tracing::error!(
//             "expected single host configuration element, got {}",
//             config_elements
//         );
//         return Janet::from(false);
//     }

//     let opts = OPTIONS
//         .with(|o| o.borrow().clone())
//         .expect("Failed to recover options");

//     debug!(
//         opts,
//         "host/ensure-remove",
//         "Parsed Janet config follows:\n{}",
//         janet_helpers::pretty_janet(&janet_config[0], 4)
//     );

//     let janet_config = &janet_config[0].unwrap();

//     tracing::debug!("extracting Janet config struct");

//     let config_struct = match janet_config {
//         TaggedJanet::Struct(data) => data,
//         other => {
//             tracing::error!("expected Janet struct, got {}", other);
//             return Janet::from(false);
//         }
//     };

//     tracing::debug!("extracting Janet metadata");

//     let janet_metadata = match config_struct.get(Janet::from(":metadata")) {
//         Some(md) => md,
//         None => {
//             tracing::error!("host config has no metadata");
//             return Janet::from(false);
//         }
//     };

//     tracing::debug!("extracting Janet metadata");

//     let janet_resources = match config_struct.get(Janet::from(":resources")) {
//         Some(md) => md,
//         None => {
//             tracing::error!("host config has no resources");
//             return Janet::from(false);
//         }
//     };

//     match parser::parse_config(janet_metadata, janet_resources, &opts) {
//         Ok(config) => match ensure_and_remove(&config, &opts) {
//             // You'd think a JanetAbstract would be the right thing here, but it gets very
//             // complicated. The struct is simple enough to do this.
//             Ok(summary) => janet_helpers::wrap_summary(&summary),
//             Err(e) => {
//                 tracing::error!("error in ensure_and_remove: {}", e);
//                 Janet::from(false)
//             }
//         },
//         Err(e) => {
//             tracing::error!("error generating Rust config: {}", e);
//             Janet::from(false)
//         }
//     }
// }

fn ensure_and_remove(config: &HostConfig, opts: &Opts) -> anyhow::Result<ApplySummary> {
    tracing::info!("Configuring host: {}", config.metadata.name);

    let ensure_order = &[
        "zfs",
        "pkg",
        "gem",
        "user",
        "cron",
        "directory",
        "file",
        "symlink",
        "file-line",
        "smf",
        "misc",
    ];

    let mut summary_total = ApplySummary {
        resources: 0,
        changes: 0,
        errors: 0,
    };

    let mut changed_ids: ChangedIds = HashSet::new();

    let initial_context = ApplyContext {
        changed_ids: HashSet::new(),
    };

    for resource_type in ensure_order {
        if let Some(resources) = config.resources.ensure.get(*resource_type) {
            for resource in resources {
                match resource.apply(&initial_context, opts) {
                    Ok(summary) => {
                        summary_total = summary_total + summary;
                        if summary.changes > 0 {
                            changed_ids.insert(resource.id());
                        }
                    }
                    Err(e) => {
                        tracing::error!("could not ensure {}: {}", resource.id(), e);
                        summary_total = summary_total + ONE_RESOURCE_ONE_ERROR;
                    }
                }
            }
        } else {
            tracing::debug!("{}: no resources to ensure", resource_type);
        }
    }

    let remove_order = &[
        "file-line",
        "symlink",
        "file",
        "directory",
        "cron",
        "user",
        "smf",
        "gem",
        "pkg",
        "zfs",
    ];

    for resource_type in remove_order {
        if let Some(resources) = config.resources.remove.get(*resource_type) {
            for resource in resources {
                match resource.apply(&initial_context, opts) {
                    Ok(summary) => summary_total = summary_total + summary,
                    Err(e) => {
                        tracing::error!("could not remove {}: {}", resource.id(), e);
                        summary_total = summary_total + ONE_RESOURCE_ONE_ERROR;
                    }
                }
            }
        } else {
            tracing::debug!("{}: no resources to remove", resource_type);
        }
    }

    let svc_context = ApplyContext { changed_ids };

    // We deal with services last, and differently.
    //
    if let Some(svcs) = config.resources.ensure.get("svc") {
        for svc in svcs {
            match svc.apply(&svc_context, opts) {
                Ok(summary) => summary_total = summary_total + summary,
                Err(e) => {
                    tracing::error!("could not ensure {}: {}", svc.id(), e);
                    summary_total = summary_total + ONE_RESOURCE_ONE_ERROR;
                }
            }
        }
    } else {
        tracing::debug!("svc: no resources to ensure");
    }

    Ok(summary_total)
}
