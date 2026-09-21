#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;
    use snltest::fixture;

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

    #[test]
    #[cfg(target_os = "illumos")]
    #[ignore]
    fn test_facts_work() {
        cargo_bin_cmd!("gurp")
            .arg("apply")
            .arg("--no-lock")
            .arg("--no-report")
            .arg(fixture!("test-facts.janet"))
            .assert()
            .success();
    }
}
