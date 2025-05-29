mod doers;
mod test_utils;
mod utils;
use crate::utils::types::Opts;
use camino::Utf8PathBuf;
use clap::Parser;

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
    /// Specify a gurp Janet library, in preference to the built-in
    #[arg(short = 'L', long = "gurp-lib", global = true)]
    gurp_lib_path: Option<Utf8PathBuf>,
    /// One or more hostfiles
    #[arg(required = true)]
    files: Vec<Utf8PathBuf>,
} // might not need the global. Will there be subcommands?

fn main() -> anyhow::Result<()> {
    let mut exit_code = 0;
    let cli = Cli::parse();

    let opts = Opts {
        debug: cli.debug,
        noop: cli.noop,
        verbose: cli.verbose,
        gurp_lib_path: cli.gurp_lib_path,
    };

    for host_file in cli.files {
        match doers::host::do_it(&host_file, &opts) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("ERROR: {}", e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}
