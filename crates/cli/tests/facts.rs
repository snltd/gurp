#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_facts_command() {
        cargo_bin_cmd!("gurp")
            .arg("facts")
            .assert()
            .success()
            .stdout(predicate::str::contains(
                ":zones  A struct with zone names names as keys, and",
            ));
    }
}
