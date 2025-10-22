#[cfg(test)]
mod test {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_server_no_args() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("server")
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "the following required arguments were not provided",
            ));
    }

    #[test]
    #[ignore]
    fn test_server_mising_dir() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg("server")
            .arg("--config-dir=/nodir")
            .assert()
            .failure()
            .stdout(predicate::str::contains("did not find config dir: /nodir"));
    }
}
