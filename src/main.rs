mod doers;
mod utils;
use crate::doers::host;
use crate::utils::janet_runner;
use crate::utils::types::Opts;
use camino::Utf8PathBuf;
use clap::Parser;
use janetrs::client::Error;

#[derive(Parser)]
#[clap(version, about = "Configures hosts, or might do one day", long_about = None)]
struct Cli {
    /// Be verbose
    #[arg(short, long, global = true)]
    pub verbose: bool,
    /// Be very verbose
    #[arg(short, long, global = true)]
    pub debug: bool,
    /// Say what would happen, without actually doing it
    #[arg(short, long, global = true)]
    pub noop: bool,
    /// One or more hostfiles
    #[arg(required = true)]
    files: Vec<Utf8PathBuf>,
}

fn configure_host(host_file_path: &Utf8PathBuf, opts: &Opts) -> anyhow::Result<bool> {
    let janet_host_config = std::fs::read_to_string(host_file_path)?;
    let mut client = janet_runner::janet_client();
    let host_config = host::define_host_config(&mut client, janet_host_config.as_str())?;
    host::configure(&host_config, opts)?;
    Ok(true)
}

fn main() -> Result<(), Error> {
    let mut exit_code = 0;
    let cli = Cli::parse();
    let opts = Opts {
        debug: cli.debug,
        noop: cli.noop,
        verbose: cli.verbose,
    };

    for host_file in cli.files {
        if let Err(e) = configure_host(&host_file, &opts) {
            eprintln!("Error configuring host: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}
