#[macro_export]
#[allow(unused_macros)]
macro_rules! apply_resources {
    ($summary_total:ident, $changed_ids:ident, $resources:expr, $opts:expr) => {
        let total_count = $resources.len();
        for (i, resource) in $resources.iter().enumerate() {
            let chunks: Vec<_> = resource.id.split("/").collect();
            if chunks.len() >= 3 {
                tracing::debug!(
                    "applying {} {}/{}: {}",
                    chunks[1],
                    i + 1,
                    total_count,
                    resource.id
                );
            } else {
                tracing::debug!("applying [{}/{}]: {}", i + 1, total_count, resource.id);
            }
            let summary = match resource.apply($opts) {
                Ok(summary) => summary,
                Err(e) => {
                    tracing::error!("from {} doer: {}", chunks[2], e);
                    let err: anyhow::Error = e.into();
                    return Err(err.context(format!("failed to apply resource {}", resource.id)));
                }
            };
            $summary_total = $summary_total + summary;
            if summary.changes > 0 {
                $changed_ids.insert(resource.id.clone());
            }
        }
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! one_change_or_stderr {
    ($cmd:expr, $msg:expr) => {{
        let output = $cmd.output()?;

        if output.status.success() {
            Ok(common::constants::ONE_RESOURCE_ONE_CHANGE)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("{}: {}", $msg, stderr.trim());
        }
    }};

    ($cmd:expr) => {{
        let output = $cmd.output()?;

        if output.status.success() {
            Ok(common::constants::ONE_RESOURCE_ONE_CHANGE)
        } else {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
        }
    }};
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! return_if_noop {
    ($opts:expr) => {
        if $opts.noop {
            return Ok(common::constants::ONE_RESOURCE_NOOP);
        }
    };

    ($opts:expr, $resources:expr, $changes:expr) => {
        if $opts.noop {
            return Ok(common::types::ApplySummary {
                resources: $resources,
                changes: $changes,
            });
        }
    };
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! cmd {
    ( $bin:expr $(, $arg:expr )* $(,)? ) => {{
        use std::process::{Command, Stdio};
        let mut cmd = Command::new($bin);
        $(
            cmd.arg($arg);
        )*
        cmd.stderr(Stdio::piped());

        tracing::debug!(command = common::helpers::command_to_string(&cmd));

        cmd
    }};
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! run_cmd {
    ( $cmd:expr ) => {{
        let output = $cmd.output()?;

        if output.status.success() {
            Result::<String, anyhow::Error>::Ok(
                String::from_utf8_lossy(&output.stdout).trim().to_owned(),
            )
        } else {
            anyhow::bail!(
                "cmd_output error: {}",
                String::from_utf8_lossy(&output.stderr).into_owned()
            );
        }
    }};
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! cmd_with_stdin {
    ( $bin:expr $(, $arg:expr )* $(,)? ) => {{
        use std::process::{Command, Stdio};
        let mut cmd = Command::new($bin);
        $(
            cmd.arg($arg);
        )*
        cmd.stdin(Stdio::piped());
        cmd.stderr(Stdio::piped());

        tracing::debug!(command = common::helpers::command_to_string(&cmd));

        cmd
    }};
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! cmd_output {
    ( $bin:expr, $( $arg:expr ),+ $(,)? ) => {{
        let mut cmd = cmd!($bin, $($arg), +);
        let output = cmd.output()?;

        if output.status.success() {
            Result::<String, anyhow::Error>::Ok(
                String::from_utf8_lossy(&output.stdout).trim().to_owned()
            )
        } else {
            anyhow::bail!(
                "cmd_output error: {}",
                String::from_utf8_lossy(&output.stderr).into_owned()
            );
        }
    }};
}

#[cfg(test)]
mod test {
    use common::constants::{ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE};
    use common::types::{ApplyOpts, ApplySummary};
    use predicates::prelude::*;
    use tester::{defopts, defopts_noop};

    #[test]
    fn test_cmd_macro_valid_command() {
        let mut cmd = cmd!("/usr/bin/true");
        let result = cmd.output().unwrap();
        assert!(result.status.success());
    }

    #[test]
    fn test_cmd_macro_valid_command_with_args() {
        let mut cmd = cmd!("/bin/ls", "-l", "/");
        let result = cmd.output().unwrap();
        assert!(result.status.success());
        let ls_output = String::from_utf8_lossy(&result.stdout);
        assert!(predicate::str::contains(" root ").eval(&ls_output));
    }

    #[test]
    fn test_cmd_macro_invalid_command() {
        let mut cmd = cmd!("/bin/nonsense", "-o", "rubbish");
        assert!(cmd.status().is_err());
    }

    #[test]
    fn test_cmd_output_macro_valid_command_with_args() -> anyhow::Result<()> {
        let output = cmd_output!("/bin/echo", "merp", "merp", "merp")?;
        assert_eq!("merp merp merp", &output);
        Ok(())
    }

    #[test]
    fn test_cmd_output_macro_invalid_command() -> anyhow::Result<()> {
        let result = (|| cmd_output!("/bin/nonsense", "merp", "merp", "merp"))();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_return_if_noop_macro() {
        fn wrapper(opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
            return_if_noop!(opts);
            Ok(ONE_RESOURCE_ONE_CHANGE)
        }

        assert_eq!(ONE_RESOURCE_NOOP, wrapper(&defopts_noop()).unwrap());
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, wrapper(&defopts()).unwrap());
    }

    #[test]
    fn test_one_change_or_stderr_macro_ok() {
        fn wrapper() -> anyhow::Result<ApplySummary> {
            let mut cmd = cmd!("/bin/echo");
            one_change_or_stderr!(cmd)
        }

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, wrapper().unwrap());
    }

    #[test]
    fn test_one_change_or_stderr_macro_err() {
        fn wrapper() -> anyhow::Result<ApplySummary> {
            let mut cmd = cmd!("/bin/chubb");
            one_change_or_stderr!(cmd)
        }

        assert!(wrapper().is_err());
    }
}
