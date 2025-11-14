#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_server_no_args() {
        cargo_bin_cmd!("gurp")
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
        cargo_bin_cmd!("gurp")
            .arg("server")
            .arg("--config-dir=/nodir")
            .assert()
            .failure()
            .stdout(predicate::str::contains("did not find config dir: /nodir"));
    }
}
