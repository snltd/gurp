use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::types::{ApplySummary, Opts};
use crate::utils::helpers;
use anyhow::bail;
use serde::Deserialize;
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};

const TAG_LINE: &str = "# gurp managed ID";
const CRONTAB_BIN: &str = "/bin/crontab";

// THINGS TO KNOW / THINGS TO DO.
// We use crontab(1) to apply changes. That checks values are valid, so we won't bother.

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StrOrNumber {
    Str(String),
    Number(u32),
}

impl fmt::Display for StrOrNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrOrNumber::Str(s) => write!(f, "{s}"),
            StrOrNumber::Number(n) => write!(f, "{n}"),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GurpCronEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub user: String,
    #[serde(flatten)]
    pub desired_state: CronState,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct CronState {
    pub minute: StrOrNumber,
    pub hour: StrOrNumber,
    pub day_of_month: StrOrNumber,
    pub month_of_year: StrOrNumber,
    pub day_of_week: StrOrNumber,
    pub command: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpCronRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub user: String,
}

impl GurpCronEnsure {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let content = current_crontab(&self.user)?;
        match self.ensured_crontab(&content)? {
            Some(new_crontab) => {
                tracing::info!("changing: {}", self.name);
                tracing::debug!("new crontab follows\n{}", new_crontab);
                if opts.noop {
                    Ok(ONE_RESOURCE_NOOP)
                } else {
                    write_crontab(&self.user, &new_crontab)
                }
            }
            None => {
                tracing::debug!("no change: {}", &self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        }
    }

    fn ensured_crontab(&self, content: &str) -> anyhow::Result<Option<String>> {
        let identifier = format!("{} {}", TAG_LINE, &self.id);
        let s = &self.desired_state;
        let required_line = format!(
            "{} {} {} {} {} {}",
            s.minute, s.hour, s.day_of_month, s.month_of_year, s.day_of_week, s.command
        );

        let mut seen_identifier = false;
        let mut new_crontab: Vec<String> = Vec::new();
        let mut insert_here = false;

        for l in content.lines() {
            if insert_here {
                seen_identifier = true;
                insert_here = false;

                if l == required_line {
                    return Ok(None);
                } else {
                    new_crontab.push(required_line.clone());
                    continue;
                }
            }

            if l == identifier {
                insert_here = true;
            }

            new_crontab.push(l.to_string());
        }

        if !seen_identifier {
            new_crontab.push(identifier);
            new_crontab.push(required_line.clone());
        }

        Ok(Some(new_crontab.iter().map(|l| format!("{l}\n")).collect()))
    }
}

impl GurpCronRemove {
    pub fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let content = current_crontab(&self.user)?;
        match self.removed_crontab(&content)? {
            // If you try to write an empty file, crontab(1) will reject it. If we take out the
            // managed resource and there's nothing left, we have to *remove* the crontab.
            Some(new_crontab) => {
                tracing::info!("removing: {}", self.name);
                if new_crontab.is_empty() {
                    tracing::debug!("new {} crontab is empty", self.user);
                    if opts.noop {
                        Ok(ONE_RESOURCE_NOOP)
                    } else {
                        tracing::debug!("removing crontab: {}", self.user);
                        self.empty_crontab()
                    }
                } else {
                    tracing::debug!("new {} crontab follows\n{}", self.user, new_crontab);
                    if opts.noop {
                        Ok(ONE_RESOURCE_NOOP)
                    } else {
                        write_crontab(&self.user, &new_crontab)
                    }
                }
            }
            None => {
                tracing::debug!("no change: {}", &self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        }
    }

    fn removed_crontab(&self, content: &str) -> anyhow::Result<Option<String>> {
        let identifier = format!("{} {}", TAG_LINE, &self.id);
        let mut changed = false;
        let mut new_crontab: Vec<String> = Vec::new();
        let mut remove_here = false;

        for l in content.lines() {
            if remove_here {
                remove_here = false;
                changed = true;
                continue;
            }

            if l == identifier {
                remove_here = true;
                continue;
            }

            new_crontab.push(l.to_string());
        }

        if changed {
            Ok(Some(new_crontab.iter().map(|l| format!("{l}\n")).collect()))
        } else {
            Ok(None)
        }
    }

    fn empty_crontab(&self) -> anyhow::Result<ApplySummary> {
        let mut cmd = Command::new(CRONTAB_BIN);
        cmd.arg("-u").arg(&self.user).arg("-r");
        tracing::debug!(command = helpers::command_to_string(&cmd));
        let result = cmd.status()?;

        if result.success() {
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!("Failed to empty {} crontab", self.user)
        }
    }
}

fn current_crontab(username: &str) -> anyhow::Result<String> {
    let mut cmd = Command::new(CRONTAB_BIN);
    cmd.arg("-u").arg(username).arg("-l");
    tracing::debug!(command = helpers::command_to_string(&cmd));
    let result = cmd.output()?;
    Ok(String::from_utf8(result.stdout)?)
}

fn write_crontab(username: &str, content: &str) -> anyhow::Result<ApplySummary> {
    let mut cmd = Command::new(CRONTAB_BIN);
    cmd.arg("-u")
        .arg(username)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped());

