use crate::doers::cron::GurpCron;
use crate::doers::directory::GurpDirectory;
use crate::doers::file::GurpFile;
use crate::doers::file_line::GurpFileLine;
use crate::doers::misc::GurpMisc;
use crate::doers::pkg::GurpPkg;
use crate::doers::svc::GurpSvc;
use crate::doers::user::GurpUser;
use std::collections::{HashMap, HashSet};
use std::ops::Add;

pub type ExitCode = u8;

#[derive(Clone)]
pub struct Opts {
    pub debug: bool,
    pub noop: bool,
    pub verbose: bool,
}

pub enum Resource {
    Directory(GurpDirectory),
    User(GurpUser),
    Pkg(GurpPkg),
    FileLine(GurpFileLine),
    File(GurpFile),
    Cron(GurpCron),
    Svc(GurpSvc),
    Misc(GurpMisc),
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

impl Default for ApplySummary {
    fn default() -> Self {
        Self {
            resources: 0,
            changes: 0,
            errors: 0,
        }
    }
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

pub type ChangedIds = HashSet<String>;

pub struct ApplyContext {
    pub changed_ids: ChangedIds,
}
