// use crate::doers::directory::DirectoryResource;
use crate::{
    doers::directory::{DirectoryToEnsure, DirectoryToRemove},
    utils::types::Opts,
};
use std::collections::HashMap;

pub trait Apply {
    fn apply(&self, opts: &Opts) -> anyhow::Result<bool>;
}

#[derive(Debug)]
pub enum Ensure {
    Directory(DirectoryToEnsure),
}

impl Ensure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<bool> {
        match self {
            Ensure::Directory(d) => d.apply(opts),
        }
    }
}

#[derive(Debug)]
pub enum Remove {
    Directory(DirectoryToRemove),
}

impl Remove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<bool> {
        match self {
            Remove::Directory(d) => d.apply(opts),
        }
    }
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
