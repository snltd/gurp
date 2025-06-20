use crate::common::types::{
    EnsureResources, HostConfig, HostMetadata, HostResources, Opts, RemoveResources,
};
use crate::doers::{cron, directory, file, file_line, misc, pkg, smf, svc, user, zfs};
use crate::utils::janet_helpers::JanetExt;
use crate::{debug, verbose, warn};
use anyhow::anyhow;
use colored::Colorize;
use janetrs::{Janet, JanetKeyword, TaggedJanet};
use std::collections::HashMap;

// Where we turn Janet data into Rust structs

pub fn parse_config(
    janet_metadata: &Janet,
    janet_resources: &Janet,
    opts: &Opts,
) -> anyhow::Result<HostConfig> {
    Ok(HostConfig {
        metadata: extract_metadata(janet_metadata, opts)?,
        resources: extract_resources(janet_resources, opts)?,
    })
}

fn extract_ensure_resources(
    janet_resources: &Janet,
    opts: &Opts,
) -> anyhow::Result<EnsureResources> {
    let resources = janet_resources.extract_struct()?;
    let mut ret = HashMap::new();

    for (resource_type, resource_list) in resources {
        let resource_type = resource_type.unwrap().to_string();
        let resource_list = resource_list.extract_array()?;

        debug!(
            opts,
            "parser/extract",
            "Found {} {} resource(s) to ensure",
            resource_list.len(),
            resource_type
        );

        match resource_type.as_str() {
            ":pkg" => {
                ret.insert(
                    "pkg".to_owned(),
                    pkg::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            ":zfs" => {
                ret.insert(
                    "zfs".to_owned(),
                    zfs::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            ":directory" => {
                ret.insert(
                    "directory".to_owned(),
                    directory::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            ":file" => {
                ret.insert(
                    "file".to_owned(),
                    file::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            ":cron" => {
                ret.insert(
                    "cron".to_owned(),
                    cron::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            ":user" => {
                ret.insert(
                    "user".to_owned(),
                    user::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            ":file-line" => {
                ret.insert(
                    "file-line".to_owned(),
                    file_line::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            ":misc" => {
                ret.insert(
                    "misc".to_owned(),
                    misc::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            ":smf" => {
                ret.insert(
                    "smf".to_owned(),
                    smf::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            ":svc" => {
                ret.insert(
                    "svc".to_owned(),
                    svc::unpack_ensure_list(&resource_list, opts)?,
                );
            }
            other => warn!(
                opts,
                "parser/extract/ensure",
                "'{}' resources are not implemented",
                other.replacen(':', "", 1)
            ),
        }
    }

    Ok(ret)
}

fn extract_remove_resources(
    janet_resources: &Janet,
    opts: &Opts,
) -> anyhow::Result<RemoveResources> {
    let resources = janet_resources.extract_struct()?;
    let mut ret = HashMap::new();

    for (resource_type, resource_list) in resources {
        let resource_type = resource_type.unwrap().to_string();
        let resource_list = resource_list.extract_array()?;

        debug!(
            opts,
            "parser/extract",
            "Found {} {} resource(s) to remove",
            resource_list.len(),
            resource_type
        );

        match resource_type.as_str() {
            ":directory" => {
                ret.insert(
                    "directory".to_owned(),
                    directory::unpack_remove_list(&resource_list, opts)?,
                );
            }
            ":file" => {
                ret.insert(
                    "file".to_owned(),
                    file::unpack_remove_list(&resource_list, opts)?,
                );
            }
            ":zfs" => {
                ret.insert(
                    "zfs".to_owned(),
                    zfs::unpack_remove_list(&resource_list, opts)?,
                );
            }
            ":user" => {
                ret.insert(
                    "user".to_owned(),
                    user::unpack_remove_list(&resource_list, opts)?,
                );
            }
            ":pkg" => {
                ret.insert(
                    "pkg".to_owned(),
                    pkg::unpack_remove_list(&resource_list, opts)?,
                );
            }
            ":cron" => {
                ret.insert(
                    "cron".to_owned(),
                    cron::unpack_remove_list(&resource_list, opts)?,
                );
            }
            ":smf" => {
                ret.insert(
                    "smf".to_owned(),
                    smf::unpack_remove_list(&resource_list, opts)?,
                );
            }
            ":file-line" => {
                ret.insert(
                    "file-line".to_owned(),
                    file_line::unpack_remove_list(&resource_list, opts)?,
                );
            }
            other => warn!(
                opts,
                "parser/extract/remove",
                "'{}' resources are not implemented",
                other.replacen(':', "", 1)
            ),
        }
    }

    Ok(ret)
}

fn extract_resources(janet_resources: &Janet, opts: &Opts) -> anyhow::Result<HostResources> {
    debug!(opts, "parser/extract", "Extracting ensure/remove struct");
    let resource_actions = janet_resources.extract_struct()?;

    let ensure_resources = match resource_actions.get(JanetKeyword::from("ensure")) {
        Some(resources) => extract_ensure_resources(resources, opts)?,
        None => {
            verbose!(opts, "No ensure resources found");
            HashMap::new()
        }
    };

    let remove_resources = match resource_actions.get(JanetKeyword::from("remove")) {
        Some(resources) => extract_remove_resources(resources, opts)?,
        None => {
            verbose!(opts, "No remove resources found");
            HashMap::new()
        }
    };

    Ok(HostResources {
        ensure: ensure_resources,
        remove: remove_resources,
    })
}

fn extract_metadata(janet_metadata: &Janet, opts: &Opts) -> anyhow::Result<HostMetadata> {
    debug!(opts, "parser/extract", "Extracting metadata");
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::{defopts, init_janet};
    use janetrs::structs;

    #[test]
    fn test_janet_to_rust_metadata_good() {
        init_janet();

        let good_janet_metadata = Janet::wrap(structs! { ":name" => "test_name"});

        assert_eq!(
            HostMetadata {
                name: "test_name".to_owned(),
            },
            extract_metadata(&good_janet_metadata, &defopts()).unwrap()
        );

        let bad_janet_metadata = Janet::wrap(structs! { ":unknown" => "test_name" });
        assert!(extract_metadata(&bad_janet_metadata, &defopts()).is_err());
    }
}
