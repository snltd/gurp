use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::types::{ApplySummary, Opts};
use camino::Utf8PathBuf;
use serde::Deserialize;
use std::fs;
use std::io::Write;

// THINGS TO KNOW / THINGS TO DO.
// File is not managed here. Use a file resource.
// This is super-basic. It appends lines and removes lines. That's it.
// Doesn't even do regex. Exact matches only.
// It reads the entirety of the file into memory.
// Appended lines have a \n at the beginning and end.
// Removing a line puts a newline on the end of the file if there wasn't one already.
// We always read the file. There's no caching or anyhing.
// Files are not backed up.

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpFileLineEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub line: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpFileLineRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub line: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

fn line_exists(path: &Utf8PathBuf, line: &str) -> anyhow::Result<bool> {
    let contents = fs::read_to_string(path)?;
    Ok(contents.lines().any(|l| l == line))
}

impl GurpFileLineEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if line_exists(&self.path, &self.line)? {
            tracing::debug!("no change: {}", &self.path);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("creating: {}", &self.path);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                let fh = fs::OpenOptions::new().append(true).open(&self.path)?;
                writeln!(&fh, "\n{}\n", self.line.as_str())?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        }
    }
}

impl GurpFileLineRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if line_exists(&self.path, &self.line)? {
            tracing::info!("removing: {}", &self.path);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                let content = fs::read_to_string(&self.path)?;

                let out: String = content
                    .lines()
                    .filter(|l| l != &self.line)
                    .map(|line| format!("{line}\n"))
                    .collect();

                fs::write(&self.path, out)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            tracing::debug!("no change: {}", &self.path);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::{defopts, defopts_noop, janet2json};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use indoc::{formatdoc, indoc};

    #[test]
    fn test_file_line_ensure_file_does_not_exist() {
        let json_def = janet2json(indoc! {r#"
            (file-line/ensure "/test-role/file-line/test-does-not-exist"
                :line "some irrelevant text")
                "#});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();
        assert!(sut.apply(&defopts()).is_err());
    }

    #[test]
    fn test_file_line_ensure_file_does_not_contain_desired_line() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/ensure \"{}\" :line \"line_4\")
            ", file_to_modify});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3\nline_4\n".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_ensure_file_does_not_contain_desired_line_noop() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/ensure \"{}\" :line \"line_4\")
            ", file_to_modify});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_ensure_file_contains_desired_line() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/ensure \"{}\" :line \"line_3\")
            ", file_to_modify});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_remove_file_contains_desired_line() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/remove \"{}\" :line \"line_2\")
            ", file_to_modify});

        let sut: GurpFileLineRemove = serde_json::from_str(&json_def).unwrap();

        sut.apply(&defopts()).unwrap();
        // assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(
            "line_1\nline_3\n".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_remove_file_does_not_contain_desired_line() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("line_1\nline_2\nline_3")
            .unwrap();
        let file_to_modify = temp.join("test-file");

        let json_def = janet2json(&formatdoc! {"
            (file-line/remove \"{}\" :line \"line_4\")
            ", file_to_modify.to_string_lossy()});

        let sut: GurpFileLineRemove = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    fn test_file() -> (TempDir, Utf8PathBuf) {
        let temp = TempDir::new().unwrap();
        let file = temp.child("test-file");
        file.write_str("line_1\nline_2\nline_3").unwrap();
        (
            temp,
            Utf8PathBuf::from_path_buf(file.path().to_path_buf()).unwrap(),
        )
    }
}
