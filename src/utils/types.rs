use camino::Utf8PathBuf;

#[derive(Clone)]
pub struct Opts {
    pub debug: bool,
    pub noop: bool,
    pub verbose: bool,
    pub gurp_lib_path: Option<Utf8PathBuf>,
}
