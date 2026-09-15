use common::types::ApplyVmOpts;
use embed::client;
use janetrs::TaggedJanet;
use nix::unistd::{Group, User, getgid, getuid};
use std::fs;

pub fn my_user() -> String {
    User::from_uid(getuid()).unwrap().unwrap().name
}

pub fn my_group() -> String {
    Group::from_gid(getgid()).unwrap().unwrap().name
}

pub fn janet2json(janet_defn: &str) -> String {
    let client = client::gurp(&ApplyVmOpts::default(), false)
        .expect("janet2json failed to create gurp client");
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

pub fn raw_example(relative_path: &str) -> String {
    let example_file = snltest::repo_root()
        .join("janet/examples")
        .join(relative_path);
    fs::read_to_string(&example_file)
        .unwrap_or_else(|_| panic!("cannot find Janet example: {}", example_file))
}

pub fn deserialized_example<T: serde::de::DeserializeOwned>(relative_path: &str) -> T {
    let example_code = raw_example(relative_path);
    let example_json = janet2json(&example_code);
    serde_json::from_str(&example_json)
        .unwrap_or_else(|e| panic!("could not deserialize json: {}\nError: {}", example_json, e))
}

#[macro_export]
macro_rules! propmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        std::collections::HashMap::from([
            $(($key.to_string(), $value.to_string()),)*
        ])
    }};
}
