use anyhow::{bail, ensure, Context};
use camino::Utf8PathBuf;
use common::cmd;
use common::constants::{
    GROUPS_BIN, ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, PROFILES_BIN, PROTECTED_USERS,
    USERADD_BIN, USERDEL_BIN, USERMOD_BIN,
};
use common::types::{ApplyOpts, ApplySummary};
use nix::unistd::{Group, User};
use serde::Deserialize;
use std::fs;
use std::process::Command;

pub const SHADOW_FIELDS: usize = 9;
pub const SHADOW_PATH: &str = "/etc/shadow";

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpUserEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub desired_state: UserState,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
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
#[cfg_attr(test, derive(PartialEq))]
pub struct GurpUserRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl GurpUserEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if !user_exists(&self.name)? {
            tracing::info!("creating user: {}", self.name);
            return self.create(opts);
        }

        let current = self.current_state()?;
        let desired = &self.desired_state;
        let mut changes = 0;

        let mut cmd = Command::new(USERMOD_BIN);

        if current.gecos != desired.gecos {
            changes += 1;
            tracing::info!(
                "change user gecos: [{}] {} -> {}",
                self.name,
                current.gecos,
                desired.gecos
            );
            cmd.arg("-c").arg(&desired.gecos);
        }

        if current.home_dir != desired.home_dir {
            changes += 1;
            tracing::info!(
                "change user home-dir: [{}] {} -> {}",
                self.name,
                current.home_dir,
                desired.home_dir
            );
            cmd.arg("-d").arg(&desired.home_dir);
        }

        if current.primary_group != desired.primary_group {
            changes += 1;
            tracing::info!(
                "change user primary-group: [{}] {} -> {}",
                self.name,
                current.primary_group,
                desired.primary_group
            );
            cmd.arg("-g").arg(&desired.primary_group);
        }

        if current.other_groups != desired.other_groups {
            changes += 1;
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
        }

        if current.profiles != desired.profiles {
            changes += 1;
            if let Some(profiles) = desired.profiles.as_ref() {
                tracing::info!("change profiles: [{}] -> {}", self.name, profiles.join(","));
                cmd.arg("-P");
                cmd.arg(profiles.join(","));
            } else {
                tracing::info!("clear profiles: [{}]", self.name);
                cmd.arg("-P");
                cmd.arg("");
            }
        }

        if current.shell != desired.shell {
            changes += 1;
            tracing::info!(
                "change user shell: [{}] {} -> {}",
                self.name,
                current.shell,
                desired.shell
            );
            cmd.arg("-s").arg(&desired.shell);
        }

        if current.uid != desired.uid {
            changes += 1;
            tracing::info!(
                "change user uid: [{}] {} -> {}",
                self.name,
                current.uid,
                desired.uid
            );
            cmd.arg("-u").arg(desired.uid.to_string());
        }

        return_if_noop!(opts, 1, changes);

        cmd.arg(&self.name);

        if changes > 0 {
            tracing::debug!(command = cmd::to_string(&cmd));

            let result = cmd.output()?;

            ensure!(
                result.status.success(),
                String::from_utf8_lossy(&result.stderr).into_owned()
            );
        }

        if current.password_hash.is_some() && current.password_hash != desired.password_hash {
            changes += 1;
            let desired_hash = desired.password_hash.as_ref().unwrap();
            self.update_shadow(&Utf8PathBuf::from(SHADOW_PATH), &self.name, desired_hash)?;
        }

        Ok(ApplySummary {
            resources: 1,
            changes,
        })
    }

    fn create(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
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
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if user_exists(&self.name)? {
            ensure!(
                !PROTECTED_USERS.contains(&self.name.as_str()),
                format!("protected resource: {}", self.name)
            );

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
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;

    #[test]
    fn test_deserialize_user_ensure_gurpuser() {
        assert_eq!(
            GurpUserEnsure {
                id: "/NO-ROLE/user/gurpuser".to_owned(),
                name: "gurpuser".to_owned(),
                desired_state: UserState {
                    primary_group: "sysadmin".to_owned(),
                    uid: 1264,
                    home_dir: Utf8PathBuf::from("/home/gurpuser"),
                    shell: Utf8PathBuf::from("/bin/ksh"),
                    gecos: "Gurp Managed User".to_owned(),
                    password_hash: Some("w0934cm-4i5c-42u5cn492hrc97h234ui".to_owned()),
                    other_groups: None,
                    profiles: None,
                }
            },
            deserialized_example("user/ensure-user-gurpuser.janet")
        );
    }

    #[test]
    fn test_deserialize_user_remove_user_lolex() {
        assert_eq!(
            GurpUserRemove {
                id: "/NO-ROLE/user/lolex".to_owned(),
                name: "lolex".to_owned(),
            },
            deserialized_example("user/remove-user-lolex.janet")
        );
    }

    #[test]
    fn test_update_shadow_replaces_correctly() {
        let original_shadow = indoc::indoc! { "
            gurpuser:oldhash:264:14:99999:7:::
            otheruser:oldhash:..."};

        let expected_shadow = indoc::indoc! { "
            gurpuser:NEWHASH:264:14:99999:7:::
            otheruser:oldhash:..."};

        let path = Utf8PathBuf::from("/tmp/shadow-test");

        fs::write(&path, original_shadow).unwrap();

        let g = GurpUserEnsure {
            desired_state: UserState {
                password_hash: Some("NEWHASH".into()),
                ..deserialized_example("user/ensure-user-gurpuser.janet")
            },
            ..deserialized_example("user/ensure-user-gurpuser.janet")
        };

        g.update_shadow(&path, "gurpuser", "NEWHASH").unwrap();
        assert_eq!(expected_shadow, fs::read_to_string(&path).unwrap());
    }
}
