#[cfg(test)]
mod test {
    use assert_cmd::Command;
    use predicates::prelude::*;

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
    fn test_compile_missing_file() {
        Command::cargo_bin("gurp")
            .unwrap()
            .env("GURP_NO_COLOUR", "1")
            .arg("compile")
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
            .arg("tests/resources/bad.janet")
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "compile error: unknown symbol physical",
            ));
    }

    #[test]
    #[ignore]
    fn test_compile() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("compile")
            .arg("tests/resources/sample/serv-gurp.janet")
            .assert()
            .success()
            .stdout(predicate::str::contains(":directory"));
    }
}
