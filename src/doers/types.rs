// use crate::doers::directory::DirectoryResource;
use crate::{
    doers::directory::{DirectoryToEnsure, DirectoryToRemove},
    doers::package::{PackagesToEnsure, PackagesToRemove},
    utils::types::Opts,
};
use std::collections::HashMap;

pub trait Apply {
    fn apply(&self, opts: &Opts) -> anyhow::Result<bool>;
}

#[derive(Debug)]
pub enum Ensure {
    Directory(DirectoryToEnsure),
    Packages(PackagesToEnsure),
}

impl Ensure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<bool> {
        match self {
            Ensure::Directory(resource) => resource.apply(opts),
            Ensure::Packages(resource) => resource.apply(opts),
        }
    }
}

#[derive(Debug)]
pub enum Remove {
    Directory(DirectoryToRemove),
    Packages(PackagesToRemove),
}

impl Remove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<bool> {
        match self {
            Remove::Directory(resource) => resource.apply(opts),
            Remove::Packages(resource) => resource.apply(opts),
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
