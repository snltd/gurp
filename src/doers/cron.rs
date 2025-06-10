use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplySummary, Opts, Resource};
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::anyhow;
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray};
use paste::paste;
use std::fs;
use std::io::Write;

// THINGS TO KNOW / THINGS TO DO.
#[derive(Debug, PartialEq, Eq)]
pub struct GurpCron {
    pub action: Action,
    pub exists: bool,
    pub id: String,
    pub name: String,
    pub desired_state: Option<CronState>,
    pub doer: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CronState {
    pub user: String,
    pub minute: String,
    pub hour: String,
    pub day_of_month: String,
    pub month_of_year: String,
    pub day_of_week: String,
}

crate::unpack_fn!(ensure_list, FileLine, GurpCron);
crate::unpack_fn!(remove_list, FileLine, GurpCron);
crate::impl_apply!(GurpCron);

impl TryFrom<&Janet> for GurpCron {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let name = data.get_field_pathbuf("name")?;

        let action = janet_helpers::action_as_enum(&data)?;
        let state = match action {
            Action::Ensure => Some(CronState {}),
            Action::Ensure => None,
        }

        Ok(GurpCron {
            action,
            exists,
            id: data.get_field_string("_id")?,
            name: data.get_field_string("name")?,
            desired_state: state,
            doer: "file-line".to_owned(),
        })
    }
}
