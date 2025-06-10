use crate::common::constants::{
    ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE, ONE_RESOURCE_ONE_ERROR,
};
use crate::common::output::Output;
use crate::common::traits::Apply;
use crate::common::types::{Action, ApplySummary, Opts, Resource};
use crate::utils::janet_helpers::{self, JanetExt, JanetStructExt};
use janetrs::{Janet, JanetArray};
use paste::paste;
use std::io::Write;
use std::process::{Command, Stdio};

const TAG_LINE: &str = "gurp managed id";

// THINGS TO KNOW / THINGS TO DO.
#[derive(Debug, PartialEq, Eq)]
pub struct GurpCron {
    pub action: Action,
    // pub exists: bool,
    pub id: String,
    pub name: String,
    pub user: String,
    pub desired_state: Option<CronState>,
    pub doer: String,
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

crate::unpack_fn!(ensure_list, Cron, GurpCron);
crate::unpack_fn!(remove_list, Cron, GurpCron);
crate::impl_apply!(GurpCron);

impl TryFrom<&Janet> for GurpCron {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<Self> {
        let data = value.extract_struct()?;

        // I'm not going to verify the numbers. MVP and all that.
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
            // exists,
            id: data.get_field_string("_id")?,
            name: data.get_field_string("name")?,
            desired_state: state,
            doer: "file-line".to_owned(),
        })
    }
}

