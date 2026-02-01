use xml::reader::{EventReader, XmlEvent};

pub fn parse(content: &str) -> Result<Vec<XmlEvent>, xml::reader::Error> {
    EventReader::from_str(content).into_iter().collect()
}
