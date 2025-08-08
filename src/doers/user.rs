use crate::common::types::Changes;
use crate::prelude::*;
use anyhow::Context;
use nix::unistd::{Group, User};
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
pub struct GurpUserEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub desired_state: UserState,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct UserState {
    pub uid: u32,
    #[serde(rename = "home-dir")]
    pub home_dir: Utf8PathBuf,
    pub shell: Utf8PathBuf,
    pub gecos: String,
    #[serde(rename = "primary-group")]
    pub primary_group: String,
    #[serde(rename = "password-hash")]
    pub password_hash: Option<String>,
    pub profiles: Option<Vec<String>>,
    pub other_groups: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GurpUserRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpUserEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if !user_exists(&self.name)? {
            tracing::info!("creating user: {}", self.name);
            return self.create(opts);
        }

        let mut run_cmd = false;
        let current = self.current_state()?;
        let desired = &self.desired_state;
        let changes = self.changes(&current, desired);

        if changes.is_empty() {
            tracing::debug!("no change: {}", self.name);
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

        if changes.contains(&"other-groups") {
            if let Some(groups) = desired.other_groups.as_ref() {
                tracing::info!(
                    "change other-groups: [{}] -> {}",
                    self.name,
                    groups.join(",")
                );
                cmd.arg("-G");
                cmd.arg(groups.join(","));
            } else {
                tracing::info!("clear other-groups: [{}]", self.name);
                cmd.arg("-G");
                cmd.arg("");
            }
            run_cmd = true;
        }

        if changes.contains(&"profiles") {
            if let Some(profiles) = desired.profiles.as_ref() {
                tracing::info!("change profiles: [{}] -> {}", self.name, profiles.join(","));
                cmd.arg("-P");
                cmd.arg(profiles.join(","));
            } else {
                tracing::info!("clear profiles: [{}]", self.name);
                cmd.arg("-P");
                cmd.arg("");
            }
            run_cmd = true;
        }

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
            self.update_shadow(&Utf8PathBuf::from(SHADOW_PATH), &self.name, desired_hash)?;
        }

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }

    fn create(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let mut cmd = cmd!(
            USERADD_BIN,
            "-c",
            &self.desired_state.gecos,
            "-g",
            &self.desired_state.primary_group,
            "-d",
            &self.desired_state.home_dir,
            "-s",
            &self.desired_state.shell,
            "-u",
            self.desired_state.uid.to_string(),
        );

        if let Some(other_groups) = &self.desired_state.other_groups {
            cmd.arg("-G");
            cmd.arg(other_groups.join(","));
        }

        if let Some(profiles) = &self.desired_state.profiles {
            cmd.arg("-P");
            cmd.arg(profiles.join(","));
        }

        cmd.arg(&self.name);

        return_if_noop!(opts);

        let result = cmd.output()?;

        if result.status.success() {
            if let Some(password_hash) = &self.desired_state.password_hash {
                self.update_shadow(&Utf8PathBuf::from(SHADOW_PATH), &self.name, password_hash)?;
            }
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(String::from_utf8_lossy(&result.stderr).into_owned())
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

        if current.other_groups != desired.other_groups {
            to_change.push("other-groups");
        }

        if current.profiles != desired.profiles {
            to_change.push("profiles");
        }

        if !to_change.is_empty() {
            tracing::debug!("to change for {}: {}", self.name, to_change.join(", "));
        }

        to_change
    }

    fn current_state(&self) -> anyhow::Result<UserState> {
        tracing::debug!("getting state: {}", &self.name);
        match User::from_name(&self.name)? {
            Some(user) => {
                let primary_group = Group::from_gid(user.gid)?.context(format!(
                    "Group id '{}' not found for user '{}'",
                    user.gid, self.name
                ))?;

                let password_hash = if self.desired_state.password_hash.is_some() {
                    Some(self.hash_for_user(&Utf8PathBuf::from(SHADOW_PATH))?)
                } else {
                    None
                };

                Ok(UserState {
                    uid: user.uid.into(),
                    home_dir: Utf8PathBuf::try_from(user.dir)?,
                    shell: Utf8PathBuf::try_from(user.shell)?,
                    gecos: user.gecos.to_string_lossy().to_string(),
                    primary_group: primary_group.name.clone(),
                    password_hash,
                    profiles: user_profiles(&user.name)?,
                    other_groups: user_groups(&user.name, &primary_group.name)?,
                })
            }
            None => bail!("Could not find user {}", self.name),
        }
    }

    fn hash_for_user(&self, shadow_path: &Utf8PathBuf) -> anyhow::Result<String> {
        let desired_hash = match &self.desired_state.password_hash {
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
        user: &str,
        new_hash: &str,
    ) -> anyhow::Result<()> {
        tracing::info!("change user password-hash: [{}]", self.name);

        let raw_file = fs::read_to_string(shadow_path)?;
        let line_prefix = format!("{user}:");

        let output: String = raw_file
            .lines()
            .map(|l| {
                if l.starts_with(&line_prefix) {
                    let mut chunks: Vec<_> = l.split(':').collect();
                    if chunks.len() < SHADOW_FIELDS {
                        bail!("invalid shadow entry for {}", self.name);
                    }
                    chunks[1] = new_hash;
                    Ok(chunks.join(":"))
                } else {
                    Ok(l.to_string())
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");

        Ok(fs::write(shadow_path, output)?)
    }
}

impl GurpUserRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if user_exists(&self.name)? {
            if PROTECTED_USERS.contains(&self.name.as_str()) {
                tracing::warn!("protected resource: {}", self.name);
                return Ok(ONE_RESOURCE_ONE_ERROR);
            }

            tracing::info!("removing user: {}", self.name);

            let mut cmd = cmd!(USERDEL_BIN, &self.name);
            return_if_noop!(opts);
            one_change_or_stderr!(cmd, format!("error deleting user {}", self.name))
        } else {
            tracing::debug!("not present: {}", self.name);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn user_groups(username: &str, primary_group: &str) -> anyhow::Result<Option<Vec<String>>> {
    let raw = cmd_output!(GROUPS_BIN, username)?;
    let groups: Vec<_> = raw
        .split_whitespace()
        .filter_map(|g| {
            if g == primary_group {
                None
            } else {
                Some(g.to_owned())
            }
        })
        .collect();

    if groups.is_empty() {
        Ok(None)
    } else {
        Ok(Some(groups))
    }
}

fn user_profiles(username: &str) -> anyhow::Result<Option<Vec<String>>> {
    let raw = cmd_output!(PROFILES_BIN, username)?;
    let profiles: Vec<_> = raw
        .lines()
        .filter_map(|l| {
            let profile = l.trim();

            if profile != format!("{username}:")
                && profile != "All"
                && profile != "Basic Solaris User"
            {
                Some(profile.to_owned())
            } else {
                None
            }
        })
        .collect();

    if profiles.is_empty() {
        Ok(None)
    } else {
        Ok(Some(profiles))
    }
}

fn user_exists(username: &str) -> anyhow::Result<bool> {
    Ok(User::from_name(username)?.is_some())
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
            other_groups: None,
            profiles: None,
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
            other_groups: None,
            profiles: None,
        }
    }

    #[test]
    fn test_changes_detects_differences() {
        let g = GurpUserEnsure {
            id: "1".into(),
            name: "testuser".into(),
            desired_state: modified_state(),
        };

        let changes = g.changes(&dummy_state(), &g.desired_state);
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
        let g = GurpUserEnsure {
            id: "1".into(),
            name: "testuser".into(),
            desired_state: state.clone(),
        };

        let changes = g.changes(&state, &g.desired_state);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_update_shadow_replaces_correctly() {
        let input = "testuser:oldhash:18000:0:99999:7:::\notheruser:oldhash:...";
        let path = Utf8PathBuf::from("/tmp/shadow-test");

        fs::write(&path, input).unwrap();

        let g = GurpUserEnsure {
            id: "1".into(),
            name: "testuser".into(),
            desired_state: UserState {
                password_hash: Some("NEWHASH".into()),
                ..dummy_state()
            },
        };

        g.update_shadow(&path, "testuser", "NEWHASH").unwrap();

        assert_eq!(
            "testuser:NEWHASH:18000:0:99999:7:::\notheruser:oldhash:...",
            fs::read_to_string(&path).unwrap()
        );
    }
}
