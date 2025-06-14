use xmlwriter::{Options, XmlWriter};

pub struct SmfBuilder {
    xml: XmlWriter,
    fingerprint: u64,
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

        SmfBuilder {
            xml,
            fingerprint: fingerprint(def),
        }
    }

    pub fn add_service<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut ServiceBuilder),
    {
        self.xml.start_element("service");
        self.xml.write_attribute("name", name);
        self.xml.write_attribute("type", "service");
        // I want to be able to store the hash fingerprint in this version field so I can quickly
        // see if the manifest will change
        self.xml.write_attribute("version", &self.fingerprint);

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

impl<'a> ServiceBuilder<'a> {
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

#[derive(Hash)]
struct SmfDefinitionExecMethodContext {
    user: String,
    group: Option<String>,
    privileges: Option<String>,
}

#[derive(Hash)]
struct SmfDefinitionExecMethod {
    exec: String,
    timeout: u32,
    context: Option<SmfDefinitionExecMethodContext>,
}

#[derive(Hash)]
struct SmfDefinitionDependencySvc {
    name: String,
    restart_on: String,
    fmri: String,
}

#[derive(Hash)]
struct SmfDefinition {
    name: String,
    description: String,
    fmri: String,
    default_enabled: bool,
    single_instance: bool,
    start_method: Option<SmfDefinitionExecMethod>,
    stop_method: Option<SmfDefinitionExecMethod>,
    refresh_method: Option<SmfDefinitionExecMethod>,
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn fingerprint<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

fn make_manifest(def: &SmfDefinition) -> String {
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

        svc.add_stability();
        svc.add_template(&def.description);
    });

    let mut ret = "<?xml version='1.0'?>\n".to_owned();
    ret.push_str("<!DOCTYPE service_bundle SYSTEM '/usr/share/lib/xml/dtd/service_bundle.dtd.1'>");
    ret.push_str(&builder.finish());
    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::load_fixture;
    // use minidom::Element;
    use pretty_assertions::assert_eq;
    use xml::reader::{EventReader, XmlEvent};

    #[test]
    fn test_make_manifest() {
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

        let result = make_manifest(&test_svc);
        let expected = load_fixture("util/smf_helper/telegraf.xml");
        // let result_xml: Vec<XmlEvent> = EventReader::from_str(&result).into_iter().collect():
        // let expected_xml: Vec<XmlEvent> = EventReader::from_str(&expected).into_iter().collect();
        let result_xml = parse_xml(&result);
        let expected_xml = parse_xml(&expected);

        assert_eq!(&expected_xml, &result_xml);
    }

    fn parse_xml(content: &str) -> Result<Vec<XmlEvent>, xml::reader::Error> {
        EventReader::from_str(content).into_iter().collect()
    }

    /*
    use xmltree::Element;

    fn normalize(xml: &str) -> Element {
        Element::parse(xml.as_bytes()).expect("valid XML")
    }

    fn elements_equal(a: &Element, b: &Element) -> bool {
        a.name == b.name
            && a.attributes == b.attributes
            && a.get_text() == b.get_text()
            && a.children.len() == b.children.len()
            && a.children
                .iter()
                .zip(&b.children)
                .all(|(ac, bc)| match (ac, bc) {
                    (xmltree::XMLNode::Element(e1), xmltree::XMLNode::Element(e2)) => {
                        elements_equal(e1, e2)
                    }
                    (xmltree::XMLNode::Text(t1), xmltree::XMLNode::Text(t2)) => t1 == t2,
                    _ => false,
                })
    }
    */
}
