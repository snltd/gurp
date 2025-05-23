// use crate::doers::directory::DirectoryResource;
use camino::Utf8PathBuf;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Ensure {
    Directory(DirectoryEnsure),
}

#[derive(Debug)]
pub enum Remove {
    Directory(DirectoryRemove),
}

#[derive(Debug, PartialEq)]
pub struct DirectoryEnsure {
    pub id: String,
    pub group: String,
    pub mode: String,
    pub name: String,
    pub owner: String,
    pub path: Utf8PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct DirectoryRemove {
    pub id: String,
    pub path: Utf8PathBuf,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct DirectoryStateEnsure {
    pub group: String,
    pub mode: String,
    pub owner: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct DirectoryStateRemove {
    pub exists: bool,
}

#[derive(Debug)]
pub struct HostConfig {
    pub metadata: HostMetadata,
    pub resources: HostResources,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HostMetadata {
    pub name: String,
}

// type ResourceType = String;

pub type EnsureResources = HashMap<String, Vec<Ensure>>;
pub type RemoveResources = HashMap<String, Vec<Remove>>;

#[derive(Debug)]
pub struct HostResources {
    pub ensure: EnsureResources,
    pub remove: RemoveResources,
}

// pub struct EnableResource {
//     pub directory: Vec<DirectoryEnsure>,
// }

// pub struct RemoveResource {
//     pub directory: Vec<DirectoryRemove>,
// }

// I keep changing my mind whether this should be a hash or a vec. We'll see what fits the
// problem best.
// type HostResources = HashMap<ResourceType, Vec<Resource>>;

// #[derive(Debug, PartialEq)]
// pub enum DirectoryResource {
//     Ensure(DirectoryEnsure),
//     Remove(DirectoryRemove),
// }

// #[derive(Debug, PartialEq)]
// struct GurpDirectory {
//     pub path: Utf8PathBuf,
// }

// #[derive(Debug)]
// pub enum Resource {
//     Directory(DirectoryResource),
// }

// impl Resource {
//     fn apply(&self) {
//         match self {
//             Resource::Directory(d) => d.apply(),
//         }
//     }
// }
