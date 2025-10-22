use common::types::ExitCode;
use common::types::ServerOpts;
use server::http;

pub fn run(opts: &ServerOpts) -> ExitCode {
    if !opts.config_dir.exists() {
        tracing::error!("did not find config dir: {}", opts.config_dir);
        return 1;
    }

    tracing::info!("starting Gurp in server mode");

    match run_server() {
        Ok(_) => 0,
        Err(e) => {
            tracing::error!("server error: {e}");
            1
        }
    }
}

fn run_server() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { http::start().await })
}
