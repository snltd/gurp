#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;
    use pretty_assertions::assert_eq;
    use snltest::{cwd, fixture, load_fixture};

    const COLOUR_MARKER: &str = "\x1b[33m";
    const LINE_NO_MARKER: &str = " 4 | ";

    const SOURCE_FILES: [&str; 9] = [
        "backup",
        "dev-server",
        "grafana",
        "mariadb",
        "minidlna",
        "pkg-server",
        "records",
        "remover",
        "serv-zones",
    ];

    #[test]
    #[ignore]
    fn test_compile_to_json() {
        let canonical_test_dir = "/home/rob/work/gurp/cli";
        let test_dir = cwd().to_string();

        for host in SOURCE_FILES {
            let canonical_json = load_fixture!(&format!("compile/outputs/{host}.json"));
            let expected_json = canonical_json.replace(canonical_test_dir, &test_dir);

            let output = cargo_bin_cmd!("gurp")
                .arg("compile")
                .arg(fixture!(&format!("compile/inputs/{host}.janet")))
                .arg("--format=json")
                .assert()
                .success();

            let actual: serde_json::Value =
                serde_json::from_slice(&output.get_output().stdout).unwrap();
            let expected: serde_json::Value = serde_json::from_str(&expected_json).unwrap();

            assert_eq!(actual, expected);
        }
    }

    #[test]
    #[ignore]
    fn test_compile_to_jimage() {
        let (_tp, outdir) = snltest::fixture_dir("janet-test", vec![]);

        for host in SOURCE_FILES {
            assert!(outdir.exists());
            let outfile = outdir.join("out.jimage");

            cargo_bin_cmd!("gurp")
                .arg("compile")
                .arg(fixture!(&format!("compile/inputs/{host}.janet")))
                .arg(format!("--output-file={outfile}"))
                .arg("--format=jimage")
                .assert()
                .success();

            assert!(outfile.exists());

            let size = outfile.metadata().unwrap().len();
            // Failed compiles will be about 100 bytes long.
            println!("{size}");
            assert!(size > 10000);
        }
    }

    #[test]
    #[ignore]
    fn test_compile_to_janet() {
        for host in SOURCE_FILES {
            cargo_bin_cmd!("gurp")
                .arg("compile")
                .arg(fixture!(&format!("compile/inputs/{host}.janet")))
                .arg("--format=janet")
                .assert()
                .stdout(predicate::str::contains(":resources {:ensure"))
                .stdout(predicate::str::contains(COLOUR_MARKER).not())
                .stdout(predicate::str::contains(LINE_NO_MARKER).not());
        }
    }

    #[test]
    #[ignore]
    fn test_compile_line_no() {
        for format in ["janet", "json"] {
            cargo_bin_cmd!("gurp")
                .arg("compile")
                .arg(fixture!("compile/inputs/grafana.janet"))
                .arg("--line-no")
                .arg("--format")
                .arg(format)
                .assert()
                .stdout(predicate::str::contains(COLOUR_MARKER).not())
                .stdout(predicate::str::contains(LINE_NO_MARKER));
        }
    }

    #[test]
    #[ignore]
    fn test_compile_colour() {
        // We don't colour JSON. We only do Janet 'cos it's free
        cargo_bin_cmd!("gurp")
            .arg("compile")
            .arg(fixture!("compile/inputs/grafana.janet"))
            .arg("--colour")
            .arg("--line-no")
            .arg("--format=janet")
            .assert()
            .stdout(predicate::str::contains(COLOUR_MARKER))
            .stdout(predicate::str::contains(LINE_NO_MARKER));

        cargo_bin_cmd!("gurp")
            .arg("compile")
            .arg(fixture!("compile/inputs/grafana.janet"))
            .arg("--colour")
            .arg("--format=janet")
            .assert()
            .stdout(predicate::str::contains(COLOUR_MARKER))
            .stdout(predicate::str::contains(LINE_NO_MARKER).not());
    }
}
