#[cfg(test)]
mod test {
    use assert_cmd::Command;
    // use img_tool::test_utils::spec_helper::fixture;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_no_args() {
        Command::cargo_bin("gurp")
            .unwrap()
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "the following required arguments were not provided",
            ));
    }

    #[test]
    #[ignore]
    fn test_missing_file() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("/no/such/dir")
            .assert()
            .failure()
            .stderr("Error configuring host: No such file or directory (os error 2)\n");
    }
}
