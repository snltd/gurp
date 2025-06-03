use crate::{
    doers::directory::{DirectoryToEnsure, DirectoryToRemove},
    doers::pkg::{PkgsToEnsure, PkgsToRemove},
    doers::user::{UserToEnsure, UserToRemove},
    utils::types::Opts,
};
use std::collections::HashMap;
use std::ops::Add;

pub type Changes<'a> = Vec<&'a str>;

#[derive(Debug, PartialEq, Copy, Clone)]
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
    User(UserToEnsure),
}

impl Ensure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        match self {
            Ensure::Directory(resource) => resource.apply(opts),
            Ensure::Pkgs(resource) => resource.apply(opts),
            Ensure::User(resource) => resource.apply(opts),
        }
    }
}

#[derive(Debug)]
pub enum Remove {
    Directory(DirectoryToRemove),
    Pkgs(PkgsToRemove),
    User(UserToRemove),
}

impl Remove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        match self {
            Remove::Directory(resource) => resource.apply(opts),
            Remove::Pkgs(resource) => resource.apply(opts),
            Remove::User(resource) => resource.apply(opts),
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

pub type EnsureResources = HashMap<String, Vec<Ensure>>;
pub type RemoveResources = HashMap<String, Vec<Remove>>;

#[derive(Debug)]
pub struct HostResources {
    pub ensure: EnsureResources,
    pub remove: RemoveResources,
}

pub trait HasId {
    fn id(&self) -> &str;
}

impl Ensure {
    pub fn id(&self) -> &str {
        match self {
            Ensure::Directory(inner) => inner.id(),
            Ensure::Pkgs(inner) => inner.id(),
            Ensure::User(inner) => inner.id(),
        }
    }
}

impl Remove {
    pub fn id(&self) -> &str {
        match self {
            Remove::Directory(inner) => inner.id(),
            Remove::Pkgs(inner) => inner.id(),
            Remove::User(inner) => inner.id(),
        }
    }
}
