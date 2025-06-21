use crate::doers::cron::GurpCron;
use crate::doers::directory::GurpDirectory;
use crate::doers::file::GurpFile;
use crate::doers::file_line::GurpFileLine;
use crate::doers::gem::GurpGem;
use crate::doers::misc::GurpMisc;
use crate::doers::pkg::GurpPkg;
use crate::doers::smf::GurpSmf;
use crate::doers::svc::GurpSvc;
use crate::doers::symlink::GurpSymlink;
use crate::doers::user::GurpUser;
use crate::doers::zfs::GurpZfs;
use std::collections::{HashMap, HashSet};
use std::ops::Add;

pub type ExitCode = u8;

#[derive(Clone)]
pub struct Opts {
    pub debug: bool,
    pub noop: bool,
    pub verbose: bool,
    pub no_colour: bool,
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
    Smf(GurpSmf),
    Zfs(GurpZfs),
    Gem(GurpGem),
    Symlink(GurpSymlink),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Ensure,
    Remove,
}

pub type Changes<'a> = Vec<&'a str>;

#[derive(Debug, Default, PartialEq, Copy, Clone)]
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

pub type ChangedIds = HashSet<String>;

#[derive(Default)]
pub struct ApplyContext {
    pub changed_ids: ChangedIds,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Hash)]
pub struct SmfDefinition {
    pub name: String,
    pub description: String,
    pub fmri: String,
    pub default_enabled: bool,
    pub single_instance: bool,
    pub start_method: Option<SmfDefinitionExecMethod>,
    pub stop_method: Option<SmfDefinitionExecMethod>,
    pub refresh_method: Option<SmfDefinitionExecMethod>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Hash)]
pub struct SmfDefinitionDependencySvc {
    pub name: String,
    pub restart_on: String,
    pub fmri: String,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Hash)]
pub struct SmfDefinitionExecMethod {
    pub exec: String,
    pub timeout: u32,
    pub context: Option<SmfDefinitionExecMethodContext>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Hash)]
pub struct SmfDefinitionExecMethodContext {
    pub user: String,
    pub group: Option<String>,
    pub privileges: Option<String>,
}
