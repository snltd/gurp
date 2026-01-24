use camino::Utf8PathBuf;
use embed::helpers;
use std::{env, fs};

fn main() {
    let my_dir =
        Utf8PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cannot get CARGO_MANIFEST_DIR"));

    let repo_root = my_dir.parent().expect("cannot get repo_root");
    let janet_lib_path = repo_root.join("janet/src/build-docgen.janet");
    let client = helpers::gurp_client().expect("cannot make gurp client");

    let janet_instructions = indoc::formatdoc! { r#"
        (setdyn :running-embedded true)
        (setdyn :repo-root "{repo_root}")
        (setdyn *syspath* "{repo_root}/janet/src")
        {}
        (generate-all-docs)
        "#
        ,
        &fs::read_to_string(janet_lib_path).expect("cannot read build-docgen.janet")
    };

    client.run(janet_instructions).unwrap();
}
