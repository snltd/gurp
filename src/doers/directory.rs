use crate::doers::types::Resource;
use crate::utils::janet_helpers::{JanetExt, JanetTableExt};
use anyhow::anyhow;
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray};

#[derive(Debug, PartialEq)]
pub enum DirectoryResource {
    Ensure(DirectoryEnsure),
    Remove(DirectoryRemove),
}

#[derive(Debug, PartialEq)]
pub struct DirectoryEnsure {
    pub group: String,
    pub mode: String,
    pub name: String,
    pub owner: String,
    pub path: Utf8PathBuf,
    pub recurse: bool,
}

#[derive(Debug, PartialEq)]
struct GurpDirectory {
    pub path: Utf8PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct DirectoryState {
    pub group: String,
    pub mode: String,
    pub name: String,
    pub owner: String,
}

#[derive(Debug, PartialEq)]
pub struct DirectoryRemove {
    pub path: Utf8PathBuf,
    pub recurse: bool,
}

impl DirectoryResource {
    pub fn apply(&self) {
        println!("APPLYING THE DIR");
    }
}

impl DirectoryEnsure {
    pub fn apply(&self) {
        println!("ensuring directory {:?}", self);
    }
}

impl DirectoryRemove {
    pub fn apply(&self) {
        println!("removing directory {:?}", self);
    }
}

impl TryFrom<&Janet> for DirectoryResource {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<DirectoryResource> {
        let table = value.extract_table()?;

        match table.get_field_string("action")?.as_str() {
            "ensure" => Ok(DirectoryResource::Ensure(DirectoryEnsure {
                name: table.get_field_string("name")?,
                group: table.get_field_string("group")?,
                owner: table.get_field_string("owner")?,
                mode: table.get_field_string("mode")?,
                path: table.get_field_pathbuf("path")?,
                recurse: table.get_field_bool("recurse")?,
            })),
            "remove" => Ok(DirectoryResource::Remove(DirectoryRemove {
                path: table.get_field_pathbuf("path")?,
                recurse: table.get_field_bool("recurse")?,
            })),
            other => Err(anyhow!(format!(
                "action must be 'ensure' or 'remove' (got {})",
                other
            ))),
        }
    }
}

pub fn unpack_list(resource_list: &Janet) -> anyhow::Result<Vec<Resource>> {
    let resource_list = resource_list.extract_array()?;

    let mut ret = Vec::new();

    for r in resource_list {
        ret.push(Resource::Directory(DirectoryResource::try_from(&r)?));
    }

    Ok(ret)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unpack() {
        init_janet();

        let example_dir_ensure = Janet::wrap(janetrs::table! {
            ":action" => "ensure",
            ":group" => "sysadmin",
            ":mode" => "0755",
            ":name" => "test_directory",
            ":owner" => "rob",
            ":recurse" => true,
            ":path" => "/tmp/merp",
        });

        let expected_ensure = DirectoryResource::Ensure(DirectoryEnsure {
            group: "sysadmin".to_owned(),
            mode: "0755".to_owned(),
            name: "test_directory".to_owned(),
            owner: "rob".to_owned(),
            path: Utf8PathBuf::from("/tmp/merp"),
            recurse: true,
        });

        assert_eq!(
            expected_ensure,
            DirectoryResource::try_from(&example_dir_ensure).unwrap()
        );

        let example_dir_remove = Janet::wrap(janetrs::table! {
            ":action" => "remove",
            ":recurse" => false,
            ":path" => "/tmp/merp",
        });

        let expected_remove = DirectoryResource::Remove(DirectoryRemove {
            path: Utf8PathBuf::from("/tmp/merp"),
            recurse: false,
        });

        assert_eq!(
            expected_remove,
            DirectoryResource::try_from(&example_dir_remove).unwrap()
        );
    }

    fn init_janet() {
        unsafe {
            janetrs::lowlevel::janet_init();
        }
    }
}
