use std::process::{Command, Output};

pub fn log_error(cmd: &Command, output: Output) -> String {
    let cmd = common::cmd::to_string(cmd);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let exit_code = &output.status.code();
    tracing::error!(
        command = cmd,
        exit_code = exit_code,
        stdout = stdout,
        stderr = stderr
    );
    "error running external command".to_owned()
}

/// Builds a command from its args, returning a Command. Logs the constructed command
#[macro_export]
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

            if !output.status.success() {
                anyhow::bail!($crate::log_error(&cmd, output))
            }
        }

        Result::<common::types::ApplySummary, anyhow::Error>::Ok(common::constants::ONE_RESOURCE_ONE_CHANGE)
    }};
}

/// Receives a Command and runs it, returning a result of the standard out
#[macro_export]
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

/// Is exactly one of the Options a Some?
#[macro_export]
macro_rules! exactly_one_some {
    ($($opt:expr),+) => {
        [$($opt.is_some()),+].iter().filter(|&&x| x).count() == 1
    };
}

#[cfg(test)]
mod test {
    use predicates::prelude::*;

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
}
