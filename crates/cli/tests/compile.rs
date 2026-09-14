#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use pretty_assertions::assert_eq;
    use snltest::{cwd, fixture, load_fixture};

    #[test]
    #[ignore]
    fn test_compile_to_json() {
        let canonical_test_dir = "/home/rob/work/gurp/cli";
        let test_dir = cwd().to_string();

        for host in [
            "backup",
            "dev-server",
            "grafana",
            "mariadb",
            "minidlna",
            "pkg-server",
            "records",
            "remover",
            "serv-zones",
        ] {
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
}
