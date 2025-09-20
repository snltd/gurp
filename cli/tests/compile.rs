#[cfg(test)]
mod test {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tester::{cwd, fixture, load_fixture};

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
            let canonical_json = load_fixture(&format!("compile/outputs/{host}.json"));
            let expected_json = canonical_json.replace(canonical_test_dir, &test_dir);

            Command::cargo_bin("gurp")
                .unwrap()
                .arg("compile")
                .arg(fixture(&format!("compile/inputs/{host}.janet")))
                .arg("--format=json")
                .assert()
                .success()
                .stdout(expected_json);
        }
    }

    #[test]
    #[ignore]
    fn test_compile_no_args() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("compile")
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "the following required arguments were not provided",
            ));
    }

    #[test]
    #[ignore]
    fn test_compile_no_format() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("compile")
            .arg("tests/resources/sample/serv-gurp.janet")
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "the following required arguments were not provided",
            ));
    }

    #[test]
    #[ignore]
    fn test_compile_missing_file() {
        Command::cargo_bin("gurp")
            .unwrap()
            .env("GURP_NO_COLOUR", "1")
            .arg("compile")
            .arg("--format=json")
            .arg("/no/such/file")
            .assert()
            .failure()
            .stdout(predicate::str::ends_with(
                "reader error: No such file or directory (os error 2)\n",
            ));
    }

    #[test]
    #[ignore]
    fn test_compile_bad_janet() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("compile")
            .arg("--format=json")
            .arg("tests/resources/bad.janet")
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "compile error: unknown symbol physical",
            ));
    }

    #[test]
    #[ignore]
    fn test_compile_janet() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("compile")
            .arg("tests/resources/sample/serv-gurp.janet")
            .arg("--format=janet")
            .assert()
            .success()
            .stdout(predicate::str::contains(":directory"));
    }

    #[test]
    #[ignore]
    fn test_compile_json() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("compile")
            .arg("tests/resources/sample/serv-gurp.janet")
            .arg("--format=json")
            .assert()
            .success()
            .stdout(predicate::str::contains("\"directory\":"));
    }
}
