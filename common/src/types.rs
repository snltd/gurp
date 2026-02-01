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

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct ApplySummary {
    pub resources: u32,
    pub changes: u32,
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

#[derive(Debug)]
pub struct FileMetadata<'a> {
    pub group: &'a str,
    pub mode: &'a str,
    pub owner: &'a str,
    pub path: &'a Utf8PathBuf,
    pub changes: u32,
}

pub type VlanID = u16;
pub type ChangedIds = BTreeSet<String>;
