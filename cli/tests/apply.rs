#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_apply_no_args() {
        cargo_bin_cmd!("gurp")
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
        cargo_bin_cmd!("gurp")
            .env("GURP_NO_COLOUR", "1")
            .arg("apply")
            .arg("/no/such/file.janet")
            .assert()
            .failure()
            .stdout(predicate::str::contains(
                "could not generate config: missing file error: /no/such/file.janet",
            ));
    }

    #[test]
    #[ignore]
    fn test_bad_janet() {
        cargo_bin_cmd!("gurp")
            .env("GURP_NO_COLOUR", "1")
            .arg("apply")
            .arg("./tests/resources/bad.janet")
            .assert()
            .failure()
            .stderr(predicate::str::contains("unknown symbol physical"));
    }
}
