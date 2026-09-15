use camino::Utf8PathBuf;
use std::env::current_dir;

pub fn cwd() -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(current_dir().unwrap()).unwrap()
}

// Private copy of fixture() because referring to the tester crate would make a circular dependency
pub fn fixture(file: &str) -> Utf8PathBuf {
    cwd().join("tests").join("resources").join(file)
}
