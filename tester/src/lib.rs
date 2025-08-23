use camino::Utf8PathBuf;
use common::helpers;
use common::types::ApplyOpts;
use janet_int::constants::GURP_LIB;
use janet_int::helpers as janet_helpers;
use janet_int::reader;
use janetrs::{TaggedJanet, env::CFunOptions};
use nix::unistd::{Group, User, getgid, getuid};
use std::env::current_dir;
use std::fs;

pub fn fixture(file: &str) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(current_dir().unwrap())
        .unwrap()
        .join("tests")
        .join("resources")
        .join(file)
}

pub fn load_fixture(file: &str) -> String {
    fs::read_to_string(fixture(file)).unwrap_or_else(|_| panic!("Did not find {file}"))
}

pub fn defopts() -> ApplyOpts {
    ApplyOpts {
        dump_config: false,
        noop: false,
        colour: false,
        line_no: false,
        gurp_lib_path: None,
        compile_only: false,
    }
}

pub fn defopts_noop() -> ApplyOpts {
    ApplyOpts {
        dump_config: false,
        noop: true,
        colour: true,
        line_no: false,
        gurp_lib_path: None,
        compile_only: false,
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
    let full_janet = reader::janet_conf("", &dir, GURP_LIB, None, &defopts()).unwrap();
    let json_wrapped_host_config = format!("{full_janet}\n(encode (first (values {janet_defn})))");
    let mut client = janet_helpers::janet_client();
    client.add_c_fn(CFunOptions::new(c"encode", janet_helpers::encode_c));
    let ret = match client.run(json_wrapped_host_config) {
        Ok(janet) => janet,
        Err(e) => {
            // println!(
            //     "{}",
            //     helpers::dump_config(
            //         &full_janet,
            //         "complete Janet",
            //         &ApplyOpts {
            //             noop: false,
            //             colour: false,
            //             line_no: true,
            //             dump_config: true,
            //             gurp_lib_path: None,
            //             compile_only: false,
            //         },
            //     )
            // );
            println!("{janet_defn}");
            panic!("janet2json ERROR: {e}");
        }
    };

    println!("{ret:?}");

    match ret.unwrap() {
        TaggedJanet::String(str) => str.to_string(),
        other => panic!("no buffer from Janet: got {other}"),
    }
}
