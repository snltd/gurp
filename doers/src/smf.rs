use crate::constants::MANIFEST_DIR;
use common::prelude::*;
use common::types::SmfDefinition;
use serde::Deserialize;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use util::{smf_builder, svcs};

// THINGS TO KNOW / THINGS TO DO.
// This writes SMF manifest files to disk, and imports them as needed. As of now, the directory
// is hardcoded.

const STATE_TRANSITION_INTERVAL: Duration = Duration::from_secs(1);
const STATE_TRANSITION_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Deserialize, Debug)]
pub struct GurpSmfEnsure {
    #[serde(rename = "_id")]
    pub id: String,
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
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let new_manifest = smf_builder::make_manifest(&self.desired_state);
        let manifest_path = &manifest_path(&self.desired_state.name);

        if svcs::exists(&self.desired_state.name)? {
            tracing::debug!("service exists: {}", &self.desired_state.name);

            if manifest_path.exists() {
                let current_manifest = fs::read_to_string(manifest_path)?;
                let desired_xml = helpers::parse_xml(&new_manifest)?;
                let current_xml = helpers::parse_xml(&current_manifest)?;
                if desired_xml == current_xml {
                    tracing::debug!("no change: {}", self.desired_state.name);
                    return Ok(ONE_RESOURCE_NO_CHANGE);
                }
            } else {
                tracing::debug!("creating manifest: {} ", manifest_path);
            }

            tracing::info!("change service: {}", self.desired_state.name);
        } else {
            tracing::info!("create service: {}", self.desired_state.name);
        };

        tracing::debug!("rewriting manifest: {}", manifest_path);

        return_if_noop!(opts);

        if opts.dump_config {
            println!(
                "{}",
                helpers::dump_config(&new_manifest, "SMF manifest", opts)
            );
        }

        fs::write(manifest_path, &new_manifest)?;
        self.ensure_service(opts)
    }

    fn ensure_service(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if svcs::exists(&self.desired_state.name)? {
            let current_state = svcs::current_state(&self.desired_state.name)?;

            if current_state != "disabled" {
                svcs::set_state(&self.desired_state.name, &current_state, "disabled")?;
            }

            let mut cmd = cmd!(SVCCFG_BIN, "delete", &self.desired_state.name);
            if !opts.noop {
                cmd.status()?;
            }
        }

        let mut cmd = cmd!(
            SVCCFG_BIN,
            "import",
            manifest_path(&self.desired_state.name).as_str()
        );
        return_if_noop!(opts);
        one_change_or_stderr!(cmd)
    }
}

impl GurpSmfRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if svcs::exists(&self.name)? {
            let current_state = svcs::current_state(&self.name)?;

            if current_state != "disabled" {
                tracing::info!("svc: {} stopping service", &self.name);
                if !opts.noop {
                    svcs::set_state(&self.name, &current_state, "disabled")?;
                    self.wait_for_disabled_state()?;
                }
            }

            tracing::info!("svc: {} deleting service", &self.name);

            let mut cmd = cmd!(SVCCFG_BIN, "delete", &self.name);

            if !opts.noop {
                cmd.status()?;
            }

            let manifest_path = manifest_path(&self.name);

            if manifest_path.exists() {
                tracing::info!("svc: {} deleting manifest {}", &self.name, manifest_path);
                return_if_noop!(opts);

                fs::remove_file(manifest_path)?;
            } else {
                tracing::debug!("svc: {} no manifest at {}", &self.name, manifest_path);
            }
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("svc: {} not present", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn wait_for_disabled_state(&self) -> anyhow::Result<()> {
        let elapsed = Duration::from_secs(0);
        loop {
            if svcs::current_state(&self.name)?.as_str() == "disabled" {
                return Ok(());
            }

            sleep(STATE_TRANSITION_INTERVAL);
            let elapsed = elapsed + STATE_TRANSITION_INTERVAL;

            if elapsed >= STATE_TRANSITION_TIMEOUT {
                bail!("Timed out waiting for {} be disabled", self.name)
            }
        }
    }
}

fn manifest_path(svc_name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(MANIFEST_DIR).join(format!("gurp-{}.xml", svc_name.replace('/', "_")))
}

#[cfg(test)]
mod test {
    use super::*;
    use common::types::{
        PropertyGroupMap, PropertyMap, PropertyStruct, PropertyValue, SmfDefinitionExecMethod,
        SmfDefinitionExecMethodContext,
    };
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;
    use tester::janet2json;

    #[test]
    fn test_smf_conversion() {
        let janet_desc = indoc! {r#"
            (smf/ensure "telegraf"
                :description "Run Telegraf agent"
                :fmri "sysdef/telegraf"
                :property-groups {:application "application"}
                :properties {:application/datadir "/data"}
                (smf-dependency "example"
                    :fmri "/example/service")
                (smf-method "start"
                    :exec "/opt/site/lib/smf/method/telegraf.sh"
                    :user "telegraf"
                    :group "daemon"
                    :privileges ["basic" "file_dac_search" "sys_admin" "proc_owner" "proc_zone"]
                    :environment {:LC_CTYPE "en_US.UTF-8"})
                (smf-method "refresh"
                    :exec ":kill -THAW"
                    :timeout 60))
            "#};

        let expected = SmfDefinition {
            name: "telegraf".to_owned(),
            duration: None,
            description: Some("Run Telegraf agent".to_owned()),
            fmri: "sysdef/telegraf".to_owned(),
            single_instance: true,
            default_enabled: true,
            property_groups: Some(PropertyGroupMap::from([(
                "application".to_owned(),
                "application".to_owned(),
            )])),
            dependencies: Some(vec![SmfDefinitionDependencySvc {
                name: "example".to_owned(),
                fmri: "/example/service".to_owned(),
                restart_on: "none".to_owned(),
                grouping: "require_all".to_owned(),
                dep_type: "service".to_owned(),
            }]),
            dependents: None,
            properties: Some(PropertyMap::from([(
                "application/datadir".to_owned(),
                PropertyStruct {
                    value: PropertyValue::String("/data".to_owned()),
                    prop_type: "astring".to_owned(),
                },
            )])),
            start_method: Some(SmfDefinitionExecMethod {
                exec: "/opt/site/lib/smf/method/telegraf.sh".to_owned(),
                timeout: 60,
                context: Some(SmfDefinitionExecMethodContext {
                    user: "telegraf".to_owned(),
                    group: Some("daemon".to_owned()),
                    privileges: Some(
                        "basic,file_dac_search,sys_admin,proc_owner,proc_zone".to_owned(),
                    ),
                    environment: Some(BTreeMap::from([(
                        "LC_CTYPE".to_owned(),
                        "en_US.UTF-8".to_owned(),
                    )])),
                }),
            }),
            stop_method: Some(SmfDefinitionExecMethod {
                exec: ":kill".to_owned(),
                timeout: 10,
                context: None,
            }),
            refresh_method: Some(SmfDefinitionExecMethod {
                exec: ":kill -THAW".to_owned(),
                timeout: 60,
                context: None,
            }),
        };

        let json_def = janet2json(janet_desc);
        let sut: SmfDefinition = serde_json::from_str(&json_def).unwrap();
        assert_eq!(expected, sut);
    }
}
