use anyhow::{Context, bail, ensure};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use xmlwriter::{Options, XmlWriter};

pub struct SmfBuilder {
    xml: XmlWriter,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Hash)]
#[serde(untagged)]
pub enum PropertyValue {
    Bool(bool),
    Int(i64),
    String(String),
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Bool(b) => write!(f, "{b}"),
            PropertyValue::Int(i) => write!(f, "{i}"),
            PropertyValue::String(s) => write!(f, "\"{s}\""),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Hash)]
pub struct PropertyStruct {
    pub value: PropertyValue,
    #[serde(rename = "type")]
    pub prop_type: String,
}

pub type PropertyName = String;
pub type PropertyGroupName = String;
pub type PropertyGroupType = String;
pub type PropertyList = Vec<PropertyName>;
pub type PropertyMap = BTreeMap<String, PropertyStruct>;
pub type PropertyGroupMap = BTreeMap<PropertyGroupName, PropertyGroupType>;
pub type PropertyGroupList = BTreeSet<PropertyGroupName>;
pub type SvcProps = BTreeMap<PropertyName, PropertyStruct>;

#[derive(Deserialize, Debug, Hash, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SmfDefinition {
    pub name: String,
    pub duration: Option<String>,
    pub description: Option<String>,
    pub fmri: String,
    pub default_enabled: bool,
    pub single_instance: bool,
    pub start_method: Option<SmfDefinitionExecMethod>,
    pub stop_method: Option<SmfDefinitionExecMethod>,
    pub refresh_method: Option<SmfDefinitionExecMethod>,
    pub property_groups: Option<PropertyGroupMap>,
    pub properties: Option<PropertyMap>,
    pub dependencies: Option<Vec<SmfDefinitionDependencySvc>>,
    pub dependents: Option<Vec<SmfDefinitionDependentSvc>>,
}

#[derive(PartialEq, Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct SmfDefinitionDependencySvc {
    pub name: String,
    pub restart_on: String,
    pub fmri: String,
    pub grouping: String,
    #[serde(rename = "type")]
    pub dep_type: String,
}

#[derive(PartialEq, Debug, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct SmfDefinitionDependentSvc {
    pub name: String,
    pub restart_on: String,
    pub fmri: String,
    pub grouping: String,
    #[serde(rename = "type")]
    pub dep_type: String,
}

#[derive(Deserialize, Debug, Hash, PartialEq)]
pub struct SmfDefinitionExecMethod {
    pub exec: String,
    pub timeout: u32,
    pub context: Option<SmfDefinitionExecMethodContext>,
}

#[derive(Deserialize, Debug, Hash, PartialEq)]
pub struct SmfDefinitionExecMethodContext {
    pub user: String,
    pub group: Option<String>,
    pub privileges: Option<String>,
    pub environment: Option<BTreeMap<String, String>>,
}

impl SmfBuilder {
    pub fn new(def: &SmfDefinition) -> Self {
        let mut xml = XmlWriter::new(Options {
            use_single_quote: true,
            indent: xmlwriter::Indent::Spaces(2),
            ..Options::default()
        });

        xml.start_element("service_bundle");
        xml.write_attribute("type", "manifest");
        xml.write_attribute("name", &def.name);

        Self { xml }
    }

    pub fn add_service<F>(&mut self, name: &str, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut ServiceBuilder) -> anyhow::Result<()>,
    {
        self.xml.start_element("service");
        self.xml.write_attribute("name", name);
        self.xml.write_attribute("type", "service");
        self.xml.write_attribute("version", &1);

        let mut svc = ServiceBuilder { xml: &mut self.xml };
        f(&mut svc)?;

        self.xml.end_element(); // service
        Ok(())
    }

    pub fn finish(mut self) -> String {
        self.xml.end_element(); // service_bundle
        self.xml.end_document()
    }
}

pub struct ServiceBuilder<'a> {
    xml: &'a mut XmlWriter,
}

