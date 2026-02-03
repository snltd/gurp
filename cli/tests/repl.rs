#[cfg(test)]
mod test {
    use assert_cmd::cargo::cargo_bin_cmd;
    use predicates::prelude::*;

    #[test]
    #[ignore]
    fn test_repl_with_file_ensure() {
        cargo_bin_cmd!("gurp")
            .arg("repl")
            .write_stdin("(doc file/ensure) (os/exit)")
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Given a file path and spec, put an ensure struct in the collector.",
            ));
    }
}
