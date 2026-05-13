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

    let mut changed = actions::ensure_content(path, new_content, desired_state, compare, opts)?;

    if actions::ensure_metadata(path, desired_state, opts)? {
        changed = true;
    }

    apply_summary!(changed)
}

#[cfg(test)]
mod test {
    use crate::file::ensure::GurpFileEnsure;
    use camino_tempfile_ext::prelude::*;
    use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
    use common::types::ApplyOpts;
    use indoc::formatdoc;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tester::{janet2json, my_group, my_user};

    #[test]
    fn test_file_create_from_content_noop() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.child("test-file");

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "some-junk"
                :mode "0750"
                :owner "{}"
                :group "{}")
            "#,
            temp_file.as_path(),
            my_user(),
            my_group(),
        });

        assert!(!temp_file.exists());
        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts {
                noop: true,
                ..Default::default()
            })
            .unwrap()
        );
        assert!(!temp_file.exists());
    }

    #[test]
    fn test_file_create_from_content() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test-file");

        assert!(!temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "stuff"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
            temp_file.as_path(),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
        assert!(temp_file.exists());

        let metadata = fs::metadata(&temp_file).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o7777, 0o640);
        assert_eq!("stuff", fs::read_to_string(temp_file).unwrap());
    }

    #[test]
    fn test_file_create_from_content_template() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test-file");

        assert!(!temp_file.exists());

        // escape { with another {
        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content (template-out "{{{{ name }}}} is running a {{{{ thing }}}}"
                                        {{ :name "gurp" :thing "test" }})
                :mode "0600"
                :owner "{}"
                :group "{}")
            "#,
            temp_file.as_path(),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
        assert!(temp_file.exists());
        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        assert_eq!(
            "gurp is running a test",
            fs::read_to_string(temp_file).unwrap()
        );
    }

    #[test]
    fn test_update_file_from_content_and_set_mode() {
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir
            .child("test-file")
            .write_str("the-wrong-stuff")
            .unwrap();

        let temp_file = temp_dir.path().join("test-file");

        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "the-right-stuff"
                :mode "0400"
                :owner "{}"
                :group "{}")
            "#,
            temp_file,
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
            "the-right-stuff".to_owned(),
            fs::read_to_string(&temp_file).unwrap()
        );

        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o0400);
    }

    #[test]
    fn test_ignored_line_means_no_change_with_content() {
        let content = "today is 2015-01-30\nBut this never changes.\nAnd nor does this.\n";
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir.child("test-file").write_str(content).unwrap();

        let temp_file = temp_dir.path().join("test-file");

        fs::set_permissions(&temp_file, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! { r#"
            (file/ensure "{}"
                :content "today is 2025-06-26\nBut this never changes.\nAnd nor does this.\n"
                :mode "0600"
                :ignore-pattern "^today is"
                :owner "{}"
                :group "{}")
            "#,
            temp_file,
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

    #[test]
    fn test_file_ensure_from_content_already_correct() {
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir.child("test-file").write_str("stuff").unwrap();
        let temp_file = temp_dir.path().join("test-file");

        fs::set_permissions(&temp_file, fs::Permissions::from_mode(0o0750)).unwrap();
        assert!(temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{}"
                :content "stuff"
                :mode "0750"
                :owner "{}"
                :group "{}")
            "#,
            temp_file,
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            sut.apply(&ApplyOpts::default()).unwrap()
        );
        assert!(temp_file.exists());
    }
}
