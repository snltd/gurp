// Macros to make sure output is consistent across the doer.
//
macro_rules! log_no_change {
    ($path:expr) => {
        tracing::debug!("{}: has correct content", $path);
    };
}

macro_rules! log_updating {
    ($path:expr) => {
        tracing::debug!("{}: updating content", $path);
    };
}

macro_rules! log_creating {
    ($path:expr) => {
        tracing::debug!("{}: has ", $path);
    };
}
