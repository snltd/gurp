use camino::Utf8PathBuf;
use common::types::ApplyOpts;
use embed::helpers;
use janetrs::TaggedJanet;
use nix::unistd::{Group, User, getgid, getuid};
use std::env;
use std::fs;

pub fn cwd() -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(env::current_dir().unwrap()).unwrap()
}

pub fn fixture(file: &str) -> Utf8PathBuf {
    cwd().join("tests").join("resources").join(file)
}

pub fn load_fixture(file: &str) -> String {
    let fixture = fixture(file);
    fs::read_to_string(&fixture).unwrap_or_else(|_| panic!("Did not find {fixture}"))
}

pub fn defopts() -> ApplyOpts {
    ApplyOpts::default()
}

pub fn defopts_noop() -> ApplyOpts {
    ApplyOpts {
        noop: true,
        ..Default::default()
    }
}

pub fn my_user() -> String {
    User::from_uid(getuid()).unwrap().unwrap().name
}

pub fn my_group() -> String {
    Group::from_gid(getgid()).unwrap().unwrap().name
}

pub fn janet2json(janet_defn: &str) -> String {
    let client = helpers::gurp_client().expect("janet2json failed to create gurp client");
    let janet_instructions = format!("(to-json {janet_defn})");

    let ret = match client.run(&janet_instructions) {
        Ok(janet) => janet,
        Err(e) => {
            eprintln!("-- ERROR CAUSED BY ------------------------------------------");
            eprintln!("{janet_instructions}");
            eprintln!("-------------------------------------------------------------");
            panic!("janet2json ERROR: {e}");
        }
    };

    match ret.unwrap() {
        TaggedJanet::String(str) => str.to_string(),
        other => panic!("no buffer from Janet: got {other}"),
    }
}

pub fn deserialized_example<T: serde::de::DeserializeOwned>(relative_path: &str) -> T {
    let example_file = repo_root().join("janet/examples").join(relative_path);
    let example_code = fs::read_to_string(&example_file)
        .unwrap_or_else(|_| panic!("cannot find Janet example: {}", example_file));

    let example_json = janet2json(&example_code);
    serde_json::from_str(&example_json)
        .unwrap_or_else(|_| panic!("could not deserialize json: {}", example_json))
}

fn repo_root() -> Utf8PathBuf {
    Utf8PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cannot get CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cannot get repo_root")
        .into()
}

#[macro_export]
macro_rules! propmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        std::collections::HashMap::from([
            $(($key.to_string(), $value.to_string()),)*
        ])
    }};
}
