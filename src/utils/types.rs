// use std::collections::HashMap;

// pub type VarMap = HashMap<String, String>;

#[derive(Clone)]
pub struct Opts {
    pub module_dirs: Option<String>,
    pub debug: bool,
    pub noop: bool,
    pub verbose: bool,
}
