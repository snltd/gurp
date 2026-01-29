use crate::server;
use common::types::{ExitCode, ServerOpts};

pub fn run(opts: ServerOpts) -> ExitCode {
    if !opts.config_dir.exists() {
        tracing::error!("did not find config dir: {}", opts.config_dir);
        return 1;
    }

    tracing::info!("starting Gurp in server mode");

    match server::init::run_server(opts) {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("server error: {e}");
            1
        }
    }
}

fn run_server(opts: ServerOpts) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { server::http::start(opts).await })
}
