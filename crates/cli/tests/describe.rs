#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_describe_no_args() {
        cargo_bin_cmd!("gurp")
            .arg("compile")
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "the following required arguments were not provided",
            ));
    }

    #[test]
    #[ignore]
    fn test_describe_doer() {
        cargo_bin_cmd!("gurp")
            .arg("describe")
            .arg("gem")
            .assert()
            .success()
            .stdout(predicate::str::contains("Install and uninstall Ruby gems."));
    }

    #[test]
    #[ignore]
    fn test_describe_helper() {
        cargo_bin_cmd!("gurp")
            .arg("describe")
            .arg("zone/bhyve")
            .assert()
            .success()
            .stdout(predicate::str::contains("Mandatory properties"))
            .stdout(predicate::str::contains("Optional properties"))
            .stdout(predicate::str::contains(
                "A bhyve zone must be built from an image.",
            ));
    }

    #[test]
    #[ignore]
    fn test_describe_no_such_resource() {
        cargo_bin_cmd!("gurp")
            .arg("describe")
            .arg("nonsense")
            .assert()
            .success()
            .stdout("No help for 'nonsense'\n");
    }
}
