use common::types::{ApplySummary, CompileError};

pub(crate) enum ApplyStatus {
    Ok(ApplySummary),
    Fail(FailPhase),
}

pub(crate) enum FailPhase {
    FileNotFound,
    Network,
    Compile,
    Apply,
    Locked,
}

impl std::fmt::Display for FailPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            FailPhase::Network => "connect",
            FailPhase::Compile => "compile",
            FailPhase::Apply => "apply",
            FailPhase::Locked => "locked",
            FailPhase::FileNotFound => "fileNotFound",
        };
        write!(f, "{}", s)
    }
}

impl From<&CompileError> for FailPhase {
    fn from(e: &CompileError) -> Self {
        match e {
            CompileError::Network(_) => FailPhase::Network,
            CompileError::FileNotFound(_) => FailPhase::FileNotFound,
            CompileError::ClientCreate(_) => FailPhase::Compile,
            CompileError::Compile(_) => FailPhase::Compile,
            CompileError::Other(_) => FailPhase::Compile,
            CompileError::Io(_) => FailPhase::Compile,
        }
    }
}
