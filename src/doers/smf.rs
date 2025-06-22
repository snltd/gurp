use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::svcs;
use crate::common::traits::Apply;
use crate::common::types::{
    Action, ApplyContext, ApplySummary, Opts, Resource, SmfDefinition, SmfDefinitionExecMethod,
    SmfDefinitionExecMethodContext,
};
use crate::debug;
use crate::utils::helpers;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use crate::utils::smf_builder;
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray, JanetStruct};
use paste::paste;
use std::fs;

const MANIFEST_DIR: &str = "/opt/site/lib/smf/manifest";

// THINGS TO KNOW / THINGS TO DO.
// This writes SMF manifest files to disk, and imports them as needed. As of now, the directory
// is hardcoded.

#[derive(Debug)]
pub struct GurpSmf {
    pub action: Action,
    pub id: String,
    pub name: String,
    pub desired_state: Option<SmfDefinition>,
    pub manifest_path: Utf8PathBuf,
}

impl TryFrom<&Janet> for GurpSmf {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let action = janet_helpers::action_as_enum(&data)?;
        let name = data.get_field_string("name")?;

        let state = match action {
            Action::Ensure => Some(unpack_smf(&data)?),
            Action::Remove => None,
        };

        Ok(GurpSmf {
            action,
            manifest_path: Utf8PathBuf::from(MANIFEST_DIR).join(format!("gurp-{}.xml", name)),
            name,
            id: data.get_field_string("_id")?,
            desired_state: state,
        })
    }
}

fn unpack_smf_method(
    data: &JanetStruct,
    method: &str,
) -> anyhow::Result<Option<SmfDefinitionExecMethod>> {
    Ok(data.get_field_struct_opt(method).and_then(|m| {
        Some(SmfDefinitionExecMethod {
            exec: m.get_field_string("exec").ok()?,
            timeout: m.get_field_u32("timeout").ok()?,
            context: m.get_field_struct_opt("context").and_then(|c| {
                Some(SmfDefinitionExecMethodContext {
                    user: c.get_field_string("user").ok()?,
                    group: c.get_field_string_opt("group"),
                    privileges: c.get_field_string_opt("privileges"),
                })
            }),
        })
    }))
}

fn unpack_smf(data: &JanetStruct) -> anyhow::Result<SmfDefinition> {
    Ok(SmfDefinition {
        name: data.get_field_string("name")?,
        description: data.get_field_string("description")?,
        fmri: data.get_field_string("fmri")?,
        single_instance: data.get_field_bool("single-instance")?,
        default_enabled: data.get_field_bool("default-enabled")?,
        start_method: unpack_smf_method(data, "start-method")?,
        stop_method: unpack_smf_method(data, "stop-method")?,
        refresh_method: unpack_smf_method(data, "refresh-method")?,
    })
}

crate::unpack_fn!(ensure_list, Smf, GurpSmf, box);
crate::unpack_fn!(remove_list, Smf, GurpSmf, box);
crate::impl_apply!(GurpSmf);

impl GurpSmf {
    fn apply_ensure(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        let new_manifest = smf_builder::make_manifest(self.desired_state.as_ref().unwrap());

        if svcs::exists(&self.name)? {
            tracing::debug!("service exists: {}", &self.name);

            if self.manifest_path.exists() {
                let current_manifest = fs::read_to_string(&self.manifest_path)?;
                let desired_xml = helpers::parse_xml(&new_manifest)?;
                let current_xml = helpers::parse_xml(&current_manifest)?;
                if desired_xml == current_xml {
                    tracing::info!("no change: {}", self.name);
                    return Ok(ONE_RESOURCE_NO_CHANGE);
                }
            } else {
                tracing::debug!("creating manifest: {} ", self.manifest_path);
            }

            tracing::info!("change service: {}", self.name);
        } else {
            tracing::info!("create service: {}", self.name);
        };

        tracing::debug!("rewriting manifest: {}", self.manifest_path);

        if opts.noop {
            Ok(ONE_RESOURCE_NOOP)
        } else {
            debug!(opts, "doer/smf", "SMF manifest follows:\n{}", new_manifest);
            fs::write(&self.manifest_path, &new_manifest)?;
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

        svcs::run_svccfg("import", self.manifest_path.as_str())?;

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn apply_remove(
        &self,
        _apply_context: &ApplyContext,
        _opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::init_janet;
    use janetrs::structs;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_unpack_ensure_file() {
        init_janet();

        let start_context = structs! {
            ":user" => "telegraf",
            ":group" => "daemon",
            ":privileges" => "basic,file_dac_search,sys_admin,proc_owner,proc_zone",
        };

        let start_method = structs! {
            ":exec" => "/opt/site/lib/smf/method/telegraf.sh",
            ":timeout" => 60,
            ":context" => start_context,
        };

        let stop_method = structs! {
            ":exec" => ":kill",
            ":timeout" => 10,
        };

        let refresh_method = structs! {
            ":exec" => ":kill -THAW",
            ":timeout" => 60,
        };

        let test_ensure = structs! {
            ":_id" => "/test-role/smf/test-smf",
            ":action" => ":ensure",
            ":name" => "export",
            ":description" => "Run Telegraf agent",
            ":fmri" => "sysdef/telegraf",
            ":default-enabled" => true,
            ":single-instance" => true,
            ":start-method" => start_method,
            ":stop-method" => stop_method,
            ":refresh-method" => refresh_method,
        };

        let test_svc = SmfDefinition {
            name: "export".to_owned(),
            description: "Run Telegraf agent".to_owned(),
            fmri: "sysdef/telegraf".to_owned(),
            single_instance: true,
            default_enabled: true,
            start_method: Some(SmfDefinitionExecMethod {
                exec: "/opt/site/lib/smf/method/telegraf.sh".to_owned(),
                timeout: 60,
                context: Some(SmfDefinitionExecMethodContext {
                    user: "telegraf".to_owned(),
                    group: Some("daemon".to_owned()),
                    privileges: Some(
                        "basic,file_dac_search,sys_admin,proc_owner,proc_zone".to_owned(),
                    ),
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

        assert_eq!(test_svc, unpack_smf(&test_ensure).unwrap());
    }
}
