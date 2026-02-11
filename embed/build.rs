// I don't want Janet to be a build dependency, so this compiles a Gurp library image file from
// source.

fn main() {
    println!("cargo:rerun-if-changed=../janet/src");

    build_helper::ImageHelper::new(vec!["gurp.janet", "command-helpers.janet"], "gurp.jimage")
        .compile_to_file();
}
