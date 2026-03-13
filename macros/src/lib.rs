use std::process::{Command, Output};

pub fn log_error(cmd: &Command, output: Output) -> String {
    let cmd = common::cmd::to_string(cmd);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    tracing::error!(command = cmd, stdout = stdout, stderr = stderr);
    "error running external command".to_owned()
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! one_change_or_stderr {
    ($cmd:expr, $msg:expr) => {{
        let output = $cmd.output()?;

        if output.status.success() {
            Ok(common::constants::ONE_RESOURCE_ONE_CHANGE)
        } else {
            anyhow::bail!($crate::log_error(&$cmd, output))
        }
    }};

    ($cmd:expr) => {{
        let output = $cmd.output()?;

        if output.status.success() {
            Ok(common::constants::ONE_RESOURCE_ONE_CHANGE)
        } else {
            anyhow::bail!($crate::log_error(&$cmd, output))
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

/// Builds a command from its args, returning a Command. Logs the constructed command
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

        tracing::debug!(command = common::cmd::to_string(&cmd));

        cmd
    }};
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! cmd_change_or_noop{
    ( $opts:expr, $bin:expr $(, $arg:expr )* $(,)? ) => {{
        use std::process::{Command, Stdio};
        let mut cmd = Command::new($bin);
        $(
            cmd.arg($arg);
        )*
        cmd.stderr(Stdio::piped());

        tracing::debug!(command = common::cmd::to_string(&cmd));

        if !$opts.noop {
            let output = cmd.output()?;

            if output.status.success() {
                anyhow::bail!($crate::log_error(&cmd, output))
            }
        }

        Result::<common::types::ApplySummary, anyhow::Error>::Ok(common::constants::ONE_RESOURCE_ONE_CHANGE)
    }};
}

/// Receives a Command and runs it, returning a result of the standard out
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
            anyhow::bail!($crate::log_error(&$cmd, output))
        }
    }};
}

/// Builds and returns a Command from its args.
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

        tracing::debug!(command = common::cmd::to_string(&cmd));

        cmd
    }};
}

/// Builds a Command from its args, and executes that command, returning a result of stdout
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
            anyhow::bail!($crate::log_error(&cmd, output))
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
