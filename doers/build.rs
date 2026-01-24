use embed::helpers;
use std::fs;

fn main() {
    let client = helpers::gurp_client().unwrap();
    let janet_instructions = format!(
        "(setdyn :running-embedded true)\n(setdyn :repo-root \"/home/rob/work/gurp\")\n(setdyn *syspath* \"/home/rob/work/gurp/janet/src\")\n{}\n(generate-all-docs)",
        &fs::read_to_string("/home/rob/work/gurp/janet/src/build-docgen.janet").unwrap()
    );

    client.run(janet_instructions).unwrap();
}
