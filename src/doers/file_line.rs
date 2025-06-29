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
    #[serde(flatten)]
    pub desired_state: FileLineState,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpFileLineRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub line: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf, // The Path
    #[serde(flatten)]
    pub desired_state: FileLineState,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct FileLineState {
    pub line: String,
}

fn line_exists(path: &Utf8PathBuf, line: &str) -> anyhow::Result<bool> {
    let contents = fs::read_to_string(path)?;
    Ok(contents.lines().any(|l| l == line))
}

impl GurpFileLineEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if line_exists(&self.path, &self.line)? {
            tracing::info!("no change: {}", &self.path);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("creating: {}", &self.path);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                let fh = fs::OpenOptions::new().append(true).open(&self.path)?;
                writeln!(&fh, "\n{}", self.desired_state.line.as_str())?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        }
    }
}

impl GurpFileLineRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if line_exists(&self.path, &self.line)? {
            tracing::info!("no change: {}", &self.path);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("removing: {}", &self.path);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                let content = fs::read_to_string(&self.path)?;

                let out: String = content
                    .lines()
                    .filter(|l| l != &self.desired_state.line)
                    .map(|line| format!("{line}\n"))
                    .collect();

                fs::write(&self.path, out)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        }
    }
}

#[cfg(test)]
mod test {
    /*
    use super::*;
    use crate::test_utils::spec_helper::{defopts, defopts_noop, init_janet};
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use janetrs::{Janet, structs};

    #[test]
    fn test_file_line_ensure_file_does_not_exist() {
        init_janet();
        let resource = Janet::wrap(structs! {
            ":_id" => "/test-role/file-line/test-does-not-exist",
            ":action" => ":ensure",
            ":line" => "some irrelevant text",
            ":name" => "/file/does/not/exist",
        });

        assert!(GurpFileLineEnsure::try_from(&resource).is_err());
    }

    #[test]
    fn test_file_line_ensure_file_does_not_contain_desired_line() {
        init_janet();

        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("line_1\nline_2\nline_3")
            .unwrap();
        let file_to_modify = temp.join("test-file");

        let example_file_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/file-line/test-does-not-exist",
            ":action" => ":ensure",
            ":line" => "line_4",
            ":name" => file_to_modify.to_string_lossy().to_string().as_str(),
        });

        let gurp_file = GurpFileLineEnsure::try_from(&example_file_ensure).unwrap();

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            gurp_file.apply(&defopts()).unwrap()
        );
        assert_eq!(
            "line_1\nline_2\nline_3\nline_4\n".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_ensure_file_does_not_contain_desired_line_noop() {
        init_janet();

        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("line_1\nline_2\nline_3")
            .unwrap();
        let file_to_modify = temp.join("test-file");

        let example_file_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/file-line/test-does-not-exist",
            ":action" => ":ensure",
            ":line" => "line_4",
            ":name" => file_to_modify.to_string_lossy().to_string().as_str(),
        });

        let gurp_file = GurpFileLineEnsure::try_from(&example_file_ensure).unwrap();

        assert_eq!(ONE_RESOURCE_NOOP, gurp_file.apply(&defopts_noop()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_ensure_file_contains_desired_line() {
        init_janet();

        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("line_1\nline_2\nline_3")
            .unwrap();
        let file_to_modify = temp.join("test-file");

        let example_file_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/file-line/test-does-not-exist",
            ":action" => ":ensure",
            ":line" => "line_3",
            ":name" => file_to_modify.to_string_lossy().to_string().as_str(),
        });

        let gurp_file = GurpFileLineEnsure::try_from(&example_file_ensure).unwrap();

        assert_eq!(ONE_RESOURCE_NO_CHANGE, gurp_file.apply(&defopts()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_remove_file_contains_desired_line() {
        init_janet();

        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("line_1\nline_2\nline_3\n")
            .unwrap();
        let file_to_modify = temp.join("test-file");

        let example_file_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/file-line/test-does-not-exist",
            ":action" => ":remove",
            ":line" => "line_2",
            ":name" => file_to_modify.to_string_lossy().to_string().as_str(),
        });

        let gurp_file = GurpFileLineRemove::try_from(&example_file_ensure).unwrap();

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            gurp_file.apply(&defopts()).unwrap()
        );
        assert_eq!(
            "line_1\nline_3\n".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_remove_file_does_not_contain_desired_line() {
        init_janet();

        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("line_1\nline_2\nline_3")
            .unwrap();
        let file_to_modify = temp.join("test-file");

        let example_file_ensure = Janet::wrap(janetrs::structs! {
            ":_id" => "/test-role/file-line/test-does-not-exist",
            ":action" => ":remove",
            ":line" => "line_4",
            ":name" => file_to_modify.to_string_lossy().to_string().as_str(),
        });

        let gurp_file = GurpFileLineRemove::try_from(&example_file_ensure).unwrap();

        assert_eq!(ONE_RESOURCE_NOOP, gurp_file.apply(&defopts_noop()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }
    */
}
