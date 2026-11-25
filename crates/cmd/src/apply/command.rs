use super::lockfile::ApplyLock;
use super::report::{Report, ReportArgs, ReportStatus};
use super::{config, metrics};
use crate::apply::types::{ApplyStatus, FailPhase};
use camino::{Utf8Path, Utf8PathBuf};
use common::constants::{GURP_DIR_REPORT, GURP_LOCKFILE};
use common::types::{ApplyOpts, ApplySummary, CompileError};
use doers::types::Applicator;
use gurptel::{flush, types::TelemetryProviders};
use jiff::{SignedDuration, Timestamp};
use std::process::ExitCode;

struct ReportInfo {
    status: ReportStatus,
    fail_phase: Option<FailPhase>,
    summary: Option<ApplySummary>,
}

pub fn run(
    host_file: Option<&Utf8Path>,
    opts: &ApplyOpts,
    providers: TelemetryProviders,
) -> ExitCode {
    let maybe_lock = set_up_lock(opts);
    let exit_code = _run(host_file, maybe_lock.as_ref(), &providers, opts);

    if let Some(lock) = &maybe_lock
        && let Err(e) = lock.release()
    {
        tracing::warn!("failed to clean up lock file at {GURP_LOCKFILE}: {e}");
    }

    flush(providers);
    exit_code
}

fn flush(providers: TelemetryProviders) {
    if providers.are_active() {
        flush::flush(providers)
    }
}

// We don't use a lockfile for --exec applies, or for noops, and the user can request no
// lock file. (Which is useful for testing more than in real life.)
fn set_up_lock(opts: &ApplyOpts) -> Option<ApplyLock> {
    if opts.noop || opts.no_lock || opts.exec.is_some() {
        None
    } else {
        Some(ApplyLock::from(&Utf8PathBuf::from(GURP_LOCKFILE)))
    }
}

// We are only one level away from exiting here, so we'll log and handle errors and pass true or
// false back to the controlling function.
fn _run(
    host_file: Option<&Utf8Path>,
    maybe_lock: Option<&ApplyLock>,
    providers: &TelemetryProviders,
    opts: &ApplyOpts,
) -> ExitCode {
    let t_start = Timestamp::now();
    let send_metrics = providers.are_active();
    let mut ret = ExitCode::SUCCESS;
    let report_info: ReportInfo;

    let compile_result = config::compile(host_file, opts);

    let t_elapsed = Timestamp::now().duration_since(t_start);

    match compile_result {
        Err(e) => {
            report_info = report_compile_failure(&e, t_elapsed, send_metrics);
            ret = ExitCode::FAILURE
        }
        Ok(json_config) => {
            // If Gurp is locked, go no further.
            if let Some(lock) = maybe_lock {
                match lock.acquire() {
                    Ok(true) => (),
                    result @ (Ok(false) | Err(_)) => {
                        let t_elapsed = Timestamp::now().duration_since(t_start);
                        if let Err(e) = &result {
                            tracing::error!("error handling lock file: {e}");
                        } else {
                            tracing::warn!("gurp apply is locked by {GURP_LOCKFILE}");
                        }
                        let report_info = report_lock_error(t_elapsed, providers.are_active());
                        write_report(host_file, t_start, t_elapsed, report_info, opts);
                        return ExitCode::FAILURE;
                    }
                }
            }

            let run_result = Applicator::from(json_config).run(opts);
            tracing::info!("Run time: {:.3?}", t_elapsed);

            report_info = match run_result {
                Ok(apply_summary) => report_apply_success(apply_summary, t_elapsed, send_metrics),
                Err(e) => {
                    ret = ExitCode::FAILURE;
                    report_apply_failure(e, host_file, t_elapsed, send_metrics)
                }
            };
        }
    }

    write_report(host_file, t_start, t_elapsed, report_info, opts);
    ret
}

