use crate::apply::lockfile::ApplyLock;
use crate::apply::types::{ApplyStatus, FailPhase};
use camino::Utf8Path;
use common::constants::APPLY_LOCKFILE;
use common::types::ApplyOpts;
use doers::types::Applicator;
use embed::compiler;
use std::process::ExitCode;
use std::time::{Duration, Instant};
use util::metrics::client::ClientMetrics;
use util::metrics::init;
use util::runtime_stats;

macro_rules! clean_up_lock {
    ($lock: expr) => {
        if let Some(lock) = $lock
            && let Err(e) = lock.remove()
        {
            tracing::warn!("could not remove lock file at {}: {e:#}", lock.path);
        }
    };
}

pub fn run(host_file: Option<&Utf8Path>, opts: &ApplyOpts) -> ExitCode {
    let start_time = Instant::now();

    if let Some(file) = host_file
        && !file.exists()
    {
        tracing::error!("config file not found: {file}");

        do_metrics(
            ApplyStatus::Fail(FailPhase::FileNotFound),
            &start_time.elapsed(),
        );

        return ExitCode::FAILURE;
    }

    let provider = init::init_metrics(opts.metrics_to.as_deref(), "gurp").unwrap_or_else(|e| {
        tracing::warn!("could not set up metrics: {e:#}");
        None
    });

    let lock = if opts.no_lock || opts.exec.is_some() {
        None
    } else {
        Some(ApplyLock::from(APPLY_LOCKFILE))
    };

    if let Some(lock) = &lock {
        match lock.is_locked() {
            Ok(false) => (),
            Ok(true) => {
                tracing::info!("execution blocked by lockfile");
                do_metrics(ApplyStatus::Fail(FailPhase::Locked), &start_time.elapsed());
                return ExitCode::FAILURE; // is that a fail?
            }
            Err(e) => {
                tracing::error!("error checking lockfile: {e:#}");
                do_metrics(ApplyStatus::Fail(FailPhase::Locked), &start_time.elapsed());
                return ExitCode::FAILURE;
            }
        }

        if let Err(e) = lock.create() {
            tracing::warn!("could not create lock file at {}: {e:#}", lock.path);
        }
    }

    let json_config = if let Some(janet_snippet) = &opts.exec {
        match compiler::raw_janet_to_json(janet_snippet, opts) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("error compiling snippet: {e:#}");
                do_metrics(ApplyStatus::Fail(FailPhase::Compile), &start_time.elapsed());
                clean_up_lock!(lock);
                return ExitCode::FAILURE;
            }
        }
    } else {
        match compiler::compile_to_json(host_file, opts) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("error compiling config: {e:#}");

                do_metrics(
                    ApplyStatus::Fail(FailPhase::from(&e)),
                    &start_time.elapsed(),
                );

                clean_up_lock!(lock);
                return ExitCode::FAILURE;
            }
        }
    };

    let run_result = Applicator::from(json_config).run(opts);
    let elapsed_time = start_time.elapsed();
    let mut exit = ExitCode::SUCCESS;

    tracing::info!("Run time: {:.3?}", elapsed_time);

    match run_result {
        Ok(apply_summary) => {
            tracing::info!(
                "resources: {}  changes: {}",
                apply_summary.resources,
                apply_summary.changes,
            );
            do_metrics(ApplyStatus::Ok(apply_summary), &elapsed_time);
        }
        Err(e) => {
            if let Some(host_file) = host_file {
                tracing::error!("apply error on {host_file}: {e:#}");
            } else {
                tracing::error!("apply error: {e:#}");
            }

            do_metrics(ApplyStatus::Fail(FailPhase::Apply), &elapsed_time);
            exit = ExitCode::FAILURE;
        }
    }

    clean_up_lock!(lock);

    if let Some(p) = provider {
        if let Err(e) = p.force_flush() {
            tracing::warn!("failed to flush metrics: {e:#}");
        }

        if let Err(e) = p.shutdown() {
            tracing::warn!("failed to shut down OTEL provider: {e:#}");
        }
    }

    exit
}

