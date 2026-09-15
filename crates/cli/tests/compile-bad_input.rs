#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_compile_no_args() {
        cargo_bin_cmd!("gurp")
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
        cargo_bin_cmd!("gurp")
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
        cargo_bin_cmd!("gurp")
            .env("GURP_NO_COLOUR", "1")
            .arg("compile")
            .arg("--format=json")
            .arg("/no/such/file.janet")
            .assert()
            .failure()
            .stdout(predicate::str::contains(
                "missing file error: /no/such/file.janet",
            ));
    }

    #[test]
    #[ignore]
    fn test_compile_bad_janet() {
        cargo_bin_cmd!("gurp")
            .arg("compile")
            .arg("--format=json")
            .arg("tests/resources/bad.janet")
            .assert()
            .failure()
            .stdout(predicate::str::contains(
                "compile error: unknown symbol physical",
            ));
    }
}
