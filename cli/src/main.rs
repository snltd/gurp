use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use common::types::ApplyOpts;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[clap(version, about = "gurp configures illumos systems", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Configure the host with the supplied configuration
    Apply {
        /// Specify a gurp Janet library, in preference to the built-in
        #[arg(short = 'L', long = "gurp-lib")]
        gurp_lib_path: Option<Utf8PathBuf>,
        /// Say what would happen, without actually doing it
        #[arg(short, long)]
        noop: bool,
        /// Dump intermediate config files to stdout
        #[arg(short, long)]
        dump_config: bool,
        /// When dumping configs, use syntax colouring where possible
        #[arg(short = 'C', long)]
        colour: bool,
        /// When dumping configs, number lines
        #[arg(short = 'N', long)]
        line_no: bool,
        /// HTTP POST InfluxDB metrics to this host
        #[arg(short = 'M', long)]
        metrics_to: Option<String>,

        /// Host configuration file
        #[arg(required = true)]
        host_config_file: Utf8PathBuf,
    },
    /// Compile the Janet description, and optionally write it to stdout
    Compile {
        /// Specify a gurp Janet library, in preference to the built-in
        #[arg(short = 'L', long = "gurp-lib")]
        gurp_lib_path: Option<Utf8PathBuf>,
        /// When displaying compiled config, number lines
        #[arg(short = 'N', long)]
        line_no: bool,

        /// Output in the given format: 'janet' or 'json'
        #[arg(short, long)]
        format: Option<String>,
        /// Host configuration file
        #[arg(required = true)]
        host_config_file: Utf8PathBuf,
    },
    /// Describe a resource type
    Describe {
        /// Resource type you wish to see described
        #[arg(required = true)]
        resource: String,
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
            metrics_to,
        } => {
            let opts = ApplyOpts {
                noop,
                dump_config,
                colour,
                line_no,
                gurp_lib_path,
                compile_only: false,
                metrics_to,
            };
            commands::apply::run(&host_config_file, &opts)
        }
        Commands::Compile {
            gurp_lib_path,
            line_no,
            host_config_file,
            format,
        } => {
            // Compile is the first part of run's code path, so we'll fake the apply options
            let opts = ApplyOpts {
                noop: false,
                dump_config: false,
                colour: false,
                line_no,
                gurp_lib_path,
                compile_only: true,
                metrics_to: None,
            };
            commands::compile::run(&host_config_file, format.as_deref(), &opts)
        }
        Commands::Describe { resource } => commands::describe::run(&resource),
        Commands::Show { thing } => commands::show::run(&thing),
    };

    tracing::debug!("exiting {}", exit_code);
    std::process::exit(exit_code as i32);
}
