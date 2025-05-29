use crate::doers::directory;
use crate::doers::types::{EnsureResources, RemoveResources};
use crate::doers::types::{HostConfig, HostMetadata, HostResources};
use crate::utils::janet_helpers::JanetExt;
use crate::utils::types::Opts;
use crate::{debug, verbose};
use anyhow::anyhow;
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
    let mut ret: EnsureResources = HashMap::new();

    for (resource_type, resource_list) in resources {
        let resource_type = resource_type.unwrap().to_string();
        let resource_list = resource_list.extract_array()?;

        #[rustfmt::skip]
        debug!(opts, "Found {} {} resources to ensure", resource_list.len(), resource_type);

        match resource_type.as_str() {
            ":directory" => {
                ret.insert(
                    "directory".to_owned(),
                    directory::unpack_ensure_list(&resource_list)?,
                );
            }
            other => eprintln!("{} resources are not implemented", other),
        }
    }

    Ok(ret)
}

fn extract_remove_resources(
    janet_resources: &Janet,
    opts: &Opts,
) -> anyhow::Result<RemoveResources> {
    let resources = janet_resources.extract_struct()?;
    let mut ret: RemoveResources = HashMap::new();

    for (resource_type, resource_list) in resources {
        let resource_type = resource_type.unwrap().to_string();
        let resource_list = resource_list.extract_array()?;

        #[rustfmt::skip]
        debug!(opts, "Found {} {} resource(s) to ensure", resource_list.len(), resource_type);

        match resource_type.as_str() {
            ":directory" => {
                ret.insert(
                    "directory".to_owned(),
                    directory::unpack_remove_list(&resource_list)?,
                );
            }
            other => eprintln!("{} resources are not implemented", other),
        }
    }

    Ok(ret)
}

fn extract_resources(janet_resources: &Janet, opts: &Opts) -> anyhow::Result<HostResources> {
    debug!(opts, "Extracting Janet enable/remove struct");
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
    debug!(opts, "Extracting Rust metadata struct");
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
