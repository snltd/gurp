#[cfg(test)]
mod test {
    use assert_cmd::Command;
    use gurp::test_utils::spec_helper::fixture;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_missing_module() {
        Command::cargo_bin("gurp")
            .unwrap()
            .arg(fixture("missing_modules.janet"))
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "Failed to load module 'missing'\n",
            ));
    }
}
