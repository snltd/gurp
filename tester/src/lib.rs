use camino::Utf8PathBuf;
use common::types::ApplyOpts;
use janet_int::{helpers, reader};
use janetrs::{TaggedJanet, env::CFunOptions};
use nix::unistd::{Group, User, getgid, getuid};
use std::env::current_dir;
use std::fs;

pub fn cwd() -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(current_dir().unwrap()).unwrap()
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
    let dir = Utf8PathBuf::from_path_buf(current_dir().unwrap()).unwrap();
    let full_janet = reader::assemble(janet_defn, &dir, &defopts()).unwrap();
    let json_wrapped_host_config = format!("{full_janet}\n(encode (first (values {janet_defn})))");
    let mut client = helpers::janet_client();

    client.add_c_fn(CFunOptions::new(c"encode", helpers::encode_c));

    let ret = match client.run(json_wrapped_host_config) {
        Ok(janet) => janet,
        Err(e) => {
            println!("{janet_defn}");
            panic!("janet2json ERROR: {e}");
        }
    };

    match ret.unwrap() {
        TaggedJanet::String(str) => str.to_string(),
        other => panic!("no buffer from Janet: got {other}"),
    }
}
