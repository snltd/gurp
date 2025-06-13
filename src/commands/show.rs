use crate::common::constants::GURP_DEFAULTS;
use crate::common::constants::GURP_LIB;
use crate::common::types::ExitCode;

pub fn run(thing: &str) -> ExitCode {
    match thing {
        "library" => show_library(),
        "defaults" => show_defaults(),
        _ => {
            eprintln!("That's not a thing I can show you");
            1
        }
    }
}

fn show_library() -> ExitCode {
    println!("{}", GURP_LIB);
    0
}

fn show_defaults() -> ExitCode {
    println!("{}", GURP_DEFAULTS);
    0
}
