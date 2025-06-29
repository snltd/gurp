#[cfg(test)]
use crate::common::types::Opts;
use camino::Utf8PathBuf;
use std::env::current_dir;

#[cfg(test)]
use std::fs;
// use std::path::Path;

#[allow(dead_code)]
pub fn fixture(file: &str) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(current_dir().unwrap())
        .unwrap()
        .join("tests")
        .join("resources")
        .join(file)
}

#[cfg(test)]
pub fn load_fixture(file: &str) -> String {
    fs::read_to_string(fixture(file)).unwrap_or_else(|_| panic!("Did not find {file}"))
}

#[cfg(test)]
pub fn defopts() -> Opts {
    Opts {
        debug: false,
        noop: false,
        no_colour: true,
    }
}

#[cfg(test)]
pub fn defopts_noop() -> Opts {
    Opts {
        debug: false,
        noop: true,
        no_colour: true,
    }
}

#[cfg(test)]
pub fn init_janet() {
    unsafe {
        janetrs::lowlevel::janet_init();
    }
}

#[cfg(test)]
use nix::unistd::{Group, User, getgid, getuid};

#[cfg(test)]
pub fn my_user() -> String {
    User::from_uid(getuid()).unwrap().unwrap().name
}

#[cfg(test)]
pub fn my_group() -> String {
    Group::from_gid(getgid()).unwrap().unwrap().name
}
