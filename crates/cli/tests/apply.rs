#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;
    use snltest::fixture;

    // Make sure that definitions are passed through to the runtime, and that essential
    // bindings are visible, for Janet and jimage config.

    #[test]
    #[ignore]
    fn test_apply_janet_without_defs() {
        cargo_bin_cmd!("gurp")
            .arg("apply")
            .arg("--no-lock")
            .arg("--no-report")
            .arg(fixture!("apply/def-test.janet"))
            .assert()
            .success()
            .stdout(predicate::str::contains("no definition found"))
            .stdout(predicate::str::contains("crates/cli/tests/resources/apply"));
    }

    #[test]
    #[ignore]
    fn test_apply_janet_with_defs() {
        cargo_bin_cmd!("gurp")
            .arg("apply")
            .arg("--no-lock")
            .arg("--no-report")
            .arg("-Dtest-def")
            .arg(fixture!("apply/def-test.janet"))
            .assert()
            .success()
            .stdout(predicate::str::contains("found a definition"))
            .stdout(predicate::str::contains("crates/cli/tests/resources/apply"));
    }

    #[test]
    #[ignore]
    fn test_apply_jimage_without_defs() {
        cargo_bin_cmd!("gurp")
            .arg("apply")
            .arg("--no-lock")
            .arg("--no-report")
            .arg("--image")
            .arg(fixture!("apply/def-test.jimage"))
            .assert()
            .success()
            .stdout(predicate::str::contains("crates/cli/tests/resources/apply"))
            .stdout(predicate::str::contains("no definition found"));
    }

    #[test]
    #[ignore]
    fn test_apply_jimage_with_defs() {
        cargo_bin_cmd!("gurp")
            .arg("apply")
            .arg("--no-lock")
            .arg("--no-report")
            .arg("-Dtest-def")
            .arg("--image")
            .arg(fixture!("apply/def-test.jimage"))
            .assert()
            .success()
            // .stdout(predicate::str::contains("crates/cli/tests/resources/apply"))
            .stdout(predicate::str::contains("found a definition"));
    }
}
