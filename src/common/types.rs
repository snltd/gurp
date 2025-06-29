use crate::doers::cron::{GurpCronEnsure, GurpCronRemove};
// use crate::doers::directory::{GurpDirectoryEnsure, GurpDirectoryRemove};
// use crate::doers::file::{GurpFileEnsure, GurpFileRemove};
use crate::doers::file_line::{GurpFileLineEnsure, GurpFileLineRemove};
// use crate::doers::gem::{GurpGemEnsure, GurpGemRemove};
use crate::doers::misc::GurpMiscEnsure;
// use crate::doers::pkg::{GurpPkgEnsure, GurpPkgRemove};
use crate::doers::smf::{GurpSmfEnsure, GurpSmfRemove};
use crate::doers::svc::GurpSvcEnsure;
use crate::doers::symlink::{GurpSymlinkEnsure, GurpSymlinkRemove};
use crate::doers::user::{GurpUserEnsure, GurpUserRemove};
use crate::doers::zfs::{GurpZfsEnsure, GurpZfsRemove};
use serde::{Deserialize, Serialize};
// use serde_json::Result;
use std::collections::HashSet;
use std::ops::Add;

pub type ExitCode = u8;

#[derive(Clone)]
pub struct Opts {
    pub debug: bool,
    pub noop: bool,
    pub verbose: bool,
    pub no_colour: bool,
}

// #[derive(Debug, Deserialize)]
// pub enum Resource {
//     Cron(Box<GurpCron>),
//     // Directory(GurpDirectory),
//     // File(GurpFile),
//     FileLine(GurpFileLine),
//     // Gem(GurpGem),
//     Misc(GurpMisc),
//     // Pkg(GurpPkg),
//     Smf(Box<GurpSmf>),
//     Svc(GurpSvc),
//     Symlink(GurpSymlink),
//     User(GurpUser),
//     Zfs(GurpZfs),
// }

// #[derive(Deserialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "lowercase")]
// pub enum Action {
//     Ensure,
//     Remove,
// }

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

#[derive(Deserialize, Debug)]
pub struct HostConfig {
    pub metadata: HostMetadata,
    pub resources: HostResources,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct HostMetadata {
    pub name: String,
}

#[derive(Default, Deserialize, Debug)]
pub struct EnsureResources {
    #[serde(default)]
    pub cron: Vec<GurpCronEnsure>,
    // #[serde(default)]
    // pub directory: Vec<GurpDirectory>,
    // #[serde(default)]
    // pub file: Vec<GurpFile>,
    #[serde(default)]
    pub file_line: Vec<GurpFileLineEnsure>,
    #[serde(default)]
    pub misc: Vec<GurpMiscEnsure>,
    // #[serde(default)]
    // pub pkg: Vec<GurpPkg>,
    #[serde(default)]
    pub smf: Vec<GurpSmfEnsure>,
    #[serde(default)]
    pub svc: Vec<GurpSvcEnsure>,
    #[serde(default)]
    pub symlink: Vec<GurpSymlinkEnsure>,
    #[serde(default)]
    pub user: Vec<GurpUserEnsure>,
    #[serde(default)]
    pub zfs: Vec<GurpZfsEnsure>,
}

#[derive(Default, Deserialize, Debug)]
pub struct RemoveResources {
    #[serde(default)]
    pub cron: Vec<GurpCronRemove>,
    // #[serde(default)]
    // pub directory: Vec<GurpDirectory>,
    // #[serde(default)]
    // pub file: Vec<GurpFile>,
    #[serde(default)]
    pub file_line: Vec<GurpFileLineRemove>,
    // #[serde(default)]
    // pub pkg: Vec<GurpPkg>,
    #[serde(default)]
    pub smf: Vec<GurpSmfRemove>,
    #[serde(default)]
    pub symlink: Vec<GurpSymlinkRemove>,
    #[serde(default)]
    pub user: Vec<GurpUserRemove>,
    #[serde(default)]
    pub zfs: Vec<GurpZfsRemove>,
}

#[derive(Deserialize, Debug)]
pub struct HostResources {
    #[serde(default)]
    pub ensure: EnsureResources,
    #[serde(default)]
    pub remove: RemoveResources,
}

pub type ChangedIds = HashSet<String>;

#[derive(Default)]
pub struct ApplyContext {
    pub changed_ids: ChangedIds,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Deserialize, Debug, Hash)]
#[serde(rename_all = "kebab-case")]
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
#[derive(Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct SmfDefinitionDependencySvc {
    pub name: String,
    pub restart_on: String,
    pub fmri: String,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Deserialize, Debug, Hash)]
pub struct SmfDefinitionExecMethod {
    pub exec: String,
    pub timeout: u32,
    pub context: Option<SmfDefinitionExecMethodContext>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Deserialize, Debug, Hash)]
pub struct SmfDefinitionExecMethodContext {
    pub user: String,
    pub group: Option<String>,
    pub privileges: Option<String>,
}
