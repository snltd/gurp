use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
    PROTECTED_USERS,
};
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Changes, Opts, Resource};
use crate::utils::helpers;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray};
use nix::unistd::{Group, User};
use paste::paste;
use std::fs;
use std::process::Command;

// THINGS TO KNOW
// Removing a group from "other-groups" will not remove the user from that group. This is a
// limitation of usermod(1m). I may fix it, or I may not.
// We do not create the user's home dir. Deal with that yourself.
// We can create non-primary groups for a new user, but not change them for an existing one.
//
pub const SHADOW_FIELDS: usize = 9;
pub const SHADOW_PATH: &str = "/etc/shadow";

const USERADD_BIN: &str = "/usr/sbin/useradd";
const USERDEL_BIN: &str = "/usr/sbin/userdel";
const USERMOD_BIN: &str = "/usr/sbin/usermod";

pub struct GurpUser {
    pub action: Action,
    pub exists: bool,
    pub id: String,
    pub name: String,
    pub desired_state: Option<UserState>,
}

#[derive(Clone)]
pub struct UserState {
    pub uid: u32,
    pub home_dir: Utf8PathBuf,
    pub shell: Utf8PathBuf,
    pub gecos: String,
    pub primary_group: String,
    pub password_hash: Option<String>,
    // pub other_groups: Vec<String>,
}

impl TryFrom<&Janet> for GurpUser {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let action = janet_helpers::action_as_enum(&data)?;
        let name = data.get_field_string("name")?;
        let exists = User::from_name(&name)?.is_some();

        let state = match action {
            Action::Ensure => Some(UserState {
                uid: data.get_field_u32("uid")?,
                home_dir: data.get_field_pathbuf("home-dir")?,
                shell: data.get_field_pathbuf("shell")?,
                gecos: data.get_field_string("gecos")?,
                primary_group: data.get_field_string("group")?,
                password_hash: data.get_field_string_opt("password-hash"),
                // other_groups: data.get_field_string_tuple("other-groups")?,
            }),
            Action::Remove => None,
        };

        Ok(GurpUser {
            name,
            id: data.get_field_string("_id")?,
            action,
            exists,
            desired_state: state,
        })
    }
}

crate::unpack_fn!(ensure_list, User, GurpUser);
crate::unpack_fn!(remove_list, User, GurpUser);
crate::impl_apply!(GurpUser);

