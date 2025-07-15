macro_rules! debug {
    ($opts:expr, $component:literal, $($arg:tt)*) => {
        if $opts.debug {
            println!("DEBUG [{}] {}", $component, format!($($arg)*));
        }
    };
}

macro_rules! apply_resources {
    ($summary_total:ident, $changed_ids:ident, $resources:expr, $opts:expr) => {
        for resource in $resources {
            tracing::debug!("applying resource '{}'", resource.id);
            let summary = resource.apply($opts)?;
            $summary_total = $summary_total + summary;
            if summary.changes > 0 {
                $changed_ids.insert(resource.id.clone());
            }
        }
    };
}

macro_rules! one_change_or_stderr {
    ($cmd:expr, $msg:expr) => {{
        let output = $cmd.output()?;

        if output.status.success() {
            Ok($crate::common::constants::ONE_RESOURCE_ONE_CHANGE)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("{}: {}", $msg, stderr.trim());
        }
    }};

    ($cmd:expr) => {{
        let output = $cmd.output()?;

        if output.status.success() {
            Ok($crate::common::constants::ONE_RESOURCE_ONE_CHANGE)
        } else {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
        }
    }};
}

macro_rules! return_if_noop {
    ($opts:expr) => {
        if $opts.noop {
            return Ok($crate::common::constants::ONE_RESOURCE_NOOP);
        }
    };
}

macro_rules! cmd {
    ( $bin:expr $(, $arg:expr )* $(,)? ) => {{
        use std::process::{Command, Stdio};
        let mut cmd = Command::new($bin);
        $(
            cmd.arg($arg);
        )*
        cmd.stderr(Stdio::piped());

        tracing::debug!(command = $crate::utils::helpers::command_to_string(&cmd));

        cmd
    }};
}

macro_rules! cmd_output {
    ( $bin:expr, $( $arg:expr ),+ $(,)? ) => {{
        let mut cmd = cmd!($bin, $($arg), +);
        let output = cmd.output()?;

        if output.status.success() {
            Result::<String, anyhow::Error>::Ok(
                String::from_utf8_lossy(&output.stdout).into_owned()
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
    use crate::common::constants::{ONE_RESOURCE_NOOP, ONE_RESOURCE_ONE_CHANGE};
    use crate::common::types::{ApplySummary, Opts};
    use crate::test_utils::spec_helper::{defopts, defopts_noop};
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
        assert_eq!("merp merp merp\n", &output);
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
        fn wrapper(opts: &Opts) -> anyhow::Result<ApplySummary> {
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
