#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_describe_gem() {
        cargo_bin_cmd!("gurp")
            .arg("doers")
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\x1b[1metherstub\x1b[0m  Create and destroy etherstubs.",
            ))
            .stdout(predicate::str::contains(
                "\x1b[1mzone\x1b[0m  Create and destroy zones. Existing zones
                            cannot be modified.",
            ));
        // Second one should test the columns are aligned
    }
}
