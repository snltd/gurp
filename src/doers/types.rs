// use crate::doers::directory::DirectoryResource;
use crate::{
    doers::directory::{DirectoryToEnsure, DirectoryToRemove},
    doers::pkg::{PkgsToEnsure, PkgsToRemove},
    utils::types::Opts,
};
use std::collections::HashMap;
use std::ops::Add;

pub type Changes<'a> = Vec<&'a str>;

#[derive(Debug, PartialEq)]
pub struct ApplySummary {
    pub resources: u32,
    pub changes: u32,
    pub errors: u32,
}

impl Add for ApplySummary {
    type Output = ApplySummary;

    fn add(self, other: ApplySummary) -> ApplySummary {
        ApplySummary {
            resources: self.resources + other.resources,
            changes: self.changes + other.changes,
            errors: self.errors + other.errors,
        }
    }
}

pub trait Apply {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary>;
}

#[derive(Debug)]
pub enum Ensure {
    Directory(DirectoryToEnsure),
    Pkgs(PkgsToEnsure),
}

impl Ensure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        match self {
            Ensure::Directory(resource) => resource.apply(opts),
            Ensure::Pkgs(resource) => resource.apply(opts),
        }
    }
}

#[derive(Debug)]
pub enum Remove {
    Directory(DirectoryToRemove),
    Pkgs(PkgsToRemove),
}

impl Remove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        match self {
            Remove::Directory(resource) => resource.apply(opts),
            Remove::Pkgs(resource) => resource.apply(opts),
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
