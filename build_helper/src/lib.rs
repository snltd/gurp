use blake3::Hash;
use camino::Utf8PathBuf;
use janetrs::TaggedJanet;
use janetrs::client::JanetClient;
use std::{env, fs};

// This is for build.rs files. Anything that fails can fail hard.

pub struct ImageHelper {
    client: JanetClient,
    repo_root: Utf8PathBuf,
    src_files: Vec<Utf8PathBuf>,
    img_file: Utf8PathBuf,
}

impl ImageHelper {
    pub fn new(src_files: Vec<&str>, img_name: &str) -> Self {
        let repo_root = ImageHelper::repo_root();

        Self {
            client: ImageHelper::client(),
            src_files: src_files
                .iter()
                .map(|f| repo_root.join("janet").join("src").join(f).to_owned())
                .collect(),
            img_file: repo_root.join("janet").join("lib").join(img_name),
            repo_root,
        }
    }

    pub fn compile_to_file(&self) -> &Self {
        let mut janet_instructions =
            "(def build-env (make-env (fiber/getenv (fiber/root))))\n".to_owned();

        for f in &self.src_files {
            janet_instructions.push_str(&format!(
                r#"(merge-module build-env (dofile "{f}" :env build-env) "" true)"#
            ));
            janet_instructions.push('\n');
        }

        janet_instructions.push_str("(make-image build-env)");

        let result = self
            .client
            .run(&janet_instructions)
            .expect("error running Janet");

        let image_bytes = match result.unwrap() {
            TaggedJanet::Buffer(buf) => buf.as_bytes().to_vec(),
            _ => panic!("expected buffer from make-image"),
        };

        if image_bytes.len() < 10000 {
            panic!("library image is suspiciously small");
        }

        if self.img_file.exists() && self.image_hash() == blake3::hash(&image_bytes) {
            println!("NO CHANGE");
        } else {
            println!("CHANGE");
            fs::write(&self.img_file, &image_bytes)
                .unwrap_or_else(|_| panic!("Failed to write jimage file {}", self.img_file));
        }

        self
    }

    pub fn call_with_image(&self, janet: &str) {
        self.client
            .run(indoc::formatdoc! { r#"
                (setdyn :repo-root "{}")
                (merge-module (fiber/getenv (fiber/root)) (load-image (slurp "{}")))
                (generate-all-docs)
                "{janet}"
            "#
            , self.repo_root, self.img_file
            })
            .unwrap_or_else(|_| panic!("error calling {janet}"));
    }

    fn client() -> JanetClient {
        JanetClient::init_with_default_env().expect("Failed to create Janet client")
    }

    fn repo_root() -> Utf8PathBuf {
        Utf8PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cannot get CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cannot get repo_root")
            .into()
    }

    fn image_hash(&self) -> Hash {
        let mut hasher = blake3::Hasher::new();
        let mut fh = fs::File::open(&self.img_file).expect("cannot read existing lib file");
        std::io::copy(&mut fh, &mut hasher).expect("cannot read existing lib file into buffer");
        hasher.finalize()
    }
}
