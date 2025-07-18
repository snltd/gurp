use crate::common::types::{SmfDefinition, SmfDefinitionDependencySvc, SmfDefinitionExecMethod};
use xmlwriter::{Options, XmlWriter};

pub struct SmfBuilder {
    xml: XmlWriter,
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

    pub fn add_service<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut ServiceBuilder),
    {
        self.xml.start_element("service");
        self.xml.write_attribute("name", name);
        self.xml.write_attribute("type", "service");
        self.xml.write_attribute("version", &1);

        let mut svc = ServiceBuilder { xml: &mut self.xml };
        f(&mut svc);

        self.xml.end_element(); // service
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

    // Everything's going in the `require_all` grouping.
    //
    pub fn add_svc_dependency(&mut self, def: &SmfDefinitionDependencySvc) {
        self.xml.start_element("dependency");
        self.xml.write_attribute("name", &def.name);
        self.xml.write_attribute("grouping", "require_all");
        self.xml.write_attribute("restart_on", &def.restart_on);
        self.xml.write_attribute("type", "service");

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

        if let Some(context) = def.context.as_ref() {
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

            self.xml.end_element();
        }

        self.xml.end_element();
    }

    // do you actually need this? It's in my manifests
    pub fn add_stability(&mut self) {
        self.xml.start_element("stability");
        self.xml.write_attribute("value", "Unstable");
        self.xml.end_element();
    }

    pub fn add_duration_pg(&mut self, duration: &str) {
        self.xml.start_element("property_group");
        self.xml.write_attribute("name", "startd");
        self.xml.write_attribute("type", "framework");

        self.xml.start_element("propval");
        self.xml.write_attribute("name", "duration");
        self.xml.write_attribute("type", "astring");
        self.xml.write_attribute("value", duration);
        self.xml.end_element();

        self.xml.end_element();
    }

    // do you actually need this? It's in my manifests
    pub fn add_template(&mut self, description: &str) {
        self.xml.start_element("template");

        self.xml.start_element("common_name");

        self.xml.start_element("loctext");
        self.xml.write_attribute("xml:lang", "C");
        self.xml.write_text(description);
        self.xml.end_element();

        self.xml.end_element();

        self.xml.end_element();
    }

    pub fn add_single_instance(&mut self) {
        self.xml.start_element("single_instance");
        self.xml.end_element();
    }
}

pub fn make_manifest(def: &SmfDefinition) -> String {
    let mut builder = SmfBuilder::new(def);

    builder.add_service(&def.fmri, |svc| {
        svc.enable_default_instance(def.default_enabled);

        if def.single_instance {
            svc.add_single_instance();
        }

        // we'll always expect network and local filesystem
        svc.add_svc_dependency(&SmfDefinitionDependencySvc {
            name: "physical".to_owned(),
            restart_on: "none".to_owned(),
            fmri: "svc:/network/physical:default".to_owned(),
        });

        svc.add_svc_dependency(&SmfDefinitionDependencySvc {
            name: "fs-local".to_owned(),
            restart_on: "none".to_owned(),
            fmri: "svc:/system/filesystem/local".to_owned(),
        });

        if let Some(method) = def.start_method.as_ref() {
            svc.add_exec_method("start", method)
        }

        if let Some(method) = def.stop_method.as_ref() {
            svc.add_exec_method("stop", method)
        }

        if let Some(method) = def.refresh_method.as_ref() {
            svc.add_exec_method("refresh", method)
        }

        if let Some(duration) = def.duration.as_ref() {
            svc.add_duration_pg(duration);
        }

        svc.add_stability();
        svc.add_template(&def.description);
    });

    let mut ret = "<?xml version='1.0'?>\n".to_owned();
    ret.push_str(
        "<!DOCTYPE service_bundle SYSTEM '/usr/share/lib/xml/dtd/service_bundle.dtd.1'>\n",
    );
    ret.push_str(&builder.finish());
    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::types::SmfDefinitionExecMethodContext;
    use crate::test_utils::spec_helper::load_fixture;
    use crate::utils::helpers;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_make_manifest() {
        let test_svc = SmfDefinition {
            name: "export".to_owned(),
            description: "Run Telegraf agent".to_owned(),
            duration: None,
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

        let result = make_manifest(&test_svc);
        let expected = load_fixture("util/smf_helper/telegraf.xml");
        let result_xml = helpers::parse_xml(&result);
        let expected_xml = helpers::parse_xml(&expected);

        assert_eq!(&expected_xml, &result_xml);
    }

    #[test]
    fn test_make_transient_manifest() {
        let test_svc = SmfDefinition {
            name: "export".to_owned(),
            description: "Run boot-service".to_owned(),
            duration: Some("transient".to_owned()),
            fmri: "sysdef/boot-service".to_owned(),
            single_instance: true,
            default_enabled: true,
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

        let result = make_manifest(&test_svc);
        let expected = load_fixture("util/smf_helper/boot-service.xml");
        let result_xml = helpers::parse_xml(&result);
        let expected_xml = helpers::parse_xml(&expected);

        assert_eq!(&expected_xml, &result_xml);
    }
}
