use crate::file::actions;
use crate::file::types::{CompareMethod, DesiredFileState};
use anyhow::{Context, ensure};
use camino::Utf8Path;
use common::types::{ApplyOpts, ApplySummary};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use url::Url;
use util::filter::FileFilter;
use util::{atomic_write, hash};

// This only deals with truly local files. :from resources in client/server mode are
// rewritten to :from-url by the front-end.

pub fn run(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<ApplySummary> {
    let mut changed = false;
    let src = desired_state.from.as_ref().context("no source file name")?;
    ensure!(src.exists(), "Missing source file: {path}");
    let backup_suffix = desired_state.backup_suffix.as_deref();

    let replacements = desired_state
        .url_replacements
        .as_ref()
        .filter(|m| !m.is_empty());

    // Fast path: no replacements, no filter — never materialize a String.
    if replacements.is_none() && matches!(compare, CompareMethod::Hash) {
        if path.exists() {
            if hash::of_file(src)? == hash::of_file(path)? {
                log_no_change!(path);
            } else {
                changed = true;
                log_updating!(path);
                if !opts.noop {
                    fs::copy(src, path)
                        .with_context(|| format!("failed to copy from {src} to {path}"))?;
                }
            }
        } else {
            changed = true;
            log_creating!(path);
            copy_file(src, path, backup_suffix, opts)
                .with_context(|| format!("failed to copy from {src} to {path}"))?;
        }
    } else {
        let content = resolve_content(src, replacements)?;
        let comparable = comparison_view(&content, compare)?;

        let same = path.exists()
            && hash::of_string(&comparable)
                == hash::of_string(&comparison_view(&fs::read_to_string(path)?, compare)?);

        if same {
            log_no_change!(path);
        } else {
            changed = true;
            if path.exists() {
                log_updating!(path);
            } else {
                log_creating!(path);
            }
            atomic_write::install(path, backup_suffix, opts, |f| {
                f.write_all(content.as_bytes())
                    .with_context(|| format!("failed_to_write {path}"))
            })?;
        }
    }

    if actions::ensure_metadata(path, desired_state, opts)? {
        changed = true;
    }
    apply_summary!(changed)
}

fn resolve_content(
    src: &Utf8Path,
    replacements: Option<&HashMap<String, Url>>,
) -> anyhow::Result<String> {
    let raw = fs::read_to_string(src).with_context(|| format!("failed to read {src} as UTF-8"))?;
    match replacements {
        Some(r) => actions::fill_in_url_replacements(raw, r),
        None => Ok(raw),
    }
}

fn comparison_view(content: &str, compare: &CompareMethod) -> anyhow::Result<String> {
    match compare {
        CompareMethod::Hash => Ok(content.to_owned()),
        CompareMethod::Filter(pattern) => Ok(FileFilter::from(pattern)?.string(content)),
    }
}

fn copy_file(
    src: &Utf8Path,
    dest: &Utf8Path,
    backup_suffix: Option<&str>,
    opts: &ApplyOpts,
) -> anyhow::Result<bool> {
    atomic_write::install(dest, backup_suffix, opts, |f| {
        let mut source =
            File::open(src).with_context(|| format!("failed to read source file {src}"))?;
        std::io::copy(&mut source, f).with_context(|| format!("failed copy {src} -> {dest}"))?;
        Ok(())
    })
}

#[cfg(test)]
mod test {
    use crate::file::ensure::FileEnsure;
    use crate::file::types::DesiredFileState;
    use camino_tempfile_ext::prelude::*;
    use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
    use common::types::ApplyOpts;
    use httpmock::prelude::*;
    use indoc::{formatdoc, indoc};
    use os_types::{FileMode, GurpId};
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

        let sut = FileEnsure {
            id: GurpId::new("/NO-ROLE/file/irrelevant").unwrap(),
            path: temp_file.clone(),
            desired_state: DesiredFileState {
                mode: FileMode::new("0755").unwrap(),
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
                url_replacements: None,
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

        let sut = FileEnsure {
            id: GurpId::new("/NO-ROLE/file/irrelevant").unwrap(),
            path: temp_file.clone(),
            desired_state: DesiredFileState {
                group: NameOrId::Name(my_group()),
                owner: NameOrId::Name(my_user()),
                mode: FileMode::new("2755").unwrap(),
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
                url_replacements: None,
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

        let sut: FileEnsure = serde_json::from_str(&json_def).unwrap();
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

        let sut: FileEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
        assert_eq!(content, fs::read_to_string(&temp_file).unwrap());
    }

    #[test]
    fn test_file_create_from_file_with_url_substitution() {
        let server = MockServer::start();

        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/replacement");
            then.status(200)
                .header("content-type", "text/plain")
                .body("hunter2");
        });

        let content = "my password is __PASSWORD__";
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir.child("test-file").write_str(content).unwrap();

        let temp_file = temp_dir.path().join("test-file");

        fs::set_permissions(&temp_file, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :from "{}"
                :mode "0750"
                :url-replacements {{ "__PASSWORD__"  "{}" }}
                :owner "{}"
                :group "{}")
            "#,
            temp_file,
            fixture("file/from-file-example"),
            server.url("/replacement"),
            my_user(),
            my_group(),
        });

        let sut: FileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
        assert!(temp_file.exists());
        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o750);
        assert_eq!(
            indoc! { r#"
                some-value 123
                another-value abc
                password hunter2
            "#},
            fs::read_to_string(temp_file).unwrap()
        );
        conf_mock.assert();
    }
}
