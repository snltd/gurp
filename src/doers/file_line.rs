use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::anyhow;
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray};
use paste::paste;
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

#[derive(Debug, PartialEq, Eq)]
pub struct GurpFileLine {
    pub action: Action,
    pub exists: bool,
    pub id: String,
    pub name: Utf8PathBuf, // The Path
    pub desired_state: FileLineState,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileLineState {
    pub line: String,
}

crate::unpack_fn!(ensure_list, FileLine, GurpFileLine);
crate::unpack_fn!(remove_list, FileLine, GurpFileLine);
crate::impl_apply!(GurpFileLine);

impl TryFrom<&Janet> for GurpFileLine {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let name = data.get_field_pathbuf("name")?;

        if !name.exists() {
            return Err(anyhow!("File {} does not exist", name));
        }

        let action = janet_helpers::action_as_enum(&data)?;
        let line = data.get_field_string("line")?;
        let contents = fs::read_to_string(name)?;
        let exists = contents.lines().any(|l| l == line);
        let state = FileLineState { line };

        Ok(GurpFileLine {
            action,
            exists,
            id: data.get_field_string("_id")?,
            name: data.get_field_pathbuf("name")?,
            desired_state: state,
        })
    }
}

impl GurpFileLine {
    fn apply_ensure(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        if self.exists {
            tracing::info!("no change: {}", &self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("creating: {}", &self.name);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                let fh = fs::OpenOptions::new().append(true).open(&self.name)?;
                writeln!(&fh, "\n{}", self.desired_state.line.as_str())?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        }
    }

    fn apply_remove(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        if !self.exists {
            tracing::info!("no change: {}", &self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("removing: {}", &self.name);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                let content = fs::read_to_string(&self.name)?;

                let out: String = content
                    .lines()
                    .filter(|l| l != &self.desired_state.line)
                    .map(|line| format!("{}\n", line))
                    .collect();

                fs::write(&self.name, out)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::{defcontext, defopts, defopts_noop, init_janet};
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

        assert!(GurpFileLine::try_from(&resource).is_err());
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

        let gurp_file = GurpFileLine::try_from(&example_file_ensure).unwrap();

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            gurp_file.apply(&defcontext(), &defopts()).unwrap()
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

        let gurp_file = GurpFileLine::try_from(&example_file_ensure).unwrap();

        assert_eq!(
            ONE_RESOURCE_NOOP,
            gurp_file.apply(&defcontext(), &defopts_noop()).unwrap()
        );
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

        let gurp_file = GurpFileLine::try_from(&example_file_ensure).unwrap();

        assert_eq!(
            ONE_RESOURCE_NO_CHANGE,
            gurp_file.apply(&defcontext(), &defopts()).unwrap()
        );
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

        let gurp_file = GurpFileLine::try_from(&example_file_ensure).unwrap();

        assert_eq!(
            ONE_RESOURCE_ONE_CHANGE,
            gurp_file.apply(&defcontext(), &defopts()).unwrap()
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

        let gurp_file = GurpFileLine::try_from(&example_file_ensure).unwrap();

        assert_eq!(
            ONE_RESOURCE_NOOP,
            gurp_file.apply(&defcontext(), &defopts_noop()).unwrap()
        );
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }
}
