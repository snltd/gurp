use crate::common::constants::JSON_LIB;
use crate::common::types::{ApplyContext, ApplySummary, ChangedIds, HostConfig, Opts};
use crate::debug;
use crate::utils::{janet_helpers, reader};
// use anyhow::anyhow;
use anyhow::bail;
use camino::Utf8PathBuf;
use janetrs::TaggedJanet;
use serde_json::Value;
// use std::cell::RefCell;
use std::collections::HashSet;

// thread_local! {
//     static OPTIONS: RefCell<Option<Opts>> = const { RefCell::new(None) };
// }

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

fn pretty_print_json_str(json_str: &str) -> anyhow::Result<()> {
    let value: Value = serde_json::from_str(json_str)?;
    let pretty = serde_json::to_string_pretty(&value)?;
    println!("{}", pretty);
    Ok(())
}

// This is the entry point from main
pub fn apply(
    host_file: &Utf8PathBuf,
    gurp_lib_path: &Option<Utf8PathBuf>,
    opts: &Opts,
) -> anyhow::Result<ApplySummary> {
    let host_config = reader::read_and_enrich_host_config(host_file, gurp_lib_path, opts, false)?;

    debug!(
        opts,
        "host-apply",
        "Janet host config follows:\n{}",
        reader::format_janet_listing(&host_config)
    );

    let client = janet_helpers::janet_client();

    let json_wrapped_host_config =
        format!("{}\n{}\n(encode (machine-config))", JSON_LIB, host_config,);

    let json_buffer = client.run(json_wrapped_host_config)?;

    let json = match json_buffer.unwrap() {
        TaggedJanet::Buffer(buf) => buf.to_string(),
        other => bail!("expected Janet::Buffer, got {}", other),
    };

    tracing::debug!("Janet returned {} char JSON buffer", json.len());
    tracing::debug!("Unpacking JSON into HostConfig");

    // println!("{}", json);

    pretty_print_json_str(&json)?;

    let host_config: HostConfig = serde_json::from_str(&json)?;

    println!("{:#?}", host_config);

    ensure_and_remove(&host_config, opts)
}

// #[janetrs::janet_fn(arity(fix(1)))]
// fn machine_config_handler(janet_config: &mut [Janet]) -> Janet {
//     // println!("{}", janet_helpers::pretty_janet(&janet_config[0], 4));
//     janet_config[0]
//     // janet_config.to_owned()[0]
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

macro_rules! apply_resources {
    ($summary_total:ident, $changed_ids:ident, $resources:expr, $ctx:expr, $opts:expr) => {
        for resource in $resources {
            let summary = resource.apply($ctx, $opts)?;
            $summary_total = $summary_total + summary;
            if summary.changes > 0 {
                $changed_ids.insert(resource.id.clone());
            }
        }
    };
}

fn ensure_and_remove(config: &HostConfig, opts: &Opts) -> anyhow::Result<ApplySummary> {
    tracing::info!("Configuring host: {}", config.metadata.name);
    let ensure = &config.resources.ensure;
    let remove = &config.resources.remove;

    let mut summary_total = ApplySummary::default();

    //     let ensure_order = &[
    //         "zfs",
    //         // "pkg",
    //         // "gem",
    //         // "user",
    //         // "cron",
    //         // "directory",
    //         // "file",
    //         // "symlink",
    //         // "file-line",
    //         // "smf",
    //         // "misc",
    //     ];

    // let mut summary_total = ApplySummary.default();

    let mut changed_ids: ChangedIds = HashSet::new();

    let initial_context = ApplyContext {
        changed_ids: HashSet::new(),
    };

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.zfs,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.user,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.cron,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.symlink,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.file_line,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.smf,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.misc,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.svc,
        &initial_context,
        opts
    );

    // for resource_type in ensure_order {
    //     if let Some(resources) = config.resources.ensure.get(*resource_type) {
    //         for resource in resources {
    //             match resource.apply(&initial_context, opts) {
    //                 Ok(summary) => {
    //                     summary_total = summary_total + summary;
    //                     if summary.changes > 0 {
    //                         changed_ids.insert(resource.id());
    //                     }
    //                 }
    //                 Err(e) => {
    //                     tracing::error!("could not ensure {}: {}", resource.id(), e);
    //                     summary_total = summary_total + ONE_RESOURCE_ONE_ERROR;
    //                 }
    //             }
    //         }
    //     } else {
    //         tracing::debug!("{}: no resources to ensure", resource_type);
    //     }
    // }

    /*
    let remove_order = &[
        // "file-line",
        // "symlink",
        // "file",
        // "directory",
        // "cron",
        // "user",
        // "smf",
        // "gem",
        // "pkg",
        "zfs",
    ];
    */
    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.file_line,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &remove.symlink,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.cron,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &remove.user,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &ensure.smf,
        &initial_context,
        opts
    );

    apply_resources!(
        summary_total,
        changed_ids,
        &remove.zfs,
        &initial_context,
        opts
    );

    // for resource_type in remove_order {
    //     if let Some(resources) = config.resources.remove.get(*resource_type) {
    //         for resource in resources {
    //             match resource.apply(&initial_context, opts) {
    //                 Ok(summary) => summary_total = summary_total + summary,
    //                 Err(e) => {
    //                     tracing::error!("could not remove {}: {}", resource.id(), e);
    //                     summary_total = summary_total + ONE_RESOURCE_ONE_ERROR;
    //                 }
    //             }
    //         }
    //     } else {
    //         tracing::debug!("{}: no resources to remove", resource_type);
    //     }
    // }

    // let svc_context = ApplyContext { changed_ids };

    // We deal with services last, and differently.
    //
    // if let Some(svcs) = config.resources.ensure.get("svc") {
    //     for svc in svcs {
    //         match svc.apply(&svc_context, opts) {
    //             Ok(summary) => summary_total = summary_total + summary,
    //             Err(e) => {
    //                 tracing::error!("could not ensure {}: {}", svc.id(), e);
    //                 summary_total = summary_total + ONE_RESOURCE_ONE_ERROR;
    //             }
    //         }
    //     }
    // } else {
    //     tracing::debug!("svc: no resources to ensure");
    // }

    Ok(summary_total)
}
