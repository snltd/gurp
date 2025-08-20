#[cfg(test)]
mod test {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_descibe_no_args() {
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
    fn test_describe_gem() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("describe")
            .arg("gem")
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "version    string          Gem version",
            ));
    }

    #[test]
    #[ignore]
    fn test_describe_no_such_resource() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("describe")
            .arg("nonsense")
            .assert()
            .success()
            .stdout(predicate::str::contains("No help for 'nonsense'"));
    }
}
