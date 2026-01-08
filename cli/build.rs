// I don't want Janet to be a build dependency, so this compiles a Gurp library image file from
// source.
//
use blake3::Hash;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use janetrs::TaggedJanet;
use janetrs::client::JanetClient;
use std::env;
use std::fs;

fn hash_of_file(path: &Utf8PathBuf) -> Hash {
    let mut hasher = blake3::Hasher::new();
    let mut fh = fs::File::open(path).expect("cannot read existing lib file");
    std::io::copy(&mut fh, &mut hasher).expect("cannot read existing lib file into buffer");
    hasher.finalize()
}

fn main() {
    let crate_dir =
        Utf8PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cannot get CARGO_MANIFEST_DIR"));

    let top_level = crate_dir.parent().expect("cannot get top-level directory");
    let lib_dir = top_level.join("janet/lib");
    let lib_file = lib_dir.join("gurp.janet");
    let jimage_path = lib_dir.join("gurp.jimage");

    env::set_current_dir(&lib_dir).expect("Failed to change to janet/lib directory");

    let client = JanetClient::init_with_default_env().expect("Failed to create Janet client");

    let janet_instructions = formatdoc! { r#"
        (def build-env (make-env (fiber/getenv (fiber/root))))
        (merge-module build-env (dofile "{lib_file}" :env build-env) "" true)
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

    if jimage_path.exists() && hash_of_file(&jimage_path) == blake3::hash(&image_bytes) {
        println!("NO CHANGE");
    } else {
        println!("CHANGE");
        fs::write(&jimage_path, &image_bytes).expect("Failed to write jimage file");
    }

    // println!("cargo:rerun-if-changed=janet/lib/gurp.jimage");
}
