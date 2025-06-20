#[cfg(test)]
use crate::common::types::ApplyContext;
#[cfg(test)]
use crate::common::types::Opts;
use camino::Utf8PathBuf;
#[cfg(test)]
use std::collections::HashSet;
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
    fs::read_to_string(fixture(file)).unwrap_or_else(|_| panic!("Did not find {}", file))
}

#[cfg(test)]
pub fn defopts() -> Opts {
    Opts {
        debug: false,
        noop: false,
        verbose: false,
        no_colour: true,
    }
}

#[cfg(test)]
pub fn defopts_noop() -> Opts {
    Opts {
        debug: false,
        noop: true,
        verbose: false,
        no_colour: true,
    }
}

#[cfg(test)]
pub fn defcontext() -> ApplyContext {
    ApplyContext {
        changed_ids: HashSet::new(),
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

// #[cfg(test)]
// use assert_fs::TempDir;
// #[cfg(test)]
// use camino::Utf8Path;

// #[allow(dead_code)]
// #[cfg(test)]
// pub trait TempDirExt {
//     fn utf8_path(&self) -> &Utf8Path;
// }

// #[cfg(test)]
// impl TempDirExt for TempDir {
//     fn utf8_path(&self) -> &Utf8Path {
//         Utf8Path::from_path(self.path()).unwrap()
//     }
// }

// #[allow(dead_code)]
// pub fn files_in_dir(dir: &Path) -> usize {
//     let files: Vec<_> = dir.read_dir().unwrap().collect();
//     files.len()
// }
