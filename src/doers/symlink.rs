use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::debug;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::bail;
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray};
use paste::paste;
use std::fmt::Debug;
use std::fs;
use std::os::unix;

// THINGS TO KNOW / THINGS TO DO.
// Only does symbolic links.

#[derive(Debug, PartialEq, Eq)]
pub struct GurpSymlink {
    pub action: Action,
    pub exists: bool,
    pub id: String,
    pub name: Utf8PathBuf, // The Path
    pub source: Option<Utf8PathBuf>,
    pub doer: String,
}

impl TryFrom<&Janet> for GurpSymlink {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let name = data.get_field_pathbuf("name")?;
        let exists = name.exists();
        let action = janet_helpers::action_as_enum(&data)?;

        let source = match action {
            Action::Ensure => Some(data.get_field_pathbuf("source")?),
            Action::Remove => None,
        };

        Ok(GurpSymlink {
            action,
            exists,
            id: data.get_field_string("_id")?,
            name: data.get_field_pathbuf("name")?,
            source,
            doer: "symlink".to_owned(),
        })
    }
}

crate::unpack_fn!(ensure_list, Symlink, GurpSymlink);
crate::unpack_fn!(remove_list, Symlink, GurpSymlink);
crate::impl_apply!(GurpSymlink);

impl GurpSymlink {
    fn apply_ensure(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        let target = &self.name;
        let source = self.source.as_ref().unwrap();

        if !source.exists() {
            bail!("source not found: {}", source);
        }

        if !target.exists() {
            output.creating(target);

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                unix::fs::symlink(source, target)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else if target.is_symlink() {
            let current_source = target.read_link_utf8()?;
            if current_source == *source {
                output.no_change(&self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            } else {
                output.change(target, &current_source, source);
                debug!(opts, "doer/symlink", "removing existing link {}", target);
                fs::remove_file(target)?;
                unix::fs::symlink(source, target)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        } else {
            bail!("{} exists and is not a symlink", &target);
        }
    }

    fn apply_remove(
        &self,
        _apply_context: &ApplyContext,
        _opts: &Opts,
        output: &Output,
    ) -> anyhow::Result<ApplySummary> {
        if self.exists {
            output.removing(&self.name);
            fs::remove_file(&self.name)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            output.not_present(&self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::spec_helper::{defopts, defopts_noop};

    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use camino::Utf8PathBuf;
    use std::os::unix;

    fn make_symlink(
        action: Action,
        name: &Utf8PathBuf,
        source: Option<Utf8PathBuf>,
    ) -> GurpSymlink {
        GurpSymlink {
            action,
            exists: name.exists(),
            id: "test-id".to_string(),
            name: name.clone(),
            source,
            doer: "symlink".to_string(),
        }
    }

    #[test]
    fn test_symlink_creation() {
        let temp = TempDir::new().unwrap();
        let src = temp.child("src");
        let dst = temp.child("dst");
        src.write_str("data").unwrap();

        let symlink = make_symlink(
            Action::Ensure,
            &Utf8PathBuf::from_path_buf(dst.path().to_path_buf()).unwrap(),
            Some(Utf8PathBuf::from_path_buf(src.path().to_path_buf()).unwrap()),
        );

        let output = Output::new("test-symlink", &defopts());
        let result = symlink
            .apply_ensure(&ApplyContext::default(), &defopts(), &output)
            .unwrap();
        assert_eq!(result, ONE_RESOURCE_ONE_CHANGE);
        assert!(dst.path().is_symlink());
    }

    #[test]
    fn test_symlink_noop_creation() {
        let temp = TempDir::new().unwrap();
        let src = temp.child("src");
        let dst = temp.child("dst");
        src.write_str("noop").unwrap();

        let symlink = make_symlink(
            Action::Ensure,
            &Utf8PathBuf::from_path_buf(dst.path().to_path_buf()).unwrap(),
            Some(Utf8PathBuf::from_path_buf(src.path().to_path_buf()).unwrap()),
        );

        let output = Output::new("test-symlink", &defopts_noop());
        let result = symlink
            .apply_ensure(&ApplyContext::default(), &defopts_noop(), &output)
            .unwrap();
        assert_eq!(result, ONE_RESOURCE_NOOP);
        assert!(!dst.path().exists());
    }

    #[test]
    fn test_symlink_removal() {
        let temp = TempDir::new().unwrap();
        let src = temp.child("src");
        let dst = temp.child("dst");
        src.write_str("x").unwrap();
        unix::fs::symlink(src.path(), dst.path()).unwrap();

        let symlink = make_symlink(
            Action::Remove,
            &Utf8PathBuf::from_path_buf(dst.path().to_path_buf()).unwrap(),
            None,
        );

        let output = Output::new("test-symlink", &defopts());
        let result = symlink
            .apply_remove(&ApplyContext::default(), &defopts(), &output)
            .unwrap();
        assert_eq!(result, ONE_RESOURCE_ONE_CHANGE);
        assert!(!dst.path().exists());
    }

    #[test]
    fn test_symlink_remove_missing() {
        let temp = TempDir::new().unwrap();
        let ghost = temp.child("ghost");

        let symlink = make_symlink(
            Action::Remove,
            &Utf8PathBuf::from_path_buf(ghost.path().to_path_buf()).unwrap(),
            None,
        );

        let output = Output::new("test-symlink", &defopts());
        let result = symlink
            .apply_remove(&ApplyContext::default(), &defopts(), &output)
            .unwrap();
        assert_eq!(result, ONE_RESOURCE_NO_CHANGE);
    }
}
