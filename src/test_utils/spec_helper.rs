#[cfg(test)]
use crate::common::types::Opts;
use camino::Utf8PathBuf;
use std::env::current_dir;

#[cfg(test)]
use crate::common::constants::{GURP_LIB, JSON_LIB};
#[cfg(test)]
use crate::utils::janet_helpers::janet_client;
#[cfg(test)]
use crate::utils::reader;
#[cfg(test)]
use janetrs::TaggedJanet;

#[cfg(test)]
use std::fs;

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
use nix::unistd::{Group, User, getgid, getuid};

#[cfg(test)]
pub fn my_user() -> String {
    User::from_uid(getuid()).unwrap().unwrap().name
}

#[cfg(test)]
pub fn my_group() -> String {
    Group::from_gid(getgid()).unwrap().unwrap().name
}

#[cfg(test)]
pub fn janet2json(janet_defn: &str) -> String {
    let dir = Utf8PathBuf::from_path_buf(current_dir().unwrap()).unwrap();
    let full_janet = reader::janet_conf("", &dir, GURP_LIB, &defopts(), false).unwrap();
    let json_wrapped_host_config =
        format!("{JSON_LIB}\n{full_janet}\n(encode (first (values {janet_defn})))");
    println!("{json_wrapped_host_config}");
    let client = janet_client();
    let ret = match client.run(json_wrapped_host_config) {
        Ok(janet) => janet,
        Err(e) => panic!("janet2json ERROR: {e}"),
    };

    println!("{:?}", ret);

    match ret.unwrap() {
        TaggedJanet::Buffer(str) => str.to_string(),
        _ => panic!("no buffer from Janet"),
    }
}
