#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;
    use snltest::fixture;

    #[test]
    #[ignore]
    fn test_apply_janet_without_defs() {
        cargo_bin_cmd!("gurp")
            .arg("apply")
            .arg(fixture!("apply/def-test.janet"))
            .assert()
            .success()
            .stdout(predicate::str::contains("no definition found"));
    }

    #[test]
    #[ignore]
    fn test_apply_janet_with_defs() {
        cargo_bin_cmd!("gurp")
            .arg("apply")
            .arg("-Dtest-def")
            .arg(fixture!("apply/def-test.janet"))
            .assert()
            .success()
            .stdout(predicate::str::contains("found a definition"));
    }
}
