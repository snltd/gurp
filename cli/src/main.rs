use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use common::types::{ApplyOpts, ServerOpts};
use std::io::IsTerminal;
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
        /// Get config from a Gurp server
        #[arg(short = 's', long = "server")]
        server: Option<String>,
        /// Hostname to use when fetching config from server
        #[arg(short = 'H', long = "hostname", requires = "server")]
        hostname: Option<String>,
        /// Use a pre-compiled config, either Janet or JSON
        #[arg(short = 'p', long = "precompiled", conflicts_with = "server")]
        precompiled: bool,
        /// Specify a gurp Janet library, in preference to the built-in
        #[arg(short = 'L', long = "gurp-lib", conflicts_with = "server")]
        gurp_lib_path: Option<Utf8PathBuf>,
        /// Say what would happen, without actually doing it
        #[arg(short, long)]
        noop: bool,
        /// Dump intermediate config files to stdout
        #[arg(short = 'd', long, alias = "dump-configs")]
        dump_config: bool,
        /// When files change, dump diffs to stdout
        #[arg(short = 'D', long, alias = "dump-diff")]
        dump_diffs: bool,
        /// When dumping configs or diffs, use syntax colouring where supported
        #[arg(short = 'C', long)]
        colour: bool,
        /// When dumping configs, number lines
        #[arg(short = 'N', long)]
        line_no: bool,
        /// HTTP POST InfluxDB metrics to this host
        #[arg(short = 'M', long)]
        metrics_to: Option<String>,

        /// Host configuration file
        #[arg(required_unless_present = "server", conflicts_with = "server")]
        host_config_file: Option<Utf8PathBuf>,
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
        #[arg(short, long, required = true)]
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
    /// Run Gurp in Server mode
    Server {
        /// Where to find host configuration files
        #[arg(short, long, required = true)]
        config_dir: Utf8PathBuf,
        /// HTTP POST InfluxDB metrics to this host
        #[arg(short = 'M', long)]
        metrics_to: Option<String>,
    },
    /// Show Janet builtins
    Show {
        /// Thing to show: one of library, defaults
        #[arg(required = true)]
        thing: String,
    },
}

fn main() -> anyhow::Result<()> {
    let use_colour =
        std::io::stdout().is_terminal() && std::env::var_os("GURP_NO_COLOUR").is_none();

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
            dump_diffs,
            colour,
            line_no,
            gurp_lib_path,
            metrics_to,
            precompiled,
            server,
            hostname,
        } => {
            let opts = ApplyOpts {
                noop,
                dump_config,
                dump_diffs,
                colour,
                line_no,
                gurp_lib_path,
                metrics_to,
                precompiled,
                server,
                hostname,
                compile_only: false,
                server_name: None,
                client_name: None,
            };
            commands::apply::run(host_config_file.as_ref(), &opts)
        }
        Commands::Compile {
            gurp_lib_path,
            line_no,
            host_config_file,
            format,
        } => {
            // Compile is the first part of run's code path, so we'll fake the apply options
            let opts = ApplyOpts {
                line_no,
                gurp_lib_path,
                compile_only: true,
                ..Default::default()
            };
            commands::compile::run(&host_config_file, format.as_deref(), &opts)
        }
        Commands::Describe { resource } => commands::describe::run(&resource),
        Commands::Server {
            config_dir,
            metrics_to,
        } => commands::server::run(ServerOpts {
            config_dir,
            metrics_to,
        }),
        Commands::Show { thing } => commands::show::run(&thing),
    };

    tracing::debug!("exiting {}", exit_code);
    std::process::exit(exit_code as i32);
}
