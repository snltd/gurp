// I don't want Janet to be a build dependency, so this compiles a Gurp library image file from
// source.

fn main() {
    for entry in walkdir::WalkDir::new("../janet/src") {
        let entry = entry.unwrap();
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    build_helper::ImageHelper::new(vec!["gurp.janet", "command-lib.janet"], "gurp.jimage")
        .compile_to_file();
}
