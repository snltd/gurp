#[cfg(test)]
use crate::utils::types::Opts;
use camino::Utf8PathBuf;
use std::env::current_dir;
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
pub fn defopts() -> Opts {
    Opts {
        debug: false,
        noop: false,
        verbose: false,
    }
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
