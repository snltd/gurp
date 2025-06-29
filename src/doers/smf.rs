use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::svcs;
use crate::common::types::{ApplySummary, Opts, SmfDefinition};
use crate::debug;
use crate::utils::helpers;
use crate::utils::smf_builder;
use camino::Utf8PathBuf;
use serde::Deserialize;
use std::fs;

const MANIFEST_DIR: &str = "/opt/site/lib/smf/manifest";

// THINGS TO KNOW / THINGS TO DO.
// This writes SMF manifest files to disk, and imports them as needed. As of now, the directory
// is hardcoded.

#[derive(Deserialize, Debug)]
pub struct GurpSmfEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "svc-name")]
    pub name: String,
    #[serde(flatten)]
    pub desired_state: SmfDefinition,
}

#[derive(Deserialize, Debug)]
pub struct GurpSmfRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpSmfEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let new_manifest = smf_builder::make_manifest(&self.desired_state);
        let manifest_path = &manifest_path(&self.name);

        if svcs::exists(&self.name)? {
            tracing::debug!("service exists: {}", &self.name);

            if manifest_path.exists() {
                let current_manifest = fs::read_to_string(manifest_path)?;
                let desired_xml = helpers::parse_xml(&new_manifest)?;
                let current_xml = helpers::parse_xml(&current_manifest)?;
                if desired_xml == current_xml {
                    tracing::info!("no change: {}", self.name);
                    return Ok(ONE_RESOURCE_NO_CHANGE);
                }
            } else {
                tracing::debug!("creating manifest: {} ", manifest_path);
            }

            tracing::info!("change service: {}", self.name);
        } else {
            tracing::info!("create service: {}", self.name);
        };

        tracing::debug!("rewriting manifest: {}", manifest_path);

        if opts.noop {
            Ok(ONE_RESOURCE_NOOP)
        } else {
            debug!(opts, "doer/smf", "SMF manifest follows:\n{}", new_manifest);
            fs::write(manifest_path, &new_manifest)?;
            self.ensure_service()
        }
    }

    fn ensure_service(&self) -> anyhow::Result<ApplySummary> {
        if svcs::exists(&self.name)? {
            let current_state = svcs::current_state(&self.name)?;

            if current_state != "disabled" {
                svcs::set_state(&self.name, &current_state, "disabled")?;
            }

            svcs::run_svccfg("delete", &self.name)?;
        }

        svcs::run_svccfg("import", manifest_path(&self.name).as_str())?;

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
}

impl GurpSmfRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if svcs::exists(&self.name)? {
            let current_state = svcs::current_state(&self.name)?;

            if current_state != "disabled" {
                tracing::info!("svc: {} stopping service", &self.name);
                if !opts.noop {
                    svcs::set_state(&self.name, &current_state, "disabled")?;
                }
            }

            tracing::info!("svc: {} deleting service", &self.name);

            if !opts.noop {
                svcs::run_svccfg("delete", &self.name)?;
            }

            let manifest_path = manifest_path(&self.name);
            if manifest_path.exists() {
                tracing::info!("svc: {} deleting manifest {}", &self.name, manifest_path);

                if opts.noop {
                    return Ok(ONE_RESOURCE_NOOP);
                } else {
                    fs::remove_file(manifest_path)?;
                }
            } else {
                tracing::debug!("svc: {} no manifest at {}", &self.name, manifest_path);
            }
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("svc: {} not present", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn manifest_path(svc_name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(MANIFEST_DIR).join(format!("gurp-{svc_name}.xml"))
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::common::types::{SmfDefinitionExecMethod, SmfDefinitionExecMethodContext};
//     use crate::test_utils::spec_helper::init_janet;
//     use janetrs::structs;
//     use pretty_assertions::assert_eq;

//     #[test]
//     fn test_unpack_ensure_file() {
//         init_janet();

//         let start_context = structs! {
//             ":user" => "telegraf",
//             ":group" => "daemon",
//             ":privileges" => "basic,file_dac_search,sys_admin,proc_owner,proc_zone",
//         };

//         let start_method = structs! {
//             ":exec" => "/opt/site/lib/smf/method/telegraf.sh",
//             ":timeout" => 60,
//             ":context" => start_context,
//         };

//         let stop_method = structs! {
//             ":exec" => ":kill",
//             ":timeout" => 10,
//         };

//         let refresh_method = structs! {
//             ":exec" => ":kill -THAW",
//             ":timeout" => 60,
//         };

//         let test_ensure = structs! {
//             ":_id" => "/test-role/smf/test-smf",
//             ":action" => ":ensure",
//             ":name" => "export",
//             ":description" => "Run Telegraf agent",
//             ":fmri" => "sysdef/telegraf",
//             ":default-enabled" => true,
//             ":single-instance" => true,
//             ":start-method" => start_method,
//             ":stop-method" => stop_method,
//             ":refresh-method" => refresh_method,
//         };

//         let test_svc = SmfDefinition {
//             name: "export".to_owned(),
//             description: "Run Telegraf agent".to_owned(),
//             fmri: "sysdef/telegraf".to_owned(),
//             single_instance: true,
//             default_enabled: true,
//             start_method: Some(SmfDefinitionExecMethod {
//                 exec: "/opt/site/lib/smf/method/telegraf.sh".to_owned(),
//                 timeout: 60,
//                 context: Some(SmfDefinitionExecMethodContext {
//                     user: "telegraf".to_owned(),
//                     group: Some("daemon".to_owned()),
//                     privileges: Some(
//                         "basic,file_dac_search,sys_admin,proc_owner,proc_zone".to_owned(),
//                     ),
//                 }),
//             }),
//             stop_method: Some(SmfDefinitionExecMethod {
//                 exec: ":kill".to_owned(),
//                 timeout: 10,
//                 context: None,
//             }),
//             refresh_method: Some(SmfDefinitionExecMethod {
//                 exec: ":kill -THAW".to_owned(),
//                 timeout: 60,
//                 context: None,
//             }),
//         };

//         assert_eq!(test_svc, unpack_smf(&test_ensure).unwrap());
//     }
// }
