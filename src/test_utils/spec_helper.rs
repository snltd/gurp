#[cfg(test)]
use crate::common::constants::GURP_LIB;
#[cfg(test)]
use crate::common::types::Opts;
#[cfg(test)]
use crate::utils::janet_helpers;
#[cfg(test)]
use crate::utils::janet_helpers::janet_client;
#[cfg(test)]
use crate::utils::reader;
use camino::Utf8PathBuf;
#[cfg(test)]
use janetrs::{TaggedJanet, env::CFunOptions};
use std::env::current_dir;

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
        dump_config: false,
        noop: false,
        colour: false,
        line_no: false,
    }
}

#[cfg(test)]
pub fn defopts_noop() -> Opts {
    Opts {
        dump_config: false,
        noop: true,
        colour: true,
        line_no: false,
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
    let json_wrapped_host_config = format!("{full_janet}\n(encode (first (values {janet_defn})))");
    // println!("{json_wrapped_host_config}");
    let mut client = janet_client();
    client.add_c_fn(CFunOptions::new(c"encode", janet_helpers::encode_c));
    let ret = match client.run(json_wrapped_host_config) {
        Ok(janet) => janet,
        Err(e) => panic!("janet2json ERROR: {e}"),
    };

    println!("{ret:?}");

    match ret.unwrap() {
        TaggedJanet::String(str) => str.to_string(),
        other => panic!("no buffer from Janet: got {other}"),
    }
}
