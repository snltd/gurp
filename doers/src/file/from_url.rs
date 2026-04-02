use crate::file::actions;
use crate::file::types::{CompareMethod, DesiredFileState};
use anyhow::Context;
use camino::Utf8Path;
use camino_tempfile::NamedUtf8TempFile;
use common::types::{ApplyOpts, ApplySummary, Changes};
use std::fs;
use util::{filter, hash, http};

pub fn run(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<ApplySummary> {
    let changes = if desired_state.url_is_server {
        file_from_server(path, desired_state, compare, opts)
    } else {
        file_from_remote(path, desired_state, compare, opts)
    }?;

    Ok(ApplySummary {
        resources: 1,
        changes: changes + actions::ensure_metadata(path, desired_state, opts)?,
    })
}

fn file_from_server(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<Changes> {
    let url = desired_state.from_url.as_ref().context("no :from-url")?;
    let mut changes = 0;

    if path.exists() {
        match compare {
            CompareMethod::Hash => {
                if hash::for_remote_file(url)? == hash::of_file(path)?.to_string() {
                    log_no_change!(path);
                } else {
                    log_updating!(path);
                    changes = 1;

                    if !opts.noop {
                        http::remote_file_to_disk(url, path)?;
                    }
                }
            }
            CompareMethod::Filter(pattern) => {
                if hash::for_remote_filtered_file(url, pattern)? == hash::of_file(path)?.to_string()
                {
                    log_no_change!(path);
                } else {
                    log_updating!(path);
                    changes = 1;

                    if !opts.noop {
                        http::remote_file_to_disk(url, path)?;
                    }
                }
            }
        }
    } else {
        changes = 1;
        log_creating!(path);
        http::remote_file_to_disk(url, path)?;
    }

    Ok(changes)
}

fn file_from_remote(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<Changes> {
    let url = desired_state.from_url.as_ref().context("no :from-url")?;
    let mut changes = 0;
    let source = desired_state
        .from_url
        .as_ref()
        .context("no source file name")?;

    if path.exists() {
        if desired_state.only_fetch_from_url_once {
            tracing::debug!("{path} exists and :only-fetch-from-url-once is set");
        } else {
            let tmpfile = NamedUtf8TempFile::new()?;
            let temp_path = tmpfile.path();
            tracing::debug!("downloading {url} to {temp_path} for comparison");
            http::remote_file_to_disk(url, temp_path)?;

            match compare {
                CompareMethod::Hash => {
                    if hash::of_file(temp_path)? == hash::of_file(path)? {
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
                    let filter = filter::FileFilter::from(pattern)?;

                    if hash::of_string(&filter.file(temp_path)?)
                        == hash::of_string(&filter.file(path)?)
                    {
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
        }
    } else {
        changes = 1;
        log_creating!(path);
        http::remote_file_to_disk(url, path)?;
    }

    Ok(changes)
}
