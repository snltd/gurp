#[macro_export]
macro_rules! debug {
    ($opts:expr, $component:literal, $($arg:tt)*) => {
        if $opts.debug {
            println!("DEBUG [{}] {}", $component, format!($($arg)*));
        }
    };
}

#[macro_export]
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

#[macro_export]
macro_rules! one_change_or_stderr {
    ($cmd:expr) => {{
        let output = $cmd.output()?;

        if output.status.success() {
            Ok($crate::common::constants::ONE_RESOURCE_ONE_CHANGE)
        } else {
            bail!(String::from_utf8_lossy(&output.stderr).into_owned())
        }
    }};
}

#[macro_export]
macro_rules! return_if_noop {
    ($opts:expr) => {
        if $opts.noop {
            return Ok($crate::common::constants::ONE_RESOURCE_NOOP);
        }
    };
}

#[macro_export]
macro_rules! cmd {
    ( $bin:expr, $( $arg:expr ),+ $(,)? ) => {{
        use std::process::{Command, Stdio};
        let mut cmd = Command::new($bin);
        $(
            cmd.arg($arg);
        )+
        cmd.stderr(Stdio::piped());

        tracing::debug!(command = $crate::utils::helpers::command_to_string(&cmd));

        cmd
    }};
}

#[macro_export]
macro_rules! cmd_output {
    ( $noop:expr, $bin:expr, $( $arg:expr ),+ $(,)? ) => {{
        use std::process::{Command, Stdio};

        let mut cmd = Command::new($bin);
        $(
            cmd.arg($arg);
        )+
        cmd.stderr(Stdio::piped());

        tracing::debug!(command = $crate::utils::helpers::command_to_string(&cmd));

        if $noop {
            return Ok($crate::ONE_RESOURCE_NOOP.to_string());
        }

        let output = cmd.output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            bail!(
                "{}",
                String::from_utf8_lossy(&output.stderr).into_owned()
            );
        }
    }};
}
