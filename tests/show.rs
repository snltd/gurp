#[cfg(test)]
mod test {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_show_no_args() {
        Command::cargo_bin("gurp")
            .unwrap()
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
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("show")
            .arg("whatever")
            .assert()
            .failure()
            .stderr("That's not a thing I can show you\n");
    }

    #[test]
    #[ignore]
    fn test_show_library() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("show")
            .arg("library")
            .assert()
            .success()
            .stdout(predicate::str::contains("Creates a resource struct"));
    }

    #[test]
    #[ignore]
    fn test_show_defaults() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("show")
            .arg("defaults")
            .assert()
            .success()
            .stdout(predicate::str::contains("{:file {:owner \"root\""));
    }
}