impl GurpCron {
    fn apply_ensure(&self, opts: &Opts, output: &Output) -> anyhow::Result<ApplySummary> {
        let content = self.current_crontab()?;
        match self.ensured_crontab(&content)? {
            Some(crontab) => {
                println!("WRITE CRONTAB {}", crontab);
                if opts.noop {
                    Ok(ONE_RESOURCE_NOOP)
                } else if self.write_crontab(&content)? {
                    Ok(ONE_RESOURCE_ONE_CHANGE)
                } else {
                    Ok(ONE_RESOURCE_ONE_ERROR)
                }
            }
            None => {
                output.no_change(&self.name);
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
        let mut found_identifier = false;

        for l in content.lines() {
            if found_identifier {
                found_identifier = false;
                seen_identifier = true;

                if l == required_line {
                    return Ok(None);
                } else {
                    new_crontab.push(required_line.clone());
                    continue;
                }
            }

            if l == identifier {
                found_identifier = true;
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
        let mut seen_identifier = false;
        let mut new_crontab: Vec<String> = Vec::new();
        let mut found_identifier = false;

        for l in content.lines() {
            if found_identifier {
                found_identifier = false;
                seen_identifier = true;
                continue;
            }

            if l == identifier {
                found_identifier = true;
                continue;
            }

            new_crontab.push(l.to_string());
        }

        if seen_identifier {
            Ok(Some(
                new_crontab.iter().map(|l| format!("{}\n", l)).collect(),
            ))
        } else {
            Ok(None)
        }
    }

    fn apply_remove(&self, opts: &Opts, output: &Output) -> anyhow::Result<ApplySummary> {
        let content = self.current_crontab()?;
        match self.removed_crontab(&content)? {
            Some(crontab) => {
                if crontab.is_empty() {
                    if opts.noop {
                        Ok(ONE_RESOURCE_NOOP)
                    } else if self.empty_crontab()? {
                        Ok(ONE_RESOURCE_ONE_CHANGE)
                    } else {
                        Ok(ONE_RESOURCE_ONE_ERROR)
                    }
                } else {
                    println!("WRITE CRONTAB {}", crontab);
                    if opts.noop {
                        Ok(ONE_RESOURCE_NOOP)
                    } else if self.write_crontab(&content)? {
                        Ok(ONE_RESOURCE_ONE_CHANGE)
                    } else {
                        Ok(ONE_RESOURCE_ONE_ERROR)
                    }
                }
            }
            None => {
                output.no_change(&self.name);
                Ok(ONE_RESOURCE_NO_CHANGE)
            }
        }
    }

    fn current_crontab(&self) -> anyhow::Result<String> {
        let cmd = Command::new("/bin/crontab")
            .arg("-u")
            .arg(&self.user)
            .arg("-l")
            .output()?;

        Ok(String::from_utf8(cmd.stdout)?)
    }

    fn write_crontab(&self, content: &str) -> anyhow::Result<bool> {
        let mut child = Command::new("crontab").stdin(Stdio::piped()).spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(content.as_bytes())?;
        }

        let status = child.wait()?;
        Ok(status.success())
    }

    fn empty_crontab(&self) -> anyhow::Result<bool> {
        // Because writing an empty one does nothing.
        let mut cmd = Command::new("crontab");
        cmd.arg("-u").arg(&self.user).arg("-r");
        let result = cmd.status()?;
        Ok(result.success())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ensured_crontab_change() {
        let id = "/test-role/cron/test";
        let old_crontab = format!(
            "1 2 3 4 5 do_thing\n\
             {} {}\n\
             5 4 3 2 1 wrong_command\n\
             2 2 3 4 5 do_other_thing\n\
             ",
            TAG_LINE, id
        );

        let expected_crontab = format!(
            "1 2 3 4 5 do_thing\n\
             {} {}\n\
             4 1,12 * * 1-5 /bin/command >/var/log/file\n\
             2 2 3 4 5 do_other_thing\n\
             ",
            TAG_LINE, id
        );
        assert_eq!(
            Some(expected_crontab),
            common_ensure().ensured_crontab(&old_crontab).unwrap()
        );
    }

    #[test]
    fn test_ensured_crontab_already_there() {
        let id = "/test-role/cron/test";
        let old_crontab = format!(
            "1 2 3 4 5 do_thing\n\
             {} {}\n\
             4 1,12 * * 1-5 /bin/command >/var/log/file\n\
             2 2 3 4 5 do_other_thing\n\
             ",
            TAG_LINE, id
        );

        assert_eq!(None, common_ensure().ensured_crontab(&old_crontab).unwrap());
    }

    const TEST_ID: &str = "/test-role/cron/test";

    #[test]
    fn test_ensured_crontab_add() {
        let old_crontab = "1 2 3 4 5 do_thing\n\
                           2 2 3 4 5 do_other_thing\n\
                           ";

        let expected_crontab = format!(
            "1 2 3 4 5 do_thing\n\
             2 2 3 4 5 do_other_thing\n\
             {} {}\n\
             4 1,12 * * 1-5 /bin/command >/var/log/file\n\
             ",
            TAG_LINE, TEST_ID
        );

        assert_eq!(
            Some(expected_crontab),
            common_ensure().ensured_crontab(old_crontab).unwrap()
        );
    }

    #[test]
    fn test_removed_crontab_change() {
        let id = "/test-role/cron/test";

        let old_crontab = format!(
            "1 2 3 4 5 do_thing\n\
             {} {}\n\
             4 1,12 * * 1-5 /bin/command >/var/log/file\n\
             2 2 3 4 5 do_other_thing\n\
             ",
            TAG_LINE, id
        );

        let expected_crontab = "1 2 3 4 5 do_thing\n\
             2 2 3 4 5 do_other_thing\n\
             "
        .to_owned();

        assert_eq!(
            Some(expected_crontab),
            common_remove().removed_crontab(&old_crontab).unwrap()
        );
    }

    #[test]
    fn test_removed_crontab_not_there() {
        let old_crontab = "1 2 3 4 5 do_thing\n\
             2 2 3 4 5 do_other_thing\n\
             ";

        assert_eq!(None, common_remove().removed_crontab(old_crontab).unwrap());
    }

    fn common_ensure() -> GurpCron {
        GurpCron {
            action: Action::Ensure,
            id: TEST_ID.to_owned(),
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
            doer: "cron".to_owned(),
        }
    }

    fn common_remove() -> GurpCron {
        GurpCron {
            action: Action::Remove,
            id: TEST_ID.to_owned(),
            name: "Test job".to_owned(),
            user: "rob".to_owned(),
            desired_state: None,
            doer: "cron".to_owned(),
        }
    }
}