fn do_metrics(status: ApplyStatus, elapsed_time: &Duration) {
    let metrics_handle = ClientMetrics::new();
    let elapsed_ms = elapsed_time.as_millis();

    match status {
        ApplyStatus::Ok(summary) => {
            metrics_handle.record_apply_duration("ok", elapsed_ms as u64, None);
            metrics_handle.record_apply_changes(summary.changes as u64);
            metrics_handle.record_apply_resources(summary.resources as u64);
            #[cfg(test)]
            tracing::info!(
                "sending success metrics: {}/{}",
                summary.changes,
                summary.resources
            );
        }
        ApplyStatus::Fail(phase) => {
            metrics_handle.record_apply_duration(
                "fail",
                elapsed_ms as u64,
                Some(&phase.to_string()),
            );
            #[cfg(test)]
            tracing::info!("sending fail metrics: {phase}",);
        }
    }

    if let Some(rss) = runtime_stats::rss_bytes() {
        metrics_handle.record_apply_rss("ok", rss as u64);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use camino::Utf8PathBuf;
    use camino_tempfile_ext::prelude::*;
    use tester::defopts;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_run_no_file() {
        assert_eq!(
            ExitCode::FAILURE,
            run(Some(&Utf8PathBuf::from("/no/such/file")), &defopts())
        );

        assert!(logs_contain("config file not found"));
        assert!(logs_contain("sending fail metrics: fileNotFound"));
        assert!(!logs_contain("resources:"));
    }

    #[test]
    #[traced_test]
    fn test_snippet_does_not_compile() {
        assert_eq!(
            ExitCode::FAILURE,
            run(
                None,
                &ApplyOpts {
                    exec: Some("(not valid janet".to_owned()),
                    ..Default::default()
                }
            )
        );

        assert!(logs_contain(
            "error compiling snippet: compilation error: Failed to parse code"
        ));
        assert!(logs_contain("sending fail metrics: compile"));
        assert!(!logs_contain("resources:"));
    }

    #[test]
    #[traced_test]
    fn test_snippet_noop_success() {
        assert_eq!(
            ExitCode::SUCCESS,
            run(
                None,
                &ApplyOpts {
                    exec: Some(r#"(directory/ensure "/tmp/test")"#.to_owned()),
                    noop: true,
                    ..Default::default()
                }
            )
        );

        assert!(logs_contain("sending success metrics: 1/1"));
        assert!(logs_contain("resources: 1  changes: 1"));
    }

    #[test]
    #[traced_test]
    fn test_snippet_fails() {
        // This should always fail because the parent does not exist
        assert_eq!(
            ExitCode::FAILURE,
            run(
                None,
                &ApplyOpts {
                    exec: Some(r#"(file/ensure "/parent/does/not/exist" :content "x")"#.to_owned()),
                    ..Default::default()
                }
            )
        );

        assert!(logs_contain("parent dir does not exist"));
        assert!(logs_contain("sending fail metrics: apply"));
        assert!(!logs_contain("resources:"));
    }

    #[test]
    #[traced_test]
    fn file_does_not_compile() {
        let temp = Utf8TempDir::new().unwrap();
        let file = temp.child("bad-test.janet");
        file.write_str(indoc::indoc! {
        r#"(host "bad-janet"
                    (directory/ensure "/tmp/testdir"
                        :bad-key 123))"#
                    })
        .unwrap();

        assert_eq!(
            ExitCode::FAILURE,
            run(
                Some(file.as_path()),
                &ApplyOpts {
                    no_lock: true,
                    ..Default::default()
                }
            )
        );

        assert!(logs_contain(
            "ERROR file_does_not_compile: commands::apply::init: error compiling config: compilation error: In directory/ensure /tmp/testdir: unexpected property :bad-key. Valid properties are :owner, :group, :mode, :label"
        ));
        assert!(logs_contain("sending fail metrics: compile"));
        assert!(!logs_contain("resources:"));
    }

    #[test]
    #[traced_test]
    fn file_works() {
        let temp = Utf8TempDir::new().unwrap();
        let file = temp.child("bad-test.janet");
        file.write_str(indoc::indoc! {
        r#"(host "good-janet"
             (directory/remove "/this/does/not/exist"))"#
        })
        .unwrap();

        assert_eq!(
            ExitCode::SUCCESS,
            run(
                Some(file.as_path()),
                &ApplyOpts {
                    no_lock: true,
                    ..Default::default()
                }
            )
        );

        assert!(logs_contain("sending success metrics: 0/1"));
        assert!(!logs_contain("resources: 1 changes: 0"));
    }
}
