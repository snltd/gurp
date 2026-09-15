// Macros to make sure output is consistent across the doer.
//
macro_rules! log_no_change {
    ($path:expr) => {
        tracing::debug!("{}: has correct content", $path);
    };
}

macro_rules! log_updating {
    ($path:expr) => {
        tracing::info!("{}: updating content", $path);
    };
}

macro_rules! log_creating {
    ($path:expr) => {
        tracing::debug!("{}: creating", $path);
    };
}

macro_rules! apply_summary {
    ($changed:expr) => {
        if $changed {
            Ok(common::constants::ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(common::constants::ONE_RESOURCE_NO_CHANGE)
        }
    };
}
