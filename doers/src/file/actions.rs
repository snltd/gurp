use crate::file::actions;
use crate::file::types::{CompareMethod, DesiredFileState};
use camino::Utf8Path;
use common::info;
use common::types::{ApplyOpts, Changes};
use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use util::file::{self, FileMetadata, NameOrId};
use util::filter::FileFilter;
use util::hash;

pub fn ensure_content(
    path: &Utf8Path,
    new_content: &str,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<Changes> {
    let mut changes = 0;

    if path.exists() {
        match compare {
            CompareMethod::Hash => {
                if hash::of_string(new_content) == hash::of_file(path)? {
                    log_no_change!(path);
                } else {
                    changes += 1;
                    log_updating!(path);
                    actions::write_text_file(
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
                    changes += 1;
                    log_updating!(path);
                    actions::write_text_file(
                        path,
                        new_content,
                        desired_state.backup_suffix.as_deref(),
                        opts,
                    )?;
                }
            }
        }
    } else {
        changes += 1;
        log_creating!(path);
        actions::write_text_file(
            path,
            new_content,
            desired_state.backup_suffix.as_deref(),
            opts,
        )?;
    }

    Ok(changes)
}

/// Blat a string to disk
pub fn write_text_file(
    path: &Utf8Path,
    content: &str,
    backup_suffix: Option<&str>,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    if let Some(suffix) = backup_suffix
        && path.exists()
    {
        back_up(path, suffix, opts)?;
    }

    if opts.dump_diffs {
        let existing_content = if path.exists() {
            fs::read_to_string(path)?
        } else {
            String::new()
        };

        println!(
            "{}",
            &info::dump_diff(&existing_content, content, Some(path.as_str()), opts.colour)
        );
    }

    if !opts.noop {
        let mut fh = fs::File::create(path)?;
        fh.write_all(content.as_bytes())?;
    }

    Ok(())
}

pub fn back_up(path: &Utf8Path, suffix: &str, opts: &ApplyOpts) -> anyhow::Result<()> {
    let suffix = if suffix == "TIMESTAMP" {
        epoch_time_as_string()
    } else {
        suffix.to_owned()
    };

    let backup_target = path.with_extension(suffix);
    tracing::debug!("Backing up to {}", backup_target);

    if !opts.noop {
        fs::rename(path, &backup_target)?;
        file::ensure_metadata(
            &backup_target,
            FileMetadata {
                group: &NameOrId::Name("root".to_owned()),
                owner: &NameOrId::Name("root".to_owned()),
                mode: "0400",
            },
            opts,
        )?;
    }

    Ok(())
}

pub fn ensure_metadata(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    opts: &ApplyOpts,
) -> anyhow::Result<Changes> {
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

fn epoch_time_as_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("cannot get epoch time")
        .as_secs()
        .to_string()
}
