use crate::file::actions;
use crate::file::types::{CompareMethod, DesiredFileState};
use anyhow::Context;
use camino::Utf8Path;
use common::types::{ApplyOpts, ApplySummary};

pub fn run(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<ApplySummary> {
    let new_content = desired_state
        .content
        .as_ref()
        .context("no content for {path}")?;

    Ok(ApplySummary {
        resources: 1,
        changes: actions::ensure_content(path, new_content, desired_state, compare, opts)?
            + actions::ensure_metadata(path, desired_state, opts)?,
    })
}
