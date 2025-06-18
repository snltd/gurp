use std::process::Command;
use xml::reader::{EventReader, XmlEvent};

pub fn command_to_string(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args = cmd
        .get_args()
        .map(|a| a.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");
    format!("{program} {args}")
}

pub fn parse_xml(content: &str) -> Result<Vec<XmlEvent>, xml::reader::Error> {
    EventReader::from_str(content).into_iter().collect()
}
