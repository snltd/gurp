use crate::server::http;
use common::types::ServerOpts;
use std::process::ExitCode;
use util::metrics::init;

pub fn run(opts: ServerOpts) -> ExitCode {
    if opts.config_dir.exists() {
        tracing::info!("starting Gurp in server mode");

        let _provider = init::init_metrics(opts.metrics_to.as_deref(), "gurp.server")
            .unwrap_or_else(|e| {
                tracing::warn!("could not set up server metrics: {e}");
                None
            });

        match run_server(opts) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                tracing::error!("server error: {e}");
                ExitCode::FAILURE
            }
        }
    } else {
        tracing::error!("did not find config dir: {}", opts.config_dir);
        ExitCode::FAILURE
    }
}

fn run_server(opts: ServerOpts) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { http::start(opts).await })
}