impl GurpUser {
    fn apply_ensure(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        if !self.exists {
            tracing::info!("creating user: {}", self.name);
            return self.create(opts);
        }

        let mut run_cmd = false;
        let current = self.current_state()?;
        let desired = self.desired_state.as_ref().unwrap();
        let changes = self.changes(&current, desired);

        if changes.is_empty() {
            tracing::info!("no change: {}", self.name);
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        let mut cmd = Command::new(USERMOD_BIN);

        if changes.contains(&"gecos") {
            tracing::info!(
                "change user gecos: [{}] {} -> {}",
                self.name,
                current.gecos,
                desired.gecos
            );
            cmd.arg("-c").arg(&desired.gecos);
            run_cmd = true;
        }

        if changes.contains(&"home-dir") {
            tracing::info!(
                "change user home-dir: [{}] {} -> {}",
                self.name,
                current.home_dir,
                desired.home_dir
            );
            cmd.arg("-d").arg(&desired.home_dir);
            run_cmd = true;
        }

        if changes.contains(&"primary-group") {
            tracing::info!(
                "change user primary-group: [{}] {} -> {}",
                self.name,
                current.primary_group,
                desired.primary_group
            );
            cmd.arg("-g").arg(&desired.primary_group);
            run_cmd = true;
        }

        // if changes.contains(&"other-groups") {
        // cmd.arg("-G").arg(desired.other_groups.join(","));
        // } // Doesn't do anything now

        if changes.contains(&"shell") {
            tracing::info!(
                "change user shell: [{}] {} -> {}",
                self.name,
                current.shell,
                desired.shell
            );
            cmd.arg("-s").arg(&desired.shell);
            run_cmd = true;
        }

        if changes.contains(&"uid") {
            tracing::info!(
                "change user uid: [{}] {} -> {}",
                self.name,
                current.uid,
                desired.uid
            );
            cmd.arg("-u").arg(desired.uid.to_string());
            run_cmd = true;
        }

        if opts.noop {
            return Ok(ONE_RESOURCE_NOOP);
        }

        cmd.arg(&self.name);

        if run_cmd {
            tracing::debug!(command = helpers::command_to_string(&cmd));

            let result = cmd.output()?;

            if !result.status.success() {
                bail!(String::from_utf8_lossy(&result.stderr).into_owned())
            }
        }

        if changes.contains(&"password-hash") {
            let desired_hash = desired.password_hash.as_ref().unwrap();
            let old_hash = current.password_hash.unwrap();

            tracing::info!(
                "change user password-hash: [{}] {} -> {}",
                self.name,
                old_hash,
                desired_hash
            );

            self.update_shadow(&Utf8PathBuf::from(SHADOW_PATH), &old_hash, desired_hash)?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn apply_remove(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        if self.exists {
            if PROTECTED_USERS.contains(&self.name.as_str()) {
                tracing::warn!("protected resource: {}", self.name);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

            let mut cmd = Command::new(USERDEL_BIN);
            cmd.arg(&self.name);

            tracing::info!("removing user: {}", self.name);
            tracing::debug!(command = helpers::command_to_string(&cmd));

            if opts.noop {
                Ok(ONE_RESOURCE_NOOP)
            } else {
                let result = cmd.output()?;

                if result.status.success() {
                    Ok(ONE_RESOURCE_ONE_CHANGE)
                } else {
                    bail!(String::from_utf8_lossy(&result.stderr).into_owned())
                }
            }
        } else {
            tracing::debug!("not present: {}", self.name);
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
            to_change.push("primary-group");
        }

        if current.password_hash.is_some() && current.password_hash != desired.password_hash {
            to_change.push("password-hash");
        }

        // if current.other_groups != desired.other_groups {
        // to_change.push("other-groups");
        // } // doesn't do anything now

        tracing::debug!("to change for {}: {}", self.name, to_change.join(", "));
        to_change
    }

    fn create(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let state = self.desired_state.as_ref().unwrap();
        let mut cmd = Command::new(USERADD_BIN);

        cmd.arg("-c")
            .arg(&state.gecos)
            .arg("-g")
            .arg(&state.primary_group)
            // .arg("-G")
            // .arg(state.other_groups.join(","))
            .arg("-s")
            .arg(&state.shell)
            .arg("-u")
            .arg(state.uid.to_string())
            .arg(&self.name);

        tracing::debug!(command = helpers::command_to_string(&cmd));

        if opts.noop {
            return Ok(ONE_RESOURCE_ONE_CHANGE);
        }

        let result = cmd.output()?;

        if result.status.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(String::from_utf8_lossy(&result.stderr).into_owned())
        }
    }

    fn current_state(&self) -> anyhow::Result<UserState> {
        tracing::debug!("getting state: {}", &self.name);
        match User::from_name(&self.name)? {
            Some(user) => {
                let primary_group = Group::from_gid(user.gid)?.context(format!(
                    "Group d '{}' not found for user '{}'",
                    user.gid, self.name
                ))?;

                let password_hash = if self.desired_state.as_ref().unwrap().password_hash.is_some()
                {
                    Some(self.hash_for_user(&Utf8PathBuf::from(SHADOW_PATH))?)
                } else {
                    None
                };

                Ok(UserState {
                    uid: user.uid.into(),
                    home_dir: Utf8PathBuf::try_from(user.dir)?,
                    shell: Utf8PathBuf::try_from(user.shell)?,
                    gecos: user.gecos.to_string_lossy().to_string(),
                    primary_group: primary_group.name,
                    password_hash,
                    // other_groups: Vec::new(), // we don't do anything with this field
                })
            }
            None => bail!("Could not find user {}", self.name),
        }
    }

    fn hash_for_user(&self, shadow_path: &Utf8PathBuf) -> anyhow::Result<String> {
        let desired_hash = match &self.desired_state.as_ref().unwrap().password_hash {
            Some(hash) => hash,
            None => bail!("no hash supplied for {}", self.name),
        };

        let raw_file = fs::read_to_string(shadow_path)?;
        let correct_leader = format!("{}:{}:", self.name, desired_hash);
        let leader = format!("{}:", self.name);

        for line in raw_file.lines() {
            if line.starts_with(&correct_leader) {
                return Ok(desired_hash.to_owned());
            } else if line.starts_with(&leader) {
                let chunks: Vec<_> = line.split(':').collect();
                if chunks.len() < SHADOW_FIELDS {
                    bail!("invalid shadow entry for {}", self.name);
                }
                return Ok(chunks[1].to_owned());
            }
        }

        bail!("Did not find {} in shadow file", self.name);
    }

    fn update_shadow(
        &self,
        shadow_path: &Utf8PathBuf,
        old_hash: &str,
        new_hash: &str,
    ) -> anyhow::Result<()> {
        let raw_file = fs::read_to_string(shadow_path)?;
        let output = raw_file.replace(old_hash, new_hash);
        Ok(fs::write(shadow_path, output)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    fn dummy_state() -> UserState {
        UserState {
            uid: 1000,
            home_dir: Utf8PathBuf::from("/home/test"),
            shell: Utf8PathBuf::from("/bin/bash"),
            gecos: "Test User".into(),
            primary_group: "users".into(),
            password_hash: Some("hash1".into()),
        }
    }

    fn modified_state() -> UserState {
        UserState {
            uid: 1001,
            home_dir: Utf8PathBuf::from("/home/tester"),
            shell: Utf8PathBuf::from("/bin/zsh"),
            gecos: "Tester User".into(),
            primary_group: "staff".into(),
            password_hash: Some("hash2".into()),
        }
    }

    #[test]
    fn test_changes_detects_differences() {
        let g = GurpUser {
            action: Action::Ensure,
            exists: true,
            id: "1".into(),
            name: "testuser".into(),
            desired_state: Some(modified_state()),
        };

        let changes = g.changes(&dummy_state(), g.desired_state.as_ref().unwrap());
        assert_eq!(
            changes,
            vec![
                "uid",
                "home-dir",
                "shell",
                "gecos",
                "primary-group",
                "password-hash"
            ]
        );
    }

    #[test]
    fn test_changes_detects_no_difference() {
        let state = dummy_state();
        let g = GurpUser {
            action: Action::Ensure,
            exists: true,
            id: "1".into(),
            name: "testuser".into(),
            desired_state: Some(state.clone()),
        };

        let changes = g.changes(&state, g.desired_state.as_ref().unwrap());
        assert!(changes.is_empty());
    }

    #[test]
    fn test_update_shadow_replaces_correctly() {
        let input = "testuser:oldhash:18000:0:99999:7:::\notheruser:somehash:...";
        let path = Utf8PathBuf::from("/tmp/shadow-test");

        fs::write(&path, input).unwrap();

        let g = GurpUser {
            action: Action::Ensure,
            exists: true,
            id: "1".into(),
            name: "testuser".into(),
            desired_state: Some(UserState {
                password_hash: Some("newhash".into()),
                ..dummy_state()
            }),
        };

        g.update_shadow(&path, "oldhash", "newhash").unwrap();

        let output = fs::read_to_string(&path).unwrap();
        assert!(output.contains("testuser:newhash:"));
        assert!(!output.contains("oldhash"));
    }
}
