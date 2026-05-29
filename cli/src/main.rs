use camino::Utf8PathBuf;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::generate;
use clap_complete::shells::{Bash, Fish, Zsh};
use common::types::{
    ApplyClientOpts, ApplyOpts, ApplyOutputOpts, ApplyVmOpts, CompileOpts, ServerOpts,
};
use std::io::IsTerminal;
use std::process::ExitCode;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[clap(version, about = "Gurp configures illumos systems", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Configure this host according to the supplied configuration
    Apply {
        /// Get config from a Gurp server
        #[arg(short = 's', long = "server")]
        server: Option<String>,
        /// Hostname to use when fetching config from server
        #[arg(short = 'H', long = "hostname", requires = "server")]
        hostname: Option<String>,
        /// Use a pre-compiled JSON config, which may be local or remote
        #[arg(short = 'p', long = "precompiled")]
        precompiled: bool,
        /// Use a local pre-compiled Janet jimage as config
        #[arg(short = 'i', long = "image", conflicts_with = "server")]
        image: bool,
        /// Say what would happen, without actually doing it
        #[arg(short, long)]
        noop: bool,
        /// Define a constant which can be accessed from config
        #[arg(short = 'D', long = "define")]
        define: Vec<String>,
        /// Dump intermediate config files to stdout
        #[arg(long, alias = "dump-config")]
        dump_configs: bool,
        /// When files change, dump diffs to stdout
        #[arg(long, alias = "dump-diff")]
        dump_diffs: bool,
        /// When dumping configs or diffs, use syntax colouring where supported
        #[arg(short = 'C', long, alias = "color")]
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
        /// Execute a literal snippet of Janet config
        #[arg(short = 'e', long, alias = "execute")]
        exec: Option<String>,
        /// Do not check for or use a lockfile
        #[arg(long = "no-lock")]
        no_lock: bool,
        /// Run remove actions BEFORE ensure actions
        #[arg(long = "remove-first")]
        remove_first: bool,
        /// Only apply resources whose IDs match this regex
        #[arg(long = "only")]
        only: Option<String>,
        /// Host configuration file
        #[arg(
            required_unless_present = "server",
            conflicts_with = "server",
            required_unless_present = "exec",
            conflicts_with = "exec"
        )]
        host_config_file: Option<Utf8PathBuf>,
    },
    /// Compile a Janet host description
    Compile {
        /// When displaying compiled config, number lines
        #[arg(short = 'N', long)]
        line_no: bool,
        /// When displaying compile Janet, use syntax colouring
        #[arg(short = 'C', long)]
        colour: bool,
        /// Output in the given format: 'jimage', 'janet', or 'json'
        #[arg(short, long, required = true, default_value = "json")]
        format: String,
        /// Output file for compiled config (required for jimage, optional for others)
        #[arg(short = 'o', long = "output-file")]
        output_file: Option<Utf8PathBuf>,
        /// Host configuration file
        #[arg(required = true)]
        host_config_file: Utf8PathBuf,
    },
    /// Generate shell completions
    Completions {
        #[arg(required = true)]
        shell: String,
    },
    /// Describe a resource type
    Describe {
        /// Do not use any ANSI colouring
        #[arg(short = 'C', long, alias = "no-color")]
        no_colour: bool,
        /// Resource type you wish to see described
        #[arg(required = true)]
        resource: String,
    },
    /// List the doers in this version of Gurp
    Doers {
        /// Do not use any ANSI colouring
        #[arg(short = 'C', long, alias = "no-color")]
        no_colour: bool,
    },
    /// Open a Janet REPL with the Gurp library already loaded into the root environment
    Repl {
        /// Define a constant which can be accessed from the REPL
        #[arg(short = 'D', long = "define")]
        define: Vec<String>,
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
}

fn main() -> ExitCode {
    let use_colour =
        std::io::stdout().is_terminal() && std::env::var_os("GURP_NO_COLOUR").is_none();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_ansi(use_colour)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Apply {
            host_config_file,
            noop,
            dump_configs,
            dump_diffs,
            colour,
            line_no,
            metrics_to,
            precompiled,
            server,
            hostname,
            exec,
            destroy_everything_you_touch,
            image,
            no_lock,
            remove_first,
            only,
            define,
        } => {
            let opts = ApplyOpts {
                noop,
                metrics_to,
                precompiled,
                exec,
                destroy: destroy_everything_you_touch,
                image,
                no_lock,
                remove_first,
                only,
                output: ApplyOutputOpts {
                    colour,
                    line_no,
                    dump_configs,
                    dump_diffs,
                },
                vm: ApplyVmOpts { define },
                client: ApplyClientOpts { server, hostname },
            };
            commands::apply::init::run(host_config_file.as_deref(), &opts)
        }
        Commands::Compile {
            line_no,
            host_config_file,
            format,
            output_file,
            colour,
        } => {
            let opts = CompileOpts {
                line_no,
                format,
                output_file,
                colour,
            };

            commands::compile::run(&host_config_file, &opts)
        }
        Commands::Completions { shell } => {
            match shell.as_str() {
                "bash" => {
                    generate(Bash, &mut Cli::command(), "gurp", &mut std::io::stdout());
                }
                "fish" => {
                    generate(Fish, &mut Cli::command(), "gurp", &mut std::io::stdout());
                }
                "zsh" => {
                    generate(Zsh, &mut Cli::command(), "gurp", &mut std::io::stdout());
                }
                _ => {
                    eprintln!("unsupported shell");
                    return ExitCode::FAILURE;
                }
            }
            ExitCode::SUCCESS
        }
        Commands::Describe {
            resource,
            no_colour,
        } => commands::describe::run(&resource, no_colour),
        Commands::Doers { no_colour } => commands::doers::run(no_colour),
        Commands::Repl { define } => {
            let opts = ApplyVmOpts { define };
            commands::repl::run(&opts)
        }
        Commands::Server {
            config_dir,
            metrics_to,
        } => commands::server::init::run(ServerOpts {
            config_dir,
            metrics_to,
        }),
    }
}
