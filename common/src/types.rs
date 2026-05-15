//! Collections of types mostly relating to user input

use camino::Utf8PathBuf;
use std::collections::BTreeSet;
use std::ops::{Add, AddAssign};

/// CLI options for the apply command
#[derive(Debug, Default)]
pub struct ApplyOpts {
    pub noop: bool,
    pub metrics_to: Option<String>,
    pub precompiled: bool,
    pub image: bool,
    pub destroy: bool,
    pub exec: Option<String>,
    pub no_lock: bool,
    pub remove_first: bool,
    pub only: Option<String>,
    pub output: ApplyOutputOpts,
    pub vm: ApplyVmOpts,
    pub client: ApplyClientOpts,
}

/// User-supplied flags which affect Gurp's output
#[derive(Clone, Debug, Default)]
pub struct ApplyOutputOpts {
    pub colour: bool,
    pub line_no: bool,
    pub dump_configs: bool,
    pub dump_diffs: bool,
}

/// User-suppled flags which affect the behaviour of the Janet VM
#[derive(Clone, Debug, Default)]
pub struct ApplyVmOpts {
    pub define: Vec<String>,
}

/// User-supplied flags which affect behaviour in client mode
#[derive(Debug, Default)]
pub struct ApplyClientOpts {
    pub server: Option<String>,   // client mode only
    pub hostname: Option<String>, // currently client mode only
}

/// CLI options for the compile command
#[derive(Debug, Default)]
pub struct CompileOpts {
    pub colour: bool,
    pub format: String,
    pub line_no: bool,
    pub output_file: Option<Utf8PathBuf>,
}

/// CLI options for the server command
#[derive(Debug)]
pub struct ServerOpts {
    pub config_dir: Utf8PathBuf,
    pub metrics_to: Option<String>,
}

/// Every doer returns an ApplySummary. The apply command sums them for a report.
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

/// Compilation can fail at numerous points. We use this enum to tell the user where.
#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("compilation error: {0}")]
    Compile(#[source] anyhow::Error),
    #[error("network error: {0}")]
    Network(#[source] NetworkError),
    #[error("I/O error: {0}")]
    Io(#[source] std::io::Error),
    #[error("missing file error: {0}")]
    FileNotFound(Utf8PathBuf),
    #[error("client create error: {0}")]
    ClientCreate(#[source] anyhow::Error),
    #[error("compile error: {0}")]
    Other(#[source] anyhow::Error),
}

impl CompileError {
    pub fn is_retryable(&self) -> bool {
        match self {
            CompileError::Network(e) => e.is_retryable(),
            _ => false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("HTTP error: {0}")]
    Http(u16),
    #[error("transport error: {0}")]
    Transport(String),
}

impl NetworkError {
    pub fn is_retryable(&self) -> bool {
        match self {
            NetworkError::Http(code) => matches!(code, 429 | 503 | 504),
            NetworkError::Transport(_) => true,
        }
    }
}

/// Specific types for ApplySummary
pub type Changes = u32;
pub type Resources = u32;
pub type Changed = bool;

/// Specifics to avoid stringly typing things
pub type JsonConfig = String;
pub type VlanID = u16;
pub type ChangedIds = BTreeSet<String>;

pub type GurpMetric<'a> = (&'a str, &'a str, u128);
