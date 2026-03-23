use crate::apply::lockfile::ApplyLock;
use crate::apply::report;
use camino::Utf8PathBuf;
use common::constants::APPLY_LOCKFILE;
use common::types::ApplyOpts;
use doers::types::Applicator;
use embed::compiler;
use std::process::ExitCode;
use std::time::Instant;

macro_rules! clean_up_lock {
    ($lock: expr) => {
        if let Some(lock) = $lock
            && let Err(e) = lock.remove()
        {
            tracing::warn!("could not remove lock file at {}: {e}", lock.path);
        }
    };
}

pub fn run(host_file: Option<&Utf8PathBuf>, opts: &ApplyOpts) -> ExitCode {
    if let Some(file) = host_file
        && !file.exists()
    {
        tracing::error!("config file not found: {file}");
        return ExitCode::FAILURE;
    }

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
                tracing::error!("error checking lockfile: {e}");
                return ExitCode::FAILURE;
            }
        }

        if let Err(e) = lock.create() {
            tracing::warn!("could not create lock file at {}: {e}", lock.path);
        }
    }

    let start_time = Instant::now();

    let json_config = if let Some(janet_snippet) = &opts.exec {
        match compiler::raw_janet_to_json(janet_snippet, opts) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("error compiling snippet: {e}");
                clean_up_lock!(lock);
                return ExitCode::FAILURE;
            }
        }
    } else {
        match compiler::compile_to_json(host_file, opts) {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("error compiling config: {e}");
                clean_up_lock!(lock);
                return ExitCode::FAILURE;
            }
        }
    };

    let applicator = Applicator::from(json_config);
    let mut exit = ExitCode::SUCCESS;

    match applicator.run(opts) {
        Ok(apply_summary) => {
            let elapsed_time = start_time.elapsed();
            report::success(&apply_summary, elapsed_time, opts.metrics_to.as_deref());
        }
        Err(e) => {
            if let Some(host_file) = host_file {
                tracing::error!("apply error on {host_file}: {e}");
            } else {
                tracing::error!("apply error: {e}");
            }

            let elapsed_time = start_time.elapsed();
            report::failure(elapsed_time, opts.metrics_to.as_deref());
            exit = ExitCode::FAILURE;
        }
    }

    clean_up_lock!(lock);
    exit
}
