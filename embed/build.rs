// I don't want Janet to be a build dependency, so this compiles a Gurp library image file from
// source.

fn main() {
    build_helper::ImageHelper::new(vec!["gurp.janet"], "gurp.jimage").compile_to_file();
    println!("cargo:rerun-if-changed=janet/lib/gurp.jimage");
}
