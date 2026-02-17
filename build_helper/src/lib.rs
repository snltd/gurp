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
        let src_dir = repo_root.join("janet").join("src");

        for entry in walkdir::WalkDir::new(&src_dir) {
            let entry = entry.unwrap();
            if entry.path().extension().is_some_and(|e| e == "janet") {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
        }

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
        // Check if we need to rebuild
        if self.img_file.exists() {
            // let current_hash = self.image_hash();
            let source_hash = self.source_hash();

            // Store the source hash we built from
            let hash_file = self.img_file.with_extension("jimage.hash");
            if hash_file.exists() {
                let stored_hash: Hash = fs::read_to_string(&hash_file)
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| Hash::from([0u8; 32]));

                if stored_hash == source_hash {
                    return self;
                }
            }
        }

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

        let hash_file = self.img_file.with_extension("jimage.hash");
        fs::write(&hash_file, self.source_hash().to_string()).ok();

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

    fn source_hash(&self) -> Hash {
        let mut hasher = blake3::Hasher::new();
        let src_dir = self.repo_root.join("janet").join("src");
        let mut entries: Vec<_> = walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "janet"))
            .collect();
        entries.sort_by_key(|e| e.path().to_owned()); // stable order
        for entry in entries {
            let mut fh = fs::File::open(entry.path()).expect("cannot read source file");
            std::io::copy(&mut fh, &mut hasher).expect("cannot hash source file");
        }
        hasher.finalize()
    }
}
