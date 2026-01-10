use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use common::types::{ApplyOpts, CompileOpts, ServerOpts};
use std::io::IsTerminal;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[clap(version, about = "Gurp configures illumos systems", long_about = None)]
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
        /// When getting server configuration, request it in JSON format, compiled on the server
        #[arg(short = 'J', long = "as-json", requires = "server")]
        as_json: bool,
        /// Hostname to use when fetching config from server
        #[arg(short = 'H', long = "hostname", requires = "server")]
        hostname: Option<String>,
        /// Use a pre-compiled JSON config
        #[arg(short = 'p', long = "precompiled", conflicts_with = "server")]
        precompiled: bool,
        /// Use a local pre-compiled Janet jimage as config
        #[arg(short = 'i', long = "image", conflicts_with = "server")]
        image: bool,
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
        /// Turn all ensures into removes. Use with extreme caution
        #[arg(long)]
        destroy_everything_you_touch: bool,
        /// Host configuration file
        #[arg(required_unless_present = "server", conflicts_with = "server")]
        host_config_file: Option<Utf8PathBuf>,
    },
    /// Compile the Janet description, and optionally write it to stdout
    Compile {
        /// When displaying compiled config, number lines
        #[arg(short = 'N', long)]
        line_no: bool,
        /// Dump intermediate config files to stdout
        #[arg(short = 'd', long, alias = "dump-configs")]
        dump_config: bool,
        /// Output in the given format: 'janet', 'jimage', or 'json'
        #[arg(short, long, required = true, default_value = "json")]
        format: String,
        /// Output file for compiled config (required for jimage, optional for others)
        #[arg(short = 'o', long = "output")]
        output_file: Option<Utf8PathBuf>,
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
    /// List the doers in this version of Gurp
    Doers {},
    /// Open a Janet REPL with the Gurp library already loaded into the root environment
    Repl {},
    /// Run Gurp in Server mode
    Server {
        /// Where to find host configuration files
        #[arg(short, long, required = true)]
        config_dir: Utf8PathBuf,
        /// HTTP POST InfluxDB metrics to this host
        #[arg(short = 'M', long)]
        metrics_to: Option<String>,
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
            metrics_to,
            precompiled,
            server,
            hostname,
            destroy_everything_you_touch,
            image,
            as_json,
        } => {
            let opts = ApplyOpts {
                noop,
                dump_config,
                dump_diffs,
                colour,
                line_no,
                metrics_to,
                precompiled,
                server,
                hostname,
                compile_only: false,
                server_name: None,
                client_name: None,
                destroy: destroy_everything_you_touch,
                image,
                as_json,
            };
            commands::apply::run(host_config_file.as_ref(), &opts)
        }
        Commands::Compile {
            line_no,
            host_config_file,
            format,
            output_file,
            dump_config,
        } => {
            // Compile is the first part of run's code path, so we'll fake the apply options
            let apply_opts = ApplyOpts {
                line_no,
                compile_only: true,
                dump_config,
                ..Default::default()
            };

            let compile_opts = CompileOpts {
                format,
                output_file,
            };

            commands::compile::run(&host_config_file, &compile_opts, &apply_opts)
        }
        Commands::Describe { resource } => commands::describe::run(&resource),
        Commands::Doers {} => commands::doers::run(),
        Commands::Repl {} => commands::repl::run(),
        Commands::Server {
            config_dir,
            metrics_to,
        } => commands::server::run(ServerOpts {
            config_dir,
            metrics_to,
        }),
    };

    tracing::debug!("exiting {}", exit_code);
    std::process::exit(exit_code as i32);
}
