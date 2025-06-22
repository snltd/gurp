#[cfg(test)]
mod test {
    use assert_cmd::Command;
    use gurp::test_utils::spec_helper::fixture;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_apply_no_args() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("apply")
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "the following required arguments were not provided",
            ));
    }

    #[test]
    #[ignore]
    fn test_apply_missing_file() {
        Command::cargo_bin("gurp")
            .unwrap()
            .env("GURP_NO_COLOUR", "1")
            .arg("apply")
            .arg("/no/such/file.janet")
            .assert()
            .failure()
            .stdout(predicate::str::ends_with(
                "run error: No such file or directory (os error 2)\n",
            ));
    }

    #[test]
    #[ignore]
    fn test_bad_janet() {
        Command::cargo_bin("gurp")
            .unwrap()
            .env("GURP_NO_COLOUR", "1")
            .arg("apply")
            .arg(fixture("bad.janet"))
            .assert()
            .failure()
            .stderr(predicate::str::ends_with(
                "compile error: unknown symbol physical\n",
            ));
    }
}
