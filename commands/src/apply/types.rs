use common::types::{ApplySummary, CompileError};

pub(crate) enum ApplyStatus {
    Ok(ApplySummary),
    Fail(FailPhase),
}

pub(crate) enum FailPhase {
    Apply,
    Client,
    Compile,
    FileNotFound,
    Locked,
    Network,
}

impl std::fmt::Display for FailPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            FailPhase::Apply => "apply",
            FailPhase::Client => "client",
            FailPhase::Compile => "compile",
            FailPhase::FileNotFound => "fileNotFound",
            FailPhase::Locked => "locked",
            FailPhase::Network => "connect",
        };
        write!(f, "{}", s)
    }
}

impl From<CompileError> for FailPhase {
    fn from(e: CompileError) -> Self {
        match e {
            CompileError::Network(_) => FailPhase::Network,
            CompileError::FileNotFound(_) => FailPhase::FileNotFound,
            CompileError::ClientCreate(_) => FailPhase::Client,
            CompileError::Compile(_) => FailPhase::Compile,
            CompileError::Other(_) => FailPhase::Compile,
            CompileError::Io(_) => FailPhase::Compile,
        }
    }
}
