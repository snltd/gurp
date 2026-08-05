use crate::file::types::{CompareMethod, DesiredFileState};
use anyhow::Context;
use camino::Utf8Path;
use common::info;
use common::types::{ApplyOpts, Changed};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use url::Url;
use util::file::{self, FileMetadata};
use util::filter::FileFilter;
use util::http;
use util::{atomic_write, hash};

pub fn ensure_content(
    path: &Utf8Path,
    new_content: &str,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<Changed> {
    let mut changed = false;

    if path.exists() {
        match compare {
            CompareMethod::Hash => {
                if hash::of_string(new_content) == hash::of_file(path)? {
                    log_no_change!(path);
                } else {
                    changed = true;
                    log_updating!(path);
                    write_text_file(
                        path,
                        new_content,
                        desired_state.backup_suffix.as_deref(),
                        opts,
                    )?;
                }
            }
            CompareMethod::Filter(pattern) => {
                let filter = FileFilter::from(pattern)?;

                if hash::of_string(&filter.string(new_content))
                    == hash::of_string(&filter.file(path)?)
                {
                    log_no_change!(path);
                } else {
                    changed = true;
                    log_updating!(path);
                    write_text_file(
                        path,
                        new_content,
                        desired_state.backup_suffix.as_deref(),
                        opts,
                    )?;
                }
            }
        }
    } else {
        changed = true;
        log_creating!(path);
        write_text_file(
            path,
            new_content,
            desired_state.backup_suffix.as_deref(),
            opts,
        )?;
    }

    Ok(changed)
}

/// Blat a string to disk
pub fn write_text_file(
    path: &Utf8Path,
    content: &str,
    backup_suffix: Option<&str>,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    if opts.output.dump_diffs {
        let existing_content = if path.exists() {
            fs::read_to_string(path).with_context(|| format!("failed to read from {path}"))?
        } else {
            String::new()
        };

        println!(
            "{}",
            info::dump_diff(
                &existing_content,
                content,
                Some(path.as_str()),
                &opts.output
            )
        );
    }

    atomic_write::install(path, backup_suffix, opts, |f| {
        f.write_all(content.as_bytes())
            .with_context(|| format!("failed_to_write {path}"))
    })?;

    Ok(())
}

pub fn ensure_metadata(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    opts: &ApplyOpts,
) -> anyhow::Result<bool> {
    file::ensure_metadata(
        path,
        FileMetadata {
            group: &desired_state.group,
            mode: &desired_state.mode,
            owner: &desired_state.owner,
        },
        opts,
    )
}

/// Replace strings in the given content with strings fetched from URLs.
pub fn fill_in_url_replacements(
    content: String,
    replacements: &HashMap<String, Url>,
) -> anyhow::Result<String> {
    let mut modified = content.to_owned().clone();

    for (pattern, url) in replacements {
        tracing::debug!("filling in URL replacement '{pattern}'");
        let replacement = http::url_to_string(url)
            .with_context(|| format!("cannot fetch remote string from {url}"))?;

        modified = modified.replace(pattern, &replacement);
    }

    Ok(modified)
}
