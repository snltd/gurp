use common::types::{ApplySummary, CompileError, NetworkError};
use serde::Serialize;

pub(crate) enum ApplyStatus {
    Ok(ApplySummary),
    Fail(FailPhase),
}

#[derive(Serialize)]
pub(crate) enum FailPhase {
    Apply,
    Client,
    Compile,
    FileNotFound,
    Locked,
    Network(String),
}

impl std::fmt::Display for FailPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            FailPhase::Apply => "apply",
            FailPhase::Client => "client",
            FailPhase::Compile => "compile",
            FailPhase::FileNotFound => "fileNotFound",
            FailPhase::Locked => "locked",
            FailPhase::Network(s) => &format!("network-{s}"),
        };
        write!(f, "{}", s)
    }
}

impl From<CompileError> for FailPhase {
    fn from(e: CompileError) -> Self {
        match e {
            CompileError::Network(e) => match e {
                NetworkError::Http(code) => FailPhase::Network(code.to_string()),
                NetworkError::Transport(_) => FailPhase::Network("transport error".to_string()),
            },
            CompileError::FileNotFound(_) => FailPhase::FileNotFound,
            CompileError::ClientCreate(_) => FailPhase::Client,
            CompileError::Other(_) => FailPhase::Compile,
            CompileError::Io(_) => FailPhase::Compile,
            CompileError::Compile {
                message: _,
                trace: _,
            } => FailPhase::Compile,
        }
    }
}
