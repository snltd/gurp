use crate::file::actions;
use crate::file::types::{CompareMethod, DesiredFileState};
use anyhow::Context;
use anyhow::ensure;
use camino::Utf8Path;
use common::types::{ApplyOpts, ApplySummary};
use std::fs;
use util::filter::FileFilter;
use util::hash;

// This only deals with truly local files. :from resources in client/server mode are rewritten to
// :from-url by the front-end.

pub fn run(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<ApplySummary> {
    let mut changes = 0;
    let source = desired_state.from.as_ref().context("no source file name")?;
    ensure!(source.exists(), "Missing source file: {path}");

    if path.exists() {
        match compare {
            CompareMethod::Hash => {
                if hash::of_file(source)? == hash::of_file(path)? {
                    log_no_change!(path);
                } else {
                    changes += 1;
                    log_updating!(path);

                    if !opts.noop {
                        let _bytes = fs::copy(source, path)?;
                    }
                }
            }
            CompareMethod::Filter(pattern) => {
                let filter = FileFilter::from(pattern)?;

                if hash::of_string(&filter.file(source)?) == hash::of_string(&filter.file(path)?) {
                    log_no_change!(path);
                } else {
                    changes += 1;
                    log_updating!(path);

                    if !opts.noop {
                        let _bytes = fs::copy(source, path)?;
                    }
                }
            }
        }
    } else {
        changes += 1;
        log_creating!(path);

        if !opts.noop {
            let _bytes = fs::copy(source, path)?;
        }
    }

    Ok(ApplySummary {
        resources: 1,
        changes: changes + actions::ensure_metadata(path, desired_state, opts)?,
    })
}
