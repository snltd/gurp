use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
    PROTECTED_USERS,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplySummary, Changes, Opts, Resource};
use crate::debug;
use crate::utils::helpers;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::{Context, anyhow};
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray};
use nix::unistd::{Group, User};
use paste::paste;
use std::process::Command;

// THINGS TO KNOW
// Removing a group from "other-groups" will not remove the user from that group. This is a
// limitation of usermod(1m). I may fix it, or I may not.
// We do not create the user's home dir. Deal with that yourself.
// We can create non-primary groups for a new user, but not change them for an existing one.

pub struct GurpUser {
    pub action: Action,
    pub exists: bool,
    pub id: String,
    pub name: String,
    pub desired_state: Option<UserState>,
    pub doer: String,
}

pub struct UserState {
    pub uid: u32,
    pub home_dir: Utf8PathBuf,
    pub shell: Utf8PathBuf,
    pub gecos: String,
    pub primary_group: String,
    pub other_groups: Vec<String>,
}

impl TryFrom<&Janet> for GurpUser {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let action = janet_helpers::action_as_enum(&data)?;
        let name = data.get_field_string("name")?;
        let exists = User::from_name(&name).ok().is_some();

        let state = match action {
            Action::Ensure => Some(UserState {
                uid: data.get_field_u32("uid")?,
                home_dir: data.get_field_pathbuf("home-dir")?,
                shell: data.get_field_pathbuf("shell")?,
                gecos: data.get_field_string("gecos")?,
                primary_group: data.get_field_string("group")?,
                other_groups: data.get_field_string_tuple("other-groups")?,
            }),
            Action::Remove => None,
        };

        Ok(GurpUser {
            name,
            id: data.get_field_string("_id")?,
            action,
            exists,
            desired_state: state,
            doer: "user".to_owned(),
        })
    }
}

crate::unpack_fn!(ensure_list, User, GurpUser);
crate::unpack_fn!(remove_list, User, GurpUser);
crate::impl_apply!(GurpUser);

impl GurpUser {
    fn apply_ensure(&self, opts: &Opts, output: &Output) -> anyhow::Result<ApplySummary> {
        if !self.exists {
            output.creating(&self.name);
            return self.create(opts);
        }

        let current = self.current_state()?;
        let desired = self.desired_state.as_ref().unwrap();
        let changes = self.changes(&current, desired);

        if changes.is_empty() {
            output.no_change(&self.name);
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        let mut cmd = Command::new("/usr/sbin/usermod");

        if changes.contains(&"gecos") {
            cmd.arg("-c").arg(&desired.gecos);
        }

        if changes.contains(&"home-dir") {
            cmd.arg("-d").arg(&desired.home_dir);
        }

        if changes.contains(&"primary-group") {
            cmd.arg("-g").arg(&desired.primary_group);
        }

        if changes.contains(&"other-groups") {
            cmd.arg("-G").arg(desired.other_groups.join(","));
        } // Doesn't do anything now

        if changes.contains(&"shell") {
            cmd.arg("-s").arg(&desired.shell);
        }

        if changes.contains(&"uid") {
            cmd.arg("-u").arg(desired.uid.to_string());
        }

        cmd.arg(&self.name);

        debug!(opts, "doer/user", "{}", helpers::command_to_string(&cmd));

        let result = cmd.status()?;

        if result.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_ONE_ERROR)
        }
    }

    fn apply_remove(&self, opts: &Opts, output: &Output) -> anyhow::Result<ApplySummary> {
        if self.exists {
            if PROTECTED_USERS.contains(&self.name.as_str()) {
                output.protected(&self.name);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

            let mut cmd = Command::new("/usr/sbin/userdel");
            cmd.arg(&self.name);

            output.removing(&self.name);
            debug!(opts, "doer/user", "{}", helpers::command_to_string(&cmd));

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                let result = cmd.status()?;

                if result.success() {
                    Ok(ONE_RESOURCE_ONE_CHANGE)
                } else {
                    Ok(ONE_RESOURCE_ONE_ERROR)
                }
            }
        } else {
            output.not_present(&self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn changes<'a>(&self, current: &UserState, desired: &UserState) -> Changes<'a> {
        let mut to_change = Vec::new();

        if current.uid != desired.uid {
            to_change.push("uid");
        }

        if current.home_dir != desired.home_dir {
            to_change.push("home-dir");
        }

        if current.shell != desired.shell {
            to_change.push("shell");
        }

        if current.gecos != desired.gecos {
            to_change.push("gecos");
        }

        if current.primary_group != desired.primary_group {
            to_change.push("group");
        }

        if current.other_groups != desired.other_groups {
            to_change.push("other-groups");
        } // doesn't do anything now

        to_change
    }

    fn create(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let state = self.desired_state.as_ref().unwrap();
        let mut cmd = Command::new("/usr/sbin/useradd");

        cmd.arg("-c")
            .arg(&state.gecos)
            .arg("-g")
            .arg(&state.primary_group)
            .arg("-G")
            .arg(state.other_groups.join(","))
            .arg("-s")
            .arg(&state.shell)
            .arg("-u")
            .arg(state.uid.to_string())
            .arg(&self.name);

        debug!(opts, "doer/user", "{}", helpers::command_to_string(&cmd));

        if opts.noop {
            return Ok(ONE_RESOURCE_ONE_CHANGE);
        }

        let result = cmd.status()?;

        if result.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_ONE_ERROR)
        }
    }

    fn current_state(&self) -> anyhow::Result<UserState> {
        match User::from_name(&self.name)? {
            Some(user) => {
                let primary_group = Group::from_gid(user.gid)?.context(format!(
                    "Group d '{}' not found for user '{}'",
                    user.gid, self.name
                ))?;

                Ok(UserState {
                    uid: user.uid.into(),
                    home_dir: Utf8PathBuf::try_from(user.dir)?,
                    shell: Utf8PathBuf::try_from(user.shell)?,
                    gecos: user.gecos.to_string_lossy().to_string(),
                    primary_group: primary_group.name,
                    other_groups: Vec::new(), // we don't do anything with this field
                })
            }
            None => Err(anyhow!("Could not find user {}", self.name)),
        }
    }
}
