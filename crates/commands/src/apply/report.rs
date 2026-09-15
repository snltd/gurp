use super::types::FailPhase;
use camino::{Utf8Path, Utf8PathBuf};
use common::constants::GURP_VERSION;
use common::types::{ApplyOpts, ApplySummary};
use jiff::Timestamp;
use serde::Serialize;
use serde_json;
use std::fs;
use std::time::Duration;
use util::info;

// Makes a best-effort to write a run report. Swallows any errors. No report is written
// if we're execing a snippet.
//
#[derive(Serialize)]
pub(crate) enum ReportStatus {
    Success,
    Fail,
}

impl std::fmt::Display for ReportStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            ReportStatus::Success => "SUCCESS",
            ReportStatus::Fail => "FAIL",
        };
        write!(f, "{}", s)
    }
}

pub(crate) struct ReportArgs<'a> {
    pub status: ReportStatus,
    pub fail_phase: Option<FailPhase>,
    pub summary: Option<&'a ApplySummary>,
    pub duration: &'a Duration,
    pub t_start: Timestamp,
    pub t_end: Timestamp,
    pub host_file: Option<Utf8PathBuf>,
    pub opts: &'a ApplyOpts,
}

#[derive(Serialize)]
pub(crate) struct Report {
    status: String,
    hostname: String,
    host_file: Option<Utf8PathBuf>,
    resources: Option<u32>,
    changes: Option<u32>,
    t_start: Timestamp,
    t_end: Timestamp,
    duration: Duration,
    gurp_version: String,
    gurp_build: String,
    server: Option<String>,
    client_name: Option<String>,
    precompiled: bool,
    fail_phase: Option<FailPhase>,
}

impl Report {
    pub(crate) fn from(args: ReportArgs) -> Self {
        Self {
            status: args.status.to_string(),
            fail_phase: args.fail_phase,
            hostname: info::my_hostname().unwrap_or("UNKNOWN".to_owned()),
            host_file: args.host_file,
            resources: args.summary.map(|s| s.resources),
            changes: args.summary.map(|s| s.changes),
            t_start: args.t_start,
            t_end: args.t_end,
            duration: args.duration.to_owned(),
            gurp_version: GURP_VERSION.to_owned(),
            gurp_build: info::BUILD_HASH.to_owned(),
            server: args.opts.client.server.clone(),
            client_name: args.opts.client.hostname.clone(),
            precompiled: args.opts.precompiled,
        }
    }

    pub fn write(&self, dir: &Utf8Path) {
        let filename = if self.fail_phase.is_some() {
            "gurp_last_fail.json"
        } else {
            "gurp_last_success.json"
        };

        let path = dir.join(filename);

        match serde_json::to_string_pretty(&self) {
            Ok(json) => {
                if let Err(e) = fs::write(&path, json) {
                    eprintln!("failed to write report to {path}: {e}");
                }
            }
            Err(e) => eprintln!("failed to create report JSON: {e}"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use camino_tempfile_ext::prelude::*;
    use common::types::ApplyOpts;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_success_report() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let expected_file = temp_dir.path().join("gurp_last_success.json");

        let t_start: Timestamp = "2026-06-14 12:00:00-00".parse().unwrap();
        let t_end: Timestamp = "2026-06-14 12:00:06-00".parse().unwrap();

        Report::from(ReportArgs {
            status: ReportStatus::Success,
            fail_phase: None,
            summary: Some(&ApplySummary {
                resources: 123,
                changes: 45,
            }),
            duration: &Duration::from_millis(6789),
            t_start,
            t_end,
            host_file: None,
            opts: &ApplyOpts::default(),
        })
        .write(temp_dir.path());

        let content = fs::read_to_string(&expected_file).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(v["status"], "SUCCESS");
        assert_eq!(v["resources"], 123);
        assert_eq!(v["changes"], 45);
        assert_eq!(v["t_start"], "2026-06-14T12:00:00Z");
        assert_eq!(v["t_end"], "2026-06-14T12:00:06Z");
        assert_eq!(v["duration"]["secs"], 6);
        assert_eq!(v["duration"]["nanos"], 789_000_000);
        assert_eq!(v["fail_phase"], serde_json::Value::Null);
        assert_eq!(v["host_file"], serde_json::Value::Null);
        assert!(v["hostname"].as_str().is_some_and(|s| !s.is_empty()));
        assert!(v["gurp_version"].as_str().is_some_and(|s| !s.is_empty()));
        assert!(v["gurp_build"].as_str().is_some_and(|s| !s.is_empty()));
    }

    #[test]
    fn test_fail_report() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let expected_file = temp_dir.path().join("gurp_last_fail.json");

        let t_start: Timestamp = "2026-06-14 12:00:00-00".parse().unwrap();
        let t_end: Timestamp = "2026-06-14 12:00:06-00".parse().unwrap();

        Report::from(ReportArgs {
            status: ReportStatus::Fail,
            fail_phase: Some(FailPhase::Apply),
            summary: None,
            duration: &Duration::from_millis(6789),
            t_start,
            t_end,
            host_file: None,
            opts: &ApplyOpts::default(),
        })
        .write(temp_dir.path());

        let content = fs::read_to_string(&expected_file).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(v["status"], "FAIL");
        assert_eq!(v["resources"], serde_json::Value::Null);
        assert_eq!(v["changes"], serde_json::Value::Null);
        assert_eq!(v["t_start"], "2026-06-14T12:00:00Z");
        assert_eq!(v["t_end"], "2026-06-14T12:00:06Z");
        assert_eq!(v["duration"]["secs"], 6);
        assert_eq!(v["duration"]["nanos"], 789_000_000);
        assert_eq!(v["fail_phase"], "Apply");
        assert_eq!(v["host_file"], serde_json::Value::Null);
        assert!(v["hostname"].as_str().is_some_and(|s| !s.is_empty()));
        assert!(v["gurp_version"].as_str().is_some_and(|s| !s.is_empty()));
        assert!(v["gurp_build"].as_str().is_some_and(|s| !s.is_empty()));
    }
}
