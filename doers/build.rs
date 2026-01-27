use camino::Utf8PathBuf;
use embed::helpers;
use indoc::formatdoc;
use janetrs::TaggedJanet;
use janetrs::client::JanetClient;
use std::{env, fs};

fn compile_markdown_image(client: &JanetClient) {
    let my_dir =
        Utf8PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cannot get CARGO_MANIFEST_DIR"));
    // let client = JanetClient::init_with_default_env().expect("Failed to create Janet client");
    let repo_root = my_dir.parent().expect("cannot get repo_root");
    let src_file = repo_root.join("janet/src/doer-docs/markdown-docs.janet");
    let jimage_path = repo_root.join("janet/lib/markdown-docs.jimage");

    let janet_instructions = formatdoc! { r#"
        (def build-env (make-env (fiber/getenv (fiber/root))))
        (merge-module build-env (dofile "{src_file}" :env build-env) "" true)
        (make-image build-env)
        "#
    };

    let result = client
        .run(&janet_instructions)
        .expect("Failed to compile Janet library");

    let image_bytes = match result.unwrap() {
        TaggedJanet::Buffer(buf) => buf.as_bytes().to_vec(),
        _ => panic!("expected buffer from make-image"),
    };

    if image_bytes.len() < 10000 {
        panic!("library image is suspiciously small");
    }

    fs::write(&jimage_path, &image_bytes).expect("Failed to write jimage file");
}

use janetrs::{Janet, JanetString};

fn main() {
    let my_dir =
        Utf8PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cannot get CARGO_MANIFEST_DIR"));

    // let repo_root = my_dir.parent().expect("cannot get repo_root");
    // let janet_lib_path = repo_root.join("janet/src/doer-docs/markdown-docs.janet");
    let client = helpers::gurp_client().expect("cannot make gurp client");
    // let markdown_image = compile_markdown_image(&client);
    // let markdown_image_as_string = JanetString::new(markdown_image);
    // let markdown_image_as_jstring = Janet::string(markdown_image_as_string);
    // print!("repo_root is {repo_root}");

    let repo_root = my_dir.parent().expect("cannot get repo_root");
    let src_file = repo_root.join("janet/src/doer-docs/markdown-docs.janet");
    let jimage_path = repo_root.join("janet/lib/markdown-docs.jimage");

    let janet_instructions = format!(
        "(setdyn :repo-root \"{repo_root}\") (merge-module (fiber/getenv (fiber/root)) (load-image (slurp \"{jimage_path}\"))) (generate-all-docs)"
    );
    // let janet_instructions = indoc::formatdoc! { r#"
    //     (setdyn :running-embedded true)
    //     (setdyn :repo-root "{repo_root}")
    //     (setdyn *syspath* "{repo_root}/janet")
    //     {}
    //     (generate-all-docs)
    //     "#
    //     ,
    //     &fs::read_to_string(janet_lib_path).expect("cannot read {janet_lib_path}")
    // };

    client.run(janet_instructions).unwrap();
}
