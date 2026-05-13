use crate::file::actions;
use crate::file::types::{CompareMethod, DesiredFileState};
use anyhow::{Context, ensure};
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
    let mut changed = false;
    let source = desired_state.from.as_ref().context("no source file name")?;
    ensure!(source.exists(), "Missing source file: {path}");

    if path.exists() {
        match compare {
            CompareMethod::Hash => {
                if hash::of_file(source)? == hash::of_file(path)? {
                    log_no_change!(path);
                } else {
                    changed = true;
                    log_updating!(path);

                    if !opts.noop {
                        fs::copy(source, path)
                            .with_context(|| format!("failed to copy from {source} to {path}"))?;
                    }
                }
            }
            CompareMethod::Filter(pattern) => {
                let filter = FileFilter::from(pattern)?;

                if hash::of_string(&filter.file(source)?) == hash::of_string(&filter.file(path)?) {
                    log_no_change!(path);
                } else {
                    changed = true;
                    log_updating!(path);

                    if !opts.noop {
                        fs::copy(source, path)
                            .with_context(|| format!("failed to copy from {source} to {path}"))?;
                    }
                }
            }
        }
    } else {
        changed = true;
        log_creating!(path);

        if !opts.noop {
            fs::copy(source, path)
                .with_context(|| format!("failed to copy from {source} to {path}"))?;
        }
    }

    if actions::ensure_metadata(path, desired_state, opts)? {
        changed = true;
    }

    apply_summary!(changed)
}

#[cfg(test)]
mod test {
    use crate::file::ensure::GurpFileEnsure;
    use crate::file::types::DesiredFileState;
    use camino_tempfile_ext::prelude::*;
    use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
    use common::types::ApplyOpts;
    use indoc::formatdoc;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tester::{fixture, janet2json, my_group, my_user};
    use util::file::NameOrId;

    #[test]
    fn test_create_binary_file() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("binary-file");

        assert!(!temp_file.exists());

        let sut = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: temp_file.clone(),
            desired_state: DesiredFileState {
                mode: "0755".to_owned(),
                group: NameOrId::Name(my_group()),
                owner: NameOrId::Name(my_user()),
                content: None,
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                only_fetch_from_url_once: false,
                url_is_server: false,
            },
        };

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
        assert!(temp_file.exists());
    }

    #[test]
    fn test_create_binary_file_setuid() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("binary-file");

        assert!(!temp_file.exists());

        let sut = GurpFileEnsure {
            id: "IRRELEVANT".to_owned(),
            path: temp_file.clone(),
            desired_state: DesiredFileState {
                group: NameOrId::Name(my_group()),
                owner: NameOrId::Name(my_user()),
                mode: "2755".to_owned(),
                content: None,
                ignore_pattern: None,
                from: Some(fixture("file/binary-file")),
                backup_suffix: None,
                from_struct: None,
                to_format: None,
                from_url: None,
                with_checksum: None,
                only_fetch_from_url_once: false,
                url_is_server: false,
            },
        };

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );

        let metadata = fs::metadata(&temp_file).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o7777, 0o2755);
        assert!(temp_file.exists());
    }

    #[test]
    fn test_update_file_from_file_and_set_mode() {
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir
            .child("test-file")
            .write_str("the-wrong-stuff")
            .unwrap();

        let temp_file = temp_dir.path().join("test-file");

        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (do
            (setdyn :gurp-config-root "{}")
            (file/ensure "{}"
                :from "{}"
                :mode "0444"
                :owner "{}"
                :group "{}"))
            "#,
            temp_file.parent().unwrap(),
            temp_file,
            &fixture("file/copy-file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );

        assert!(temp_file.exists());
        assert_eq!(
            "some-content\n".to_owned(),
            fs::read_to_string(&temp_file).unwrap()
        );

        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o444);
    }

    #[test]
    fn test_ignored_line_means_no_change_with_from() {
        let content = "today is 2015-01-30\nBut this never changes.\nAnd nor does this.\n";
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir.child("test-file").write_str(content).unwrap();

        let temp_file = temp_dir.path().join("test-file");

        fs::set_permissions(&temp_file, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :from "{}"
                :mode "0600"
                :ignore-pattern "^today is"
                :owner "{}"
                :group "{}")
            "#,
            temp_file,
            fixture("file/ignore-line-file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
        assert_eq!(content, fs::read_to_string(&temp_file).unwrap());
    }
}