impl ServiceBuilder<'_> {
    pub fn enable_default_instance(&mut self, enabled: bool) {
        self.xml.start_element("create_default_instance");
        self.xml.write_attribute("enabled", &enabled);
        self.xml.end_element();
    }

    pub fn add_svc_dependency(&mut self, def: &SmfDefinitionDependencySvc) {
        self.xml.start_element("dependency");
        self.xml.write_attribute("name", &def.name);
        self.xml.write_attribute("grouping", &def.grouping);
        self.xml.write_attribute("restart_on", &def.restart_on);
        self.xml.write_attribute("type", &def.dep_type);

        self.xml.start_element("service_fmri");
        self.xml.write_attribute("value", &def.fmri);
        self.xml.end_element();

        self.xml.end_element();
    }

    pub fn add_svc_dependent(&mut self, def: &SmfDefinitionDependentSvc) {
        self.xml.start_element("dependent");
        self.xml.write_attribute("name", &def.name);
        self.xml.write_attribute("grouping", &def.grouping);
        self.xml.write_attribute("restart_on", &def.restart_on);
        self.xml.write_attribute("type", &def.dep_type);

        self.xml.start_element("service_fmri");
        self.xml.write_attribute("value", &def.fmri);
        self.xml.end_element();

        self.xml.end_element();
    }

    pub fn add_exec_method(&mut self, name: &str, def: &SmfDefinitionExecMethod) {
        self.xml.start_element("exec_method");
        self.xml.write_attribute("name", name);
        self.xml.write_attribute("type", "method");
        self.xml.write_attribute("exec", &def.exec);
        self.xml.write_attribute("timeout_seconds", &def.timeout);

        if let Some(context) = &def.context {
            self.xml.start_element("method_context");

            self.xml.start_element("method_credential");
            self.xml.write_attribute("user", &context.user);

            if let Some(group) = &context.group {
                self.xml.write_attribute("group", &group);
            }

            if let Some(privileges) = &context.privileges {
                self.xml.write_attribute("privileges", &privileges);
            }
            self.xml.end_element();

            if let Some(environment) = &context.environment {
                self.xml.start_element("method_environment");
                for (k, v) in environment {
                    self.xml.start_element("envvar");
                    self.xml.write_attribute("name", k);
                    self.xml.write_attribute("value", v);
                    self.xml.end_element();
                }
                self.xml.end_element();
            }
            self.xml.end_element(); // method_context
        }

        self.xml.end_element(); // exec_method
    }

    // do you actually need this? It's in my manifests
    pub fn add_stability(&mut self) {
        self.xml.start_element("stability");
        self.xml.write_attribute("value", "Unstable");
        self.xml.end_element();
    }

    pub fn add_properties(
        &mut self,
        prop_groups: &BTreeMap<String, String>,
        properties: &BTreeMap<String, PropertyStruct>,
    ) -> anyhow::Result<()> {
        // The property name is property_group/property.

        println!("prop_groups: {:?}", prop_groups);

        let mut current_group = "";

        // The properties are sorted because it's a BTreeSet
        for (prop, val) in properties {
            let chunks: Vec<&str> = prop.split("/").collect();
            ensure!(chunks.len() == 2, "invalid property name: {prop}");

            let group_name = chunks[0];
            let prop_name = chunks[1];

            // Open a new property group
            if current_group != group_name {
                if !current_group.is_empty() {
                    // We were already working on a different group, so close it
                    self.xml.end_element(); // property_group
                }

                let group_type = prop_groups.get(group_name).context(format!(
                    "cannot find property_group definition for {}: available: {:?}",
                    group_name,
                    prop_groups.keys()
                ))?;

                self.xml.start_element("property_group");
                self.xml.write_attribute("name", group_name);
                self.xml.write_attribute("type", group_type);

                current_group = group_name;
            }

            self.xml.start_element("propval");
            self.xml.write_attribute("name", prop_name);
            self.xml.write_attribute("type", &val.prop_type);
            self.xml.write_attribute("value", &val.value);
            self.xml.end_element(); // propval
        }

        self.xml.end_element(); // property_group
        Ok(())
    }

    pub fn add_duration_pg(&mut self, duration: &str) {
        self.xml.start_element("property_group");
        self.xml.write_attribute("name", "startd");
        self.xml.write_attribute("type", "framework");

        self.xml.start_element("propval");
        self.xml.write_attribute("name", "duration");
        self.xml.write_attribute("type", "astring");
        self.xml.write_attribute("value", duration);
        self.xml.end_element(); // propval

        self.xml.end_element(); // property_group
    }

    // do you actually need this? It's in my manifests
    pub fn add_template(&mut self, description: &str) {
        self.xml.start_element("template");
        self.xml.start_element("common_name");
        self.xml.start_element("loctext");
        self.xml.write_attribute("xml:lang", "C");
        self.xml.write_text(description);
        self.xml.end_element(); // loctext
        self.xml.end_element(); // common name
        self.xml.end_element(); // template
    }

    pub fn add_single_instance(&mut self) {
        self.xml.start_element("single_instance");
        self.xml.end_element();
    }
}

