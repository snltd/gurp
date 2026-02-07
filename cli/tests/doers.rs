#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_doers_command() {
        cargo_bin_cmd!("gurp")
            .arg("doers")
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "etherstub  Create and destroy etherstubs.",
            ))
            .stdout(predicate::str::contains(
                "zone  Create and destroy zones. Existing zones
                            cannot be modified.",
            ));
        // Second one should test the columns are aligned
    }
}
