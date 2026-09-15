use camino::Utf8Path;
use common::types::ApplyVmOpts;
use embed::runner;
use std::process::ExitCode;

pub fn run(opts: &ApplyVmOpts, syspath: &Utf8Path, gurp_config_root: &Utf8Path) -> ExitCode {
    let c_syspath = match syspath.canonicalize_utf8() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("cannot canonicalize {syspath}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let c_gurp_config_root = match gurp_config_root.canonicalize_utf8() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("cannot canonicalize {gurp_config_root}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let repl_janet = indoc::formatdoc! { r#"
        (setdyn *syspath* "{c_syspath}")
        (setdyn :gurp-config-root "{c_gurp_config_root}")
        (repl)
    "# };

    runner::run_command_and_exit(&repl_janet, opts)
}
