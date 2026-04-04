use crate::apply::lockfile::ApplyLock;
use camino::Utf8PathBuf;
use common::constants::APPLY_LOCKFILE;
use common::types::ApplyOpts;
use doers::types::Applicator;
use embed::compiler;
use std::process::ExitCode;
use std::time::Instant;
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

pub fn run(host_file: Option<&Utf8PathBuf>, opts: &ApplyOpts) -> ExitCode {
    let start_time = Instant::now();

    if let Some(file) = host_file
        && !file.exists()
    {
        tracing::error!("config file not found: {file}");
        return ExitCode::FAILURE;
    }

    let provider = init::init_metrics(opts.metrics_to.as_deref(), "gurp").unwrap_or_else(|e| {
        tracing::warn!("could not set up metrics: {e:#}");
        None
    });

    let client_metrics = ClientMetrics::new();

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
                return ExitCode::FAILURE; // is that a fail?
            }
            Err(e) => {
                tracing::error!("error checking lockfile: {e:#}");
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
                clean_up_lock!(lock);
                return ExitCode::FAILURE;
            }
        }
    } else {
        match compiler::compile_to_json(host_file, opts) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("error compiling config: {e:#}");
                clean_up_lock!(lock);
                return ExitCode::FAILURE;
            }
        }
    };

    let run_result = Applicator::from(json_config).run(opts);
    let elapsed_time = start_time.elapsed();
    let elapsed_ms = elapsed_time.as_millis();
    let mut exit = ExitCode::SUCCESS;

    tracing::info!("Run time: {:.3?}", elapsed_time);

    match run_result {
        Ok(apply_summary) => {
            tracing::info!(
                "resources: {}  changes: {}",
                apply_summary.resources,
                apply_summary.changes,
            );
            client_metrics.record_apply_duration("ok", elapsed_ms as u64);
            client_metrics.record_apply_changes(apply_summary.changes as u64);
            client_metrics.record_apply_resources(apply_summary.resources as u64);

            if let Some(rss) = runtime_stats::rss_bytes() {
                client_metrics.record_apply_rss("ok", rss as u64);
            }
        }
        Err(e) => {
            if let Some(host_file) = host_file {
                tracing::error!("apply error on {host_file}: {e:#}");
            } else {
                tracing::error!("apply error: {e:#}");
            }

            client_metrics.record_apply_duration("fail", elapsed_ms as u64);

            if let Some(rss) = runtime_stats::rss_bytes() {
                client_metrics.record_apply_rss("fail", rss as u64);
            }

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
