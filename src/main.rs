mod doers;
mod utils;
use crate::doers::host;
use crate::utils::janet_runner;
use crate::utils::types::Opts;
use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Parser;
use janetrs::client::Error;

#[derive(Parser)]
#[clap(version, about = "Configures hosts, or might do one day", long_about = None)]
struct Cli {
    /// Be verbose
    #[arg(short, long, global = true)]
    verbose: bool,
    /// Be very verbose
    #[arg(short, long, global = true)]
    debug: bool,
    /// Say what would happen, without actually doing it
    #[arg(short, long, global = true)]
    noop: bool,
    /// :-separated list of directories which may house module files
    #[arg(short = 'M', long, global = true)]
    module_dirs: Option<String>,
    /// One or more hostfiles
    #[arg(required = true)]
    files: Vec<Utf8PathBuf>,
} // might not need the global. Will there be subcommands?

fn prep_host_config(host_file_path: &Utf8PathBuf, _opts: &Opts) -> anyhow::Result<String> {
    let janet_host_config = std::fs::read_to_string(host_file_path)?;
    let qualified_path = host_file_path.canonicalize_utf8()?;
    let host_config_dir = qualified_path.parent().context("cannot find parent")?;
    Ok(format!(
        "{}\n{}",
        format!("(setdyn *syspath* \"{}\")", host_config_dir),
        janet_host_config
    ))
}

fn execute_host_config(janet_host_config: String, opts: &Opts) -> anyhow::Result<bool> {
    let mut client = janet_runner::janet_client();
    host::configure(janet_host_config, &mut client, opts)?;
    Ok(true)
}

fn main() -> Result<(), Error> {
    let mut exit_code = 0;
    let cli = Cli::parse();

    let opts = Opts {
        module_dirs: cli.module_dirs,
        debug: cli.debug,
        noop: cli.noop,
        verbose: cli.verbose,
    };

    for host_file in cli.files {
        let host_config = match prep_host_config(&host_file, &opts) {
            Ok(conf) => conf,
            Err(e) => {
                eprintln!("Error prepping host config: {}", e);
                exit_code = 1;
                continue;
            }
        };

        if let Err(e) = execute_host_config(host_config, &opts) {
            eprintln!("Error configuring host: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}
