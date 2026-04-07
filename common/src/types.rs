use camino::Utf8PathBuf;
use std::collections::BTreeSet;
use std::ops::{Add, AddAssign};

#[derive(Debug, Default)]
pub struct ApplyOpts {
    pub noop: bool,
    pub colour: bool,
    pub line_no: bool,
    pub dump_config: bool,
    pub dump_diffs: bool,
    pub compile_only: bool,
    pub metrics_to: Option<String>,
    pub precompiled: bool,
    pub image: bool,
    pub server: Option<String>,      // client mode only
    pub as_json: bool,               // client mode only
    pub hostname: Option<String>,    // currently client mode only
    pub server_name: Option<String>, // server mode only
    pub client_name: Option<String>, // server mode only
    pub destroy: bool,
    pub exec: Option<String>,
    pub no_lock: bool,
    pub remove_first: bool,
    pub only: Option<String>,
}

#[derive(Debug, Default)]
pub struct CompileOpts {
    pub format: String,
    pub output_file: Option<Utf8PathBuf>,
}

#[derive(Debug)]
pub struct ServerOpts {
    pub config_dir: Utf8PathBuf,
    pub metrics_to: Option<String>,
}

pub type Changes = u32;
pub type Resources = u32;
pub type Changed = bool;

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct ApplySummary {
    pub resources: Resources,
    pub changes: Changes,
}

impl Add for ApplySummary {
    type Output = ApplySummary;

    fn add(self, other: ApplySummary) -> ApplySummary {
        ApplySummary {
            resources: self.resources + other.resources,
            changes: self.changes + other.changes,
        }
    }
}

impl AddAssign for ApplySummary {
    fn add_assign(&mut self, other: ApplySummary) {
        self.resources += other.resources;
        self.changes += other.changes;
    }
}

pub type JsonConfig = String;

pub type VlanID = u16;
pub type ChangedIds = BTreeSet<String>;

pub type GurpMetric<'a> = (&'a str, &'a str, u128);
