use common::types::ExitCode;
use janet_int::constants::{GURP_DEFAULTS, GURP_LIB};

pub fn run(thing: &str) -> ExitCode {
    match thing {
        "library" => show_library(),
        "defaults" => show_defaults(),
        other => {
            tracing::error!("{} is not a thing I can show you", other);
            1
        }
    }
}

fn show_library() -> ExitCode {
    println!("{GURP_LIB}");
    0
}

fn show_defaults() -> ExitCode {
    println!("{GURP_DEFAULTS}");
    0
}
