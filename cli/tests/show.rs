#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_show_no_args() {
        cargo_bin_cmd!("gurp")
            .arg("show")
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "the following required arguments were not provided",
            ));
    }

    #[test]
    #[ignore]
    fn test_show_bad_thing() {
        cargo_bin_cmd!("gurp")
            .arg("show")
            .arg("whatever")
            .assert()
            .failure()
            .stdout(predicate::str::ends_with(
                "whatever is not a thing I can show you\n",
            ));
    }

    #[test]
    #[ignore]
    fn test_show_library() {
        cargo_bin_cmd!("gurp")
            .arg("show")
            .arg("library")
            .assert()
            .success()
            .stdout(predicate::str::contains("Creates a resource struct"));
    }

    #[test]
    #[ignore]
    fn test_show_defaults() {
        cargo_bin_cmd!("gurp")
            .arg("show")
            .arg("defaults")
            .assert()
            .success()
            .stdout(predicate::str::contains("{:owner \"root\""));
    }
}
