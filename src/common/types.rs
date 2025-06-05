use crate::doers::directory::GurpDirectory;
use crate::doers::pkg::GurpPkg;
use crate::doers::user::GurpUser;
use camino::Utf8PathBuf;
use std::collections::HashMap;
use std::ops::Add;

#[derive(Clone)]
pub struct Opts {
    pub debug: bool,
    pub noop: bool,
    pub verbose: bool,
    pub gurp_lib_path: Option<Utf8PathBuf>,
}

pub enum Resource {
    Directory(GurpDirectory),
    User(GurpUser),
    Pkg(GurpPkg),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Ensure,
    Remove,
}

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

pub struct HostConfig {
    pub metadata: HostMetadata,
    pub resources: HostResources,
}

#[derive(Debug, PartialEq)]
pub struct HostMetadata {
    pub name: String,
}

pub type EnsureResources = HashMap<String, Vec<Resource>>;
pub type RemoveResources = HashMap<String, Vec<Resource>>;

pub struct HostResources {
    pub ensure: EnsureResources,
    pub remove: RemoveResources,
}
