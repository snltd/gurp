use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use common::types::ApplyOpts;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[clap(version, about = "Configures hosts, or might do one day", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Configure the host with the supplied configuration
    Apply {
        /// Specify a gurp Janet library, in preference to the built-in
        #[arg(short = 'L', long = "gurp-lib", global = true)]
        gurp_lib_path: Option<Utf8PathBuf>,
        /// Say what would happen, without actually doing it
        #[arg(short, long, global = true)]
        noop: bool,
        /// Dump intermediate config files to stdout
        #[arg(short, long, global = true)]
        dump_config: bool,
        /// When dumping configs, use syntax colouring where possible
        #[arg(short = 'C', long, global = true)]
        colour: bool,
        /// When dumping configs, number lines
        #[arg(short = 'N', long, global = true)]
        line_no: bool,

        /// Host configuration file
        #[arg(required = true)]
        host_config_file: Utf8PathBuf,
    },
    /// Compile the Janet description and dump it to stdout
    Compile {
        /// Specify a gurp Janet library, in preference to the built-in
        #[arg(short = 'L', long = "gurp-lib", global = true)]
        gurp_lib_path: Option<Utf8PathBuf>,
        /// When displaying compiled config, number lines
        #[arg(short = 'N', long, global = true)]
        line_no: bool,

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
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_ansi(use_colour)
        .init();

    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Apply {
            host_config_file,
            noop,
            dump_config,
            colour,
            line_no,
            gurp_lib_path,
        } => {
            let opts = ApplyOpts {
                noop,
                dump_config,
                colour,
                line_no,
                gurp_lib_path,
                compile_only: false,
            };
            commands::apply::run(&host_config_file, &opts)
        }
        Commands::Compile {
            gurp_lib_path,
            line_no,
            host_config_file,
        } => {
            // Compile is the first part of run's code path, so we'll fake the apply options
            let opts = ApplyOpts {
                noop: false,
                dump_config: false,
                colour: false,
                line_no,
                gurp_lib_path,
                compile_only: true,
            };
            commands::compile::run(&host_config_file, &opts)
        }
        Commands::Show { thing } => commands::show::run(&thing),
    };

    tracing::debug!("exiting {}", exit_code);
    std::process::exit(exit_code as i32);
}
