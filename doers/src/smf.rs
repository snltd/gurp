use anyhow::{Context, ensure};
use camino::Utf8PathBuf;
use common::constants::{
    MANIFEST_DIR, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, SVCCFG_BIN,
};
use common::info;
use common::types::{ApplyOpts, ApplySummary};
use serde::Deserialize;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use util::smf_builder::SmfDefinition;
use util::{smf_builder, svcs, xml};

const STATE_TRANSITION_INTERVAL: Duration = Duration::from_secs(1);
const STATE_TRANSITION_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpSmfEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(flatten)]
    pub desired_state: SmfDefinition,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpSmfRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpSmfEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let new_manifest = smf_builder::make_manifest(&self.desired_state)?;
        let manifest_path = &manifest_path(&self.desired_state.name);

        if svcs::exists(&self.desired_state.name)? {
            tracing::debug!("service exists: {}", &self.desired_state.name);

            if manifest_path.exists() {
                let current_manifest = fs::read_to_string(manifest_path)?;
                let desired_xml = xml::parse(&new_manifest)?;
                let current_xml = xml::parse(&current_manifest)?;
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

        if opts.output.dump_configs {
            println!(
                "{}",
                info::dump_config(&new_manifest, Some("SMF manifest"), &opts.output)
            );
        }

        if !opts.noop {
            fs::write(manifest_path, &new_manifest)
                .with_context(|| format!("failed writing SMF manifest to {manifest_path}"))?;
        }

        self.ensure_service(opts)
    }

    fn ensure_service(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let svc = &self.desired_state.name;

        if svcs::exists(svc)? {
            let current_state = svcs::current_state(svc)?;

            if current_state != "disabled" {
                svcs::set_state(svc, &current_state, "disabled", opts)?;
            }

            cmd_change_or_noop!(opts, SVCCFG_BIN, "delete", &svc)
                .with_context(|| format!("failed to delete svc {svc}"))?;
        }

        let manifest_path = manifest_path(&self.desired_state.name);

        cmd_change_or_noop!(opts, SVCCFG_BIN, "import", &manifest_path)
            .with_context(|| format!("failed to import from {manifest_path}"))
    }
}

