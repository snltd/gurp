use std::collections::HashMap;

pub type VarMap = HashMap<String, String>;

pub struct Opts {
    pub debug: bool,
    pub noop: bool,
    pub verbose: bool,
}