pub fn make_manifest(def: &SmfDefinition) -> anyhow::Result<String> {
    let mut builder = SmfBuilder::new(def);

    builder.add_service(
        &def.fmri,
        |svc: &mut ServiceBuilder| -> anyhow::Result<()> {
            svc.enable_default_instance(def.default_enabled);

            if def.single_instance {
                svc.add_single_instance();
            }

            // we'll always expect network and local filesystem
            svc.add_svc_dependency(&SmfDefinitionDependencySvc {
                name: "physical".to_owned(),
                restart_on: "none".to_owned(),
                grouping: "require_all".to_owned(),
                dep_type: "service".to_owned(),
                fmri: "svc:/network/physical:default".to_owned(),
            });

            svc.add_svc_dependency(&SmfDefinitionDependencySvc {
                name: "fs-local".to_owned(),
                restart_on: "none".to_owned(),
                grouping: "require_all".to_owned(),
                dep_type: "service".to_owned(),
                fmri: "svc:/system/filesystem/local".to_owned(),
            });

            if let Some(dependencies) = &def.dependencies {
                for dep in dependencies {
                    svc.add_svc_dependency(dep);
                }
            }

            if let Some(dependents) = &def.dependents {
                for dep in dependents {
                    svc.add_svc_dependent(dep);
                }
            }

            if let Some(method) = &def.start_method {
                svc.add_exec_method("start", method)
            }

            if let Some(method) = &def.stop_method {
                svc.add_exec_method("stop", method)
            }

            if let Some(method) = &def.refresh_method {
                svc.add_exec_method("refresh", method)
            }

            if let Some(duration) = &def.duration {
                svc.add_duration_pg(duration);
            }

            if let Some(props) = &def.properties {
                if let Some(prop_groups) = &def.property_groups {
                    svc.add_properties(prop_groups, props)?;
                } else {
                    bail!("properties requires property_groups");
                }
            }

            svc.add_stability();

            if let Some(description) = &def.description {
                svc.add_template(description);
            }

            Ok(())
        },
    )?;

    let mut ret = "<?xml version='1.0'?>\n".to_owned();
    ret.push_str(
        "<!DOCTYPE service_bundle SYSTEM '/usr/share/lib/xml/dtd/service_bundle.dtd.1'>\n",
    );
    ret.push_str(&builder.finish());
    Ok(ret)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::xml;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;
    use tester::load_fixture;

    #[test]
    fn test_make_manifest() {
        let test_svc = SmfDefinition {
            name: "export".to_owned(),
            description: Some("Run Telegraf agent".to_owned()),
            duration: None,
            fmri: "sysdef/telegraf".to_owned(),
            single_instance: true,
            default_enabled: true,
            dependencies: Some(vec![SmfDefinitionDependencySvc {
                name: "test-dep".to_owned(),
                restart_on: "none".to_owned(),
                fmri: "svc:/example/service:default".to_owned(),
                grouping: "require_all".to_owned(),
                dep_type: "service".to_owned(),
            }]),
            dependents: None,
            start_method: Some(SmfDefinitionExecMethod {
                exec: "/opt/site/lib/smf/method/telegraf.sh".to_owned(),
                timeout: 60,
                context: Some(SmfDefinitionExecMethodContext {
                    user: "telegraf".to_owned(),
                    group: Some("daemon".to_owned()),
                    privileges: Some(
                        "basic,file_dac_search,sys_admin,proc_owner,proc_zone".to_owned(),
                    ),
                    environment: Some(BTreeMap::from([
                        ("LC_CTYPE".to_owned(), "en_US.UTF-8".to_owned()),
                        ("PATH".to_owned(), "/opt/site/bin".to_owned()),
                    ])),
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
            properties: Some(BTreeMap::from([(
                "application/setting".to_owned(),
                PropertyStruct {
                    value: PropertyValue::String("some_value".to_owned()),
                    prop_type: "astring".to_owned(),
                },
            )])),
            property_groups: Some(BTreeMap::from([(
                "application".to_owned(),
                "application".to_owned(),
            )])),
        };

        let result = make_manifest(&test_svc).unwrap();
        let expected = load_fixture("smf_helper/telegraf.xml");
        let result_xml = xml::parse(&result);
        let expected_xml = xml::parse(&expected);

        assert_eq!(&expected_xml, &result_xml);
    }

    // #[test]
    fn test_make_transient_manifest() {
        let test_svc = SmfDefinition {
            name: "export".to_owned(),
            description: Some("Run boot-service".to_owned()),
            duration: Some("transient".to_owned()),
            fmri: "sysdef/boot-service".to_owned(),
            single_instance: true,
            default_enabled: true,
            property_groups: None,
            dependencies: None,
            dependents: None,
            properties: None,
            start_method: Some(SmfDefinitionExecMethod {
                exec: "/opt/site/lib/smf/method/boot-service.sh".to_owned(),
                timeout: 60,
                context: None,
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

        let result = make_manifest(&test_svc).unwrap();
        let expected = load_fixture("smf_helper/boot-service.xml");
        let result_xml = xml::parse(&result);
        let expected_xml = xml::parse(&expected);

        assert_eq!(&expected_xml, &result_xml);
    }
}