fn write_report(
    host_file: Option<&Utf8Path>,
    t_start: Timestamp,
    t_elapsed: SignedDuration,
    info: ReportInfo,
    opts: &ApplyOpts,
) {
    if opts.no_report || opts.exec.is_some() {
        return;
    }

    if let Err(e) = Report::from(ReportArgs {
        status: info.status,
        fail_phase: info.fail_phase,
        summary: info.summary.as_ref(),
        duration: t_elapsed,
        t_start,
        t_end: Timestamp::now(),
        host_file: host_file.map(|f| f.to_owned()),
        opts,
    })
    .write(&Utf8PathBuf::from(GURP_DIR_REPORT))
    {
        tracing::error!("failed to write report: {e}");
    }
}

fn report_lock_error(t_elapsed: SignedDuration, send_metrics: bool) -> ReportInfo {
    if send_metrics {
        metrics::send(ApplyStatus::Fail(FailPhase::Locked), t_elapsed);
    }

    ReportInfo {
        status: ReportStatus::Fail,
        fail_phase: Some(FailPhase::Locked),
        summary: None,
    }
}

fn report_apply_success(
    apply_summary: ApplySummary,
    t_elapsed: SignedDuration,
    send_metrics: bool,
) -> ReportInfo {
    tracing::info!(
        "resources: {}  changes: {}",
        apply_summary.resources,
        apply_summary.changes,
    );

    if send_metrics {
        metrics::send(ApplyStatus::Ok(apply_summary), t_elapsed);
    }

    ReportInfo {
        status: ReportStatus::Success,
        fail_phase: None,
        summary: Some(apply_summary),
    }
}

fn report_apply_failure(
    e: anyhow::Error,
    host_file: Option<&Utf8Path>,
    t_elapsed: SignedDuration,
    send_metrics: bool,
) -> ReportInfo {
    tracing::error!("FAILED TO APPLY CONFIG");

    if let Some(path) = host_file {
        tracing::error!("apply error on {path}: {e:#}");
    } else {
        tracing::error!("apply error: {e:#}");
    }

    if send_metrics {
        metrics::send(ApplyStatus::Fail(FailPhase::Apply), t_elapsed);
    }

    ReportInfo {
        status: ReportStatus::Fail,
        fail_phase: Some(FailPhase::Apply),
        summary: None,
    }
}

fn report_compile_failure(
    e: &CompileError,
    t_elapsed: SignedDuration,
    send_metrics: bool,
) -> ReportInfo {
    tracing::error!("could not generate config: {e:#} (Run with debug for stack trace)");

    if let CompileError::Compile { message, trace } = e {
        if message.contains("unknown symbol machine-config") {
            tracing::error!("config may lack a (host) definition");
        };

        for line in trace {
            tracing::debug!("{line}");
        }
    };

    if send_metrics {
        metrics::send(ApplyStatus::Fail(e.into()), t_elapsed);
    }

    ReportInfo {
        status: ReportStatus::Fail,
        fail_phase: Some(FailPhase::Compile),
        summary: None,
    }
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
        assert!(!logs_contain("sending fail metrics: fileNotFound"));
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
        assert!(!logs_contain("sending fail metrics: compile"));
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
                    no_lock: true,
                    ..Default::default()
                },
                TelemetryProviders::default(),
            )
        );

        assert!(!logs_contain("sending success metrics: 1/1"));
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
                    no_lock: true,
                    ..Default::default()
                },
                TelemetryProviders::default(),
            )
        );

        assert!(logs_contain("parent dir does not exist"));
        assert!(!logs_contain("sending fail metrics: apply"));
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
            "ERROR file_does_not_compile: cmd::apply::command: could not generate \
            config: compilation error: In directory/ensure /tmp/testdir: unexpected \
            property :bad-key."
        ));
        assert!(!logs_contain("sending fail metrics: compile"));
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
                    no_lock: true,
                    ..Default::default()
                },
                TelemetryProviders::default(),
            )
        );

        assert!(!logs_contain("sending success metrics: 0/1"));
        assert!(!logs_contain("resources: 1 changes: 0"));
    }
}
