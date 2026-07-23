use super::report::{Report, ReportArgs, ReportStatus};
use super::{config, lockfile, metrics};
use crate::apply::types::{ApplyStatus, FailPhase};
use camino::{Utf8Path, Utf8PathBuf};
use common::types::{ApplyOpts, CompileError};
use doers::types::Applicator;
use gurptel::{flush, types::TelemetryProviders};
use jiff::Timestamp;
use std::process::ExitCode;
use std::time::Instant;

const REPORT_DIR: &str = "/var/log";

pub fn run(
    host_file: Option<&Utf8Path>,
    opts: &ApplyOpts,
    providers: TelemetryProviders,
) -> ExitCode {
    let t_start = Timestamp::now(); // For the report
    let start_time = Instant::now(); // For metrics

    let exit_code = match config::compile(host_file, opts) {
        Ok(cfg) => {
            let lock = lockfile::acquire(opts);

            if lockfile::is_on(&start_time, &lock) {
                ExitCode::FAILURE
            } else {
                let run_result = Applicator::from(cfg).run(opts);
                let elapsed_time = start_time.elapsed();
                tracing::info!("Run time: {:.3?}", elapsed_time);

                let run_exit_code = match run_result {
                    Ok(apply_summary) => {
                        tracing::info!(
                            "resources: {}  changes: {}",
                            apply_summary.resources,
                            apply_summary.changes,
                        );

                        if !opts.no_report && opts.exec.is_none() {
                            Report::from(ReportArgs {
                                status: ReportStatus::Success,
                                fail_phase: None,
                                summary: Some(&apply_summary),
                                duration: &elapsed_time,
                                t_start,
                                t_end: Timestamp::now(),
                                host_file: host_file.map(|f| f.to_owned()),
                                opts,
                            })
                            .write(&Utf8PathBuf::from(REPORT_DIR));
                        }

                        metrics::send(ApplyStatus::Ok(apply_summary), &elapsed_time);
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        tracing::error!("FAILED TO APPLY CONFIG");
                        if let Some(path) = host_file {
                            tracing::error!("apply error on {path}: {e:#}");
                        } else {
                            tracing::error!("apply error: {e:#}");
                        }

                        if !opts.no_report && opts.exec.is_none() {
                            Report::from(ReportArgs {
                                status: ReportStatus::Fail,
                                fail_phase: Some(FailPhase::Apply),
                                summary: None,
                                duration: &elapsed_time,
                                t_start,
                                t_end: Timestamp::now(),
                                host_file: host_file.map(|f| f.to_owned()),
                                opts,
                            })
                            .write(&Utf8PathBuf::from(REPORT_DIR));
                        }

                        metrics::send(ApplyStatus::Fail(FailPhase::Apply), &elapsed_time);
                        ExitCode::FAILURE
                    }
                };

                lockfile::release(lock);
                run_exit_code
            }
        }

        Err(e) => {
            tracing::error!("could not generate config: {e:#}");

            if let CompileError::Compile { message, trace } = &e {
                if message.contains("unknown symbol machine-config") {
                    tracing::error!("config may lack a (host) definition");
                };

                tracing::debug!(janet_trace = trace.join("\n"));
            };

            metrics::send(ApplyStatus::Fail(e.into()), &start_time.elapsed());
            ExitCode::FAILURE
        }
    };

    flush::flush(providers);
    exit_code
}

#[cfg(test)]
mod test {
    use super::*;
    use camino::Utf8PathBuf;
    use camino_tempfile_ext::prelude::*;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_run_no_file() {
        assert_eq!(
            ExitCode::FAILURE,
            run(
                Some(&Utf8PathBuf::from("/no/such/file")),
                &ApplyOpts::default(),
                TelemetryProviders::default(),
            )
        );

        assert!(logs_contain(
            "could not generate config: missing file error: /no/such/file"
        ));
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
                },
                TelemetryProviders::default(),
            )
        );

        assert!(logs_contain(
            "could not generate config: compile error: Failed to parse code"
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
                },
                TelemetryProviders::default(),
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
                },
                TelemetryProviders::default(),
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
                &ApplyOpts::default(),
                TelemetryProviders::default(),
            )
        );

        assert!(logs_contain(
            "ERROR file_does_not_compile: commands::apply::command: could not generate \
            config: compilation error: In directory/ensure /tmp/testdir: unexpected \
            property :bad-key."
        ));
        assert!(logs_contain("sending fail metrics: compile"));
        assert!(!logs_contain("resources:"));
    }

    #[test]
    #[traced_test]
    fn file_works() {
        let temp = Utf8TempDir::new().unwrap();
        let file = temp.child("good-test.janet");
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
                    no_report: true,
                    ..Default::default()
                },
                TelemetryProviders::default(),
            )
        );

        assert!(logs_contain("sending success metrics: 0/1"));
        assert!(!logs_contain("resources: 1 changes: 0"));
    }
}
