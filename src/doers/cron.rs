use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE,
};
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplyContext, ApplySummary, Opts, Resource};
use crate::utils::helpers;
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use anyhow::bail;
use janetrs::{Janet, JanetArray};
use paste::paste;
use std::io::Write;
use std::process::{Command, Stdio};

const TAG_LINE: &str = "# gurp managed ID";
const CRONTAB_BIN: &str = "/bin/crontab";

// THINGS TO KNOW / THINGS TO DO.
// We use crontab(1) to apply changes. That checks values are valid, so we won't bother.

#[derive(Debug, PartialEq, Eq)]
pub struct GurpCron {
    pub action: Action,
    pub id: String,
    pub name: String,
    pub user: String,
    pub desired_state: Option<CronState>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CronState {
    pub minute: String,
    pub hour: String,
    pub day_of_month: String,
    pub month_of_year: String,
    pub day_of_week: String,
    pub command: String,
}

crate::unpack_fn!(ensure_list, Cron, GurpCron, box);
crate::unpack_fn!(remove_list, Cron, GurpCron, box);
crate::impl_apply!(GurpCron);

impl TryFrom<&Janet> for GurpCron {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;
        let action = janet_helpers::action_as_enum(&data)?;
        let state = match action {
            Action::Ensure => Some(CronState {
                command: data.get_field_string("command")?,
                minute: data.get_field_string("minute")?,
                hour: data.get_field_string("hour")?,
                day_of_month: data.get_field_string("day-of-month")?,
                month_of_year: data.get_field_string("month-of-year")?,
                day_of_week: data.get_field_string("day-of-week")?,
            }),
            Action::Remove => None,
        };

        Ok(GurpCron {
            action,
            user: data.get_field_string("user")?,
            id: data.get_field_string("_id")?,
            name: data.get_field_string("name")?,
            desired_state: state,
        })
    }
}

impl GurpCron {
    fn apply_ensure(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        let content = self.current_crontab()?;
        match self.ensured_crontab(&content)? {
            Some(new_crontab) => {
                tracing::info!("changing: {}", self.name);
                tracing::debug!("new crontab follows\n{}", new_crontab);
                if opts.noop {
                    Ok(ONE_RESOURCE_NOOP)
                } else {
                    self.write_crontab(&new_crontab)
                }
            }
            None => {
                tracing::info!("no change: {}", &self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        }
    }

    fn apply_remove(
        &self,
        _apply_context: &ApplyContext,
        opts: &Opts,
    ) -> anyhow::Result<ApplySummary> {
        let content = self.current_crontab()?;
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
                        self.write_crontab(&new_crontab)
                    }
                }
            }
            None => {
                tracing::info!("no change: {}", &self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        }
    }

    fn ensured_crontab(&self, content: &str) -> anyhow::Result<Option<String>> {
        let identifier = format!("{} {}", TAG_LINE, &self.id);
        let s = self.desired_state.as_ref().unwrap();
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

        Ok(Some(
            new_crontab.iter().map(|l| format!("{}\n", l)).collect(),
        ))
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
            Ok(Some(
                new_crontab.iter().map(|l| format!("{}\n", l)).collect(),
            ))
        } else {
            Ok(None)
        }
    }

    fn current_crontab(&self) -> anyhow::Result<String> {
        let mut cmd = Command::new(CRONTAB_BIN);
        cmd.arg("-u").arg(&self.user).arg("-l");
        tracing::debug!(command = helpers::command_to_string(&cmd));
        let result = cmd.output()?;
        Ok(String::from_utf8(result.stdout)?)
    }

    fn write_crontab(&self, content: &str) -> anyhow::Result<ApplySummary> {
        let mut cmd = Command::new(CRONTAB_BIN)
            .arg("-u")
            .arg(&self.user)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        tracing::debug!(command = "{} -u {}", CRONTAB_BIN, self.user);

        if let Some(stdin) = cmd.stdin.as_mut() {
            tracing::debug!("{}: writing: {}", &self.name, content);
            stdin.write_all(content.as_bytes())?;
        }

        let output = cmd.wait_with_output()?;

        if output.status.success() {
            tracing::debug!("{}: crontab updated successfully", self.name);
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
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

    fn common_ensure() -> GurpCron {
        GurpCron {
            action: Action::Ensure,
            id: "/test-role/cron/test".to_owned(),
            name: "Test job".to_owned(),
            user: "rob".to_owned(),
            desired_state: Some(CronState {
                minute: "4".to_owned(),
                hour: "1,12".to_owned(),
                day_of_month: "*".to_owned(),
                month_of_year: "*".to_owned(),
                day_of_week: "1-5".to_owned(),
                command: "/bin/command >/var/log/file".to_owned(),
            }),
        }
    }

    fn common_remove() -> GurpCron {
        GurpCron {
            action: Action::Remove,
            id: "/test-role/cron/test".to_owned(),
            name: "Test job".to_owned(),
            user: "rob".to_owned(),
            desired_state: None,
        }
    }
}
