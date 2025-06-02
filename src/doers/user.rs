use crate::doers::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
};
use crate::doers::types::{Apply, ApplySummary, Changes, Ensure, Remove};
use crate::utils::helpers;
use crate::utils::janet_helpers::{JanetExt, JanetStructExt};
use crate::utils::types::Opts;
use crate::{debug, info, verbose};
use anyhow::Context;
use camino::Utf8PathBuf;
use colored::Colorize;
use janetrs::{Janet, JanetArray};
use nix::unistd::{Group, User};
use std::process::Command;
use std::sync::LazyLock;

// THINGS TO KNOW
// Removing a group from "other-groups" will not remove the user from that group. This is a
// limitation of usermod(1m). I may fix it, or I may not.
// We do not create the user's home dir. Deal with that yourself.

static NOT_ALLOWED_TO_REMOVE: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "root", "daemon", "bin", "sys", "adm", "lp", "uucp", "nuucp", "dladm", "netadm", "netcfg",
        "listen", "gdm", "unknown", "nobody", "noaccess", "nobody4", "pkg5srv",
    ]
});

#[derive(Debug, PartialEq)]
pub struct UserToEnsure {
    pub id: String,
    pub name: String,
    pub uid: u32,
    pub home_dir: Utf8PathBuf,
    pub shell: Utf8PathBuf,
    pub gcos: String,
    pub primary_group: String,
    pub other_groups: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct UserToRemove {
    pub id: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct UserEnsureState {
    pub name: String,
    pub uid: u32,
    pub home_dir: Utf8PathBuf,
    pub shell: Utf8PathBuf,
    pub gcos: String,
    pub primary_group: String,
    pub other_groups: Vec<String>,
}

impl TryFrom<&Janet> for UserToEnsure {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<UserToEnsure> {
        let data = value.extract_struct()?;

        Ok(UserToEnsure {
            id: data.get_field_string("_id")?,
            name: data.get_field_string("name")?,
            uid: data.get_field_u32("uid")?,
            home_dir: data.get_field_pathbuf("home-dir")?,
            shell: data.get_field_pathbuf("shell")?,
            gcos: data.get_field_string("gcos")?,
            primary_group: data.get_field_string("group")?,
            other_groups: data.get_field_string_tuple("other-groups")?,
        })
    }
}

impl TryFrom<&Janet> for UserToRemove {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<UserToRemove> {
        let data = value.extract_struct()?;

        Ok(UserToRemove {
            id: data.get_field_string("_id")?,
            name: data.get_field_string("name")?,
        })
    }
}

pub fn unpack_ensure_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Ensure>> {
    resource_list
        .iter()
        .map(|r| {
            let dir = UserToEnsure::try_from(r)?;
            Ok(Ensure::User(dir))
        })
        .collect()
}

pub fn unpack_remove_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Remove>> {
    resource_list
        .iter()
        .map(|r| {
            let dir = UserToRemove::try_from(r)?;
            Ok(Remove::User(dir))
        })
        .collect()
}

fn diff_states<'a>(current: &UserEnsureState, desired: &UserEnsureState) -> Changes<'a> {
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

    if current.gcos != desired.gcos {
        to_change.push("gcos");
    }

    if current.primary_group != desired.primary_group {
        to_change.push("group");
    }

    if current.other_groups != desired.other_groups {
        to_change.push("other-groups");
    }

    to_change
}

impl UserToEnsure {
    fn state(&self) -> anyhow::Result<Option<UserEnsureState>> {
        user_state(&self.name, &self.primary_group, &self.other_groups)
    }

    fn desired_state(&self) -> anyhow::Result<UserEnsureState> {
        Ok(UserEnsureState {
            name: self.name.to_owned(),
            uid: self.uid,
            home_dir: self.home_dir.to_owned(),
            shell: self.shell.to_owned(),
            gcos: self.gcos.to_owned(),
            primary_group: self.primary_group.to_owned(),
            other_groups: self.other_groups.clone(),
        })
    }
}

impl Apply for UserToEnsure {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let current_state = match user_state(&self.name, &self.primary_group, &self.other_groups)? {
            Some(state) => state,
            None => {
                info!(opts, "Creating user {} [{}]", self.name, self.id);
                return create(&self.desired_state()?, opts);
            }
        };

        let desired_state = self.desired_state()?;

        let changes = diff_states(&current_state, &desired_state);

        if changes.is_empty() {
            verbose!(
                opts,
                "user: {} [{}] : no change required",
                self.name,
                self.id
            );
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        let mut cmd = Command::new("/usr/sbin/usermod");

        if changes.contains(&"gcos") {
            cmd.arg("-c").arg(&desired_state.gcos);
        }

        if changes.contains(&"home-dir") {
            cmd.arg("-d").arg(&desired_state.home_dir);
        }

        if changes.contains(&"primary-group") {
            cmd.arg("-g").arg(&desired_state.primary_group);
        }

        if changes.contains(&"other-groups") {
            cmd.arg("-G").arg(desired_state.other_groups.join(","));
        }

        if changes.contains(&"shell") {
            cmd.arg("-s").arg(desired_state.shell);
        }

        if changes.contains(&"uid") {
            cmd.arg("-u").arg(desired_state.uid.to_string());
        }

        cmd.arg(desired_state.name);

        debug!(opts, "{}", helpers::command_to_string(&cmd));

        let result = cmd.status()?;

        if result.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_ONE_ERROR)
        }
    }
}

fn create(state: &UserEnsureState, opts: &Opts) -> anyhow::Result<ApplySummary> {
    let mut cmd = Command::new("/usr/sbin/useradd");

    cmd.arg("-c")
        .arg(&state.gcos)
        .arg("-g")
        .arg(&state.primary_group)
        .arg("-G")
        .arg(state.other_groups.join(","))
        .arg("-s")
        .arg(&state.shell)
        .arg("-u")
        .arg(state.uid.to_string())
        .arg(&state.name);

    debug!(opts, "{}", helpers::command_to_string(&cmd));

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

fn user_state(
    user_name: &str,
    group_name: &str,
    other_groups: &Vec<String>,
) -> anyhow::Result<Option<UserEnsureState>> {
    match User::from_name(user_name)? {
        Some(user) => {
            let primary_gid = Group::from_name(group_name)?
                .context(format!("Group '{}' not found", group_name))?
                .name;

            let ret = UserEnsureState {
                name: user.name,
                uid: user.uid.into(),
                home_dir: Utf8PathBuf::try_from(user.dir)?,
                shell: Utf8PathBuf::try_from(user.shell)?,
                gcos: user.gecos.to_string_lossy().to_string(),
                primary_group: primary_gid.to_string(),
                other_groups: other_groups.clone(),
            };
            Ok(Some(ret))
        }
        None => Ok(None),
    }
}

impl Apply for UserToRemove {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        match User::from_name(&self.name)? {
            Some(_) => {
                if NOT_ALLOWED_TO_REMOVE.contains(&self.name.as_str()) {
                    eprintln!("Not allowed to remove {}", self.name);
                    return Ok(ONE_RESOURCE_ONE_ERROR);
                }

                let mut cmd = Command::new("/usr/sbin/userdel");
                cmd.arg(&self.name);

                info!(opts, "Removing user {} [{}]", self.name, self.id);
                debug!(opts, "{}", helpers::command_to_string(&cmd));

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
            }
            None => {
                debug!(opts, "user {} [{}] does not exist", self.name, self.id,);
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        }
    }
}