impl GurpSmfRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let svc = &self.name;

        if svcs::exists(svc)? {
            let current_state = svcs::current_state(svc)?;

            if current_state != "disabled" {
                tracing::info!("svc: {svc} stopping service");

                if !opts.noop {
                    svcs::set_state(svc, &current_state, "disabled", opts)?;
                    self.wait_for_disabled_state()?;
                }
            }

            tracing::info!("svc: {svc} deleting service");

            let mut cmd = cmd!(SVCCFG_BIN, "delete", svc);

            if !opts.noop {
                run_cmd!(cmd).with_context(|| format!("failed to delete svc {svc}"))?;
            }

            let manifest_path = manifest_path(&self.name);

            if manifest_path.exists() {
                tracing::info!("svc: {} deleting manifest {}", &self.name, manifest_path);

                if !opts.noop {
                    fs::remove_file(&manifest_path)
                        .with_context(|| format!("failed to delete {manifest_path}"))?;
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

    fn wait_for_disabled_state(&self) -> anyhow::Result<()> {
        let elapsed = Duration::from_secs(0);
        loop {
            if svcs::current_state(&self.name)?.as_str() == "disabled" {
                return Ok(());
            }

            sleep(STATE_TRANSITION_INTERVAL);
            let elapsed = elapsed + STATE_TRANSITION_INTERVAL;

            ensure!(
                elapsed < STATE_TRANSITION_TIMEOUT,
                "Timed out waiting for {} be disabled",
                self.name
            );
        }
    }
}

fn manifest_path(svc_name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(MANIFEST_DIR).join(format!("gurp-{}.xml", svc_name.replace('/', "_")))
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;
    use tester::deserialized_example;
    use util::smf_builder::{
        PropertyStruct, PropertyValue, SmfDefinitionDependencySvc, SmfDefinitionExecMethod,
        SmfDefinitionExecMethodContext,
    };

    #[test]
    fn test_deserialize_smf_ensure_daemon_with_privs() {
        assert_eq!(
            GurpSmfEnsure {
                id: "/NO-ROLE/smf/example".to_owned(),
                desired_state: SmfDefinition {
                    name: "example".to_owned(),
                    duration: Some("child".to_owned()),
                    description: Some("Run example program".to_owned()),
                    fmri: "snltd/example".to_owned(),
                    default_enabled: true,
                    single_instance: true,
                    start_method: Some(SmfDefinitionExecMethod {
                        exec: "/app/method.sh".to_owned(),
                        timeout: 60,
                        context: Some(SmfDefinitionExecMethodContext {
                            user: "appuser".to_owned(),
                            group: Some("daemon".to_owned()),
                            privileges: Some("basic,!file_dac_search".to_owned()),
                            environment: None
                        })
                    }),
                    stop_method: Some(SmfDefinitionExecMethod {
                        exec: ":kill".to_owned(),
                        timeout: 10,
                        context: None
                    }),
                    refresh_method: None,
                    property_groups: Some(BTreeMap::from([
                        ("application".to_owned(), "application".to_owned()),
                        ("other_group".to_owned(), "framework".to_owned()),
                    ]),),
                    properties: Some(BTreeMap::from([
                        (
                            "application/port".to_owned(),
                            PropertyStruct {
                                value: PropertyValue::Int(8080),
                                prop_type: "integer".to_owned(),
                            }
                        ),
                        (
                            "application/ssl".to_owned(),
                            PropertyStruct {
                                value: PropertyValue::Bool(true),
                                prop_type: "boolean".to_owned(),
                            }
                        ),
                        (
                            "other_group/other_prop".to_owned(),
                            PropertyStruct {
                                value: PropertyValue::String("abc123".to_owned()),
                                prop_type: "astring".to_owned(),
                            }
                        )
                    ]),),
                    dependencies: Some(vec![
                        SmfDefinitionDependencySvc {
                            name: "dependency1".to_owned(),
                            fmri: "svc:/milestone/name-services:default".to_owned(),
                            restart_on: "none".to_owned(),
                            grouping: "require_all".to_owned(),
                            dep_type: "service".to_owned(),
                        },
                        SmfDefinitionDependencySvc {
                            name: "dependency2".to_owned(),
                            fmri: "svc:/system/pkgserv:default".to_owned(),
                            restart_on: "error".to_owned(),
                            grouping: "optional_all".to_owned(),
                            dep_type: "service".to_owned(),
                        },
                    ]),
                    dependents: None,
                }
            },
            deserialized_example("smf/ensure-daemon-with-privs.janet")
        );
    }

    #[test]
    fn test_generate_manifest() {
        let sut: GurpSmfEnsure = deserialized_example("smf/ensure-daemon-with-privs.janet");
        assert_eq!(
            indoc::indoc! { r#"
            <?xml version='1.0'?>
            <!DOCTYPE service_bundle SYSTEM '/usr/share/lib/xml/dtd/service_bundle.dtd.1'>
            <service_bundle type='manifest' name='example'>
              <service name='snltd/example' type='service' version='1'>
                <create_default_instance enabled='true'/>
                <single_instance/>
                <dependency name='physical' grouping='require_all' restart_on='none' type='service'>
                  <service_fmri value='svc:/network/physical:default'/>
                </dependency>
                <dependency name='fs-local' grouping='require_all' restart_on='none' type='service'>
                  <service_fmri value='svc:/system/filesystem/local'/>
                </dependency>
                <dependency name='dependency1' grouping='require_all' restart_on='none' type='service'>
                  <service_fmri value='svc:/milestone/name-services:default'/>
                </dependency>
                <dependency name='dependency2' grouping='optional_all' restart_on='error' type='service'>
                  <service_fmri value='svc:/system/pkgserv:default'/>
                </dependency>
                <exec_method name='start' type='method' exec='/app/method.sh' timeout_seconds='60'>
                  <method_context>
                    <method_credential user='appuser' group='daemon' privileges='basic,!file_dac_search'/>
                  </method_context>
                </exec_method>
                <exec_method name='stop' type='method' exec=':kill' timeout_seconds='10'/>
                <property_group name='startd' type='framework'>
                  <propval name='duration' type='astring' value='child'/>
                </property_group>
                <property_group name='application' type='application'>
                  <propval name='port' type='integer' value='8080'/>
                  <propval name='ssl' type='boolean' value='true'/>
                </property_group>
                <property_group name='other_group' type='framework'>
                  <propval name='other_prop' type='astring' value='"abc123"'/>
                </property_group>
                <stability value='Unstable'/>
                <template>
                  <common_name>
                    <loctext xml:lang='C'>
                      Run example program
                    </loctext>
                  </common_name>
                </template>
              </service>
            </service_bundle>
            "#},
            smf_builder::make_manifest(&sut.desired_state).unwrap()
        );
    }

    #[test]
    fn test_deserialize_smf_remove_service() {
        assert_eq!(
            GurpSmfRemove {
                id: "/NO-ROLE/smf/unwanted_service".to_owned(),
                name: "unwanted/service".to_owned(),
            },
            deserialized_example("smf/remove-service.janet")
        )
    }
}
