mod commands;
mod common;
mod doers;
mod test_utils;
mod utils;

use crate::common::types::Opts;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

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
    /// Don't colour things which can be coloured
    #[arg(short = 'N', long, global = true)]
    no_colour: bool,
    #[command(subcommand)]
    command: Commands,
} // might not need the global. Will there be subcommands?

#[derive(Debug, Subcommand)]
enum Commands {
    /// Configure the host with the supplied configuration
    Apply {
        /// Specify a gurp Janet library, in preference to the built-in
        #[arg(short = 'L', long = "gurp-lib", global = true)]
        gurp_lib_path: Option<Utf8PathBuf>,

        /// Host configuration file
        #[arg(required = true)]
        host_config_file: Utf8PathBuf,
    },
    /// Compile the Janet description and dump it to stdout
    Compile {
        /// Specify a gurp Janet library, in preference to the built-in
        #[arg(short = 'L', long = "gurp-lib", global = true)]
        gurp_lib_path: Option<Utf8PathBuf>,

        /// Host configuration file
        #[arg(required = true)]
        host_config_file: Utf8PathBuf,
    },
    /// Show Janet builtins
    Show {
        /// Thing to show: one of library, defaults
        #[arg(required = true)]
        thing: String,
    },
}

fn main() -> anyhow::Result<()> {
    let use_colour = std::env::var_os("GURP_NO_COLOUR").is_none();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(use_colour)
        .init();
    let cli = Cli::parse();

    let global_opts = Opts {
        debug: cli.debug,
        noop: cli.noop,
        verbose: cli.verbose,
        no_colour: cli.no_colour,
    };

    let exit_code = match cli.command {
        Commands::Apply {
            gurp_lib_path,
            host_config_file,
        } => commands::apply::run(&host_config_file, &gurp_lib_path, &global_opts),
        Commands::Compile {
            gurp_lib_path,
            host_config_file,
        } => commands::compile::run(&host_config_file, &gurp_lib_path, &global_opts),
        Commands::Show { thing } => commands::show::run(&thing),
    };

    tracing::debug!("exiting {}", exit_code);
    std::process::exit(exit_code as i32);
}