    tracing::debug!(command = helpers::command_to_string(&cmd));

    let mut child = cmd.spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        tracing::debug!("{}: writing: {}", username, content);
        stdin.write_all(content.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        tracing::debug!("{}: crontab updated successfully", username);
        Ok(ONE_RESOURCE_ONE_CHANGE)
    } else {
        bail!(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_ensured_crontab_change() {
        let old_crontab = indoc! {"
            1 2 3 4 5 do_thing
            # gurp managed ID /test-role/cron/test
            5 4 3 2 1 wrong_command
            2 2 3 4 5 do_other_thing
            "};

        let expected_crontab = indoc! {"
            1 2 3 4 5 do_thing
            # gurp managed ID /test-role/cron/test
            4 1,12 * * 1-5 /bin/command >/var/log/file
            2 2 3 4 5 do_other_thing
            "}
        .to_owned();

        assert_eq!(
            Some(expected_crontab),
            common_ensure().ensured_crontab(old_crontab).unwrap()
        );
    }

    #[test]
    fn test_ensured_crontab_already_there() {
        let old_crontab = indoc! {"
            1 2 3 4 5 do_thing
            # gurp managed ID /test-role/cron/test
            4 1,12 * * 1-5 /bin/command >/var/log/file
            2 2 3 4 5 do_other_thing
            "};

        assert_eq!(None, common_ensure().ensured_crontab(old_crontab).unwrap());
    }

    #[test]
    fn test_ensured_crontab_add() {
        let old_crontab = indoc! {"
             1 2 3 4 5 do_thing
             2 2 3 4 5 do_other_thing
             "};

        let expected_crontab = indoc! {"
            1 2 3 4 5 do_thing
            2 2 3 4 5 do_other_thing
            # gurp managed ID /test-role/cron/test
            4 1,12 * * 1-5 /bin/command >/var/log/file
            "
        }
        .to_owned();

        assert_eq!(
            Some(expected_crontab),
            common_ensure().ensured_crontab(old_crontab).unwrap()
        );
    }

    #[test]
    fn test_removed_crontab_change() {
        let old_crontab = indoc! {"
            1 2 3 4 5 do_thing
            # gurp managed ID /test-role/cron/test
            4 1,12 * * 1-5 /bin/command >/var/log/file
            2 2 3 4 5 do_other_thing
            "
        };

        let expected_crontab = indoc! {"
            1 2 3 4 5 do_thing
            2 2 3 4 5 do_other_thing
            "
        }
        .to_owned();

        assert_eq!(
            Some(expected_crontab),
            common_remove().removed_crontab(old_crontab).unwrap()
        );
    }

    #[test]
    fn test_removed_crontab_not_there() {
        let old_crontab = indoc! {"
            1 2 3 4 5 do_thing
            2 2 3 4 5 do_other_thing
        "};

        assert_eq!(None, common_remove().removed_crontab(old_crontab).unwrap());
    }

    fn common_ensure() -> GurpCronEnsure {
        GurpCronEnsure {
            id: "/test-role/cron/test".to_owned(),
            name: "Test job".to_owned(),
            user: "rob".to_owned(),
            desired_state: CronState {
                minute: StrOrNumber::Number(4),
                hour: StrOrNumber::Str("1,12".to_owned()),
                day_of_month: StrOrNumber::Str("*".to_owned()),
                month_of_year: StrOrNumber::Str("*".to_owned()),
                day_of_week: StrOrNumber::Str("1-5".to_owned()),
                command: "/bin/command >/var/log/file".to_owned(),
            },
        }
    }

    fn common_remove() -> GurpCronRemove {
        GurpCronRemove {
            id: "/test-role/cron/test".to_owned(),
            name: "Test job".to_owned(),
            user: "rob".to_owned(),
        }
    }
}
