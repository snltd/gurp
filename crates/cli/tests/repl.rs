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

    #[test]
    #[ignore]
    fn test_repl_sets_syspath() {
        cargo_bin_cmd!("gurp")
            .arg("repl")
            .arg("--syspath=/usr/bin")
            .write_stdin("(dyn *syspath*)")
            .assert()
            .success()
            .stdout(predicate::str::contains(r#"repl:1:> "/usr/bin""#));
    }

    #[test]
    #[ignore]
    fn test_repl_sets_config_root() {
        cargo_bin_cmd!("gurp")
            .arg("repl")
            .arg("--gurp-config-root=/usr")
            .write_stdin("(dyn :gurp-config-root)")
            .assert()
            .success()
            .stdout(predicate::str::contains(r#"repl:1:> "/usr""#));
    }

    #[test]
    #[ignore]
    fn test_defs_in_repl() {
        cargo_bin_cmd!("gurp")
            .arg("repl")
            .arg("-Done=un")
            .arg("-Dtwo=deux")
            .arg("-Dfrench")
            .write_stdin("(user-def :one) (user-def :two) (user-def :french) (user-def :german)")
            .assert()
            .success()
            .stdout(indoc::indoc! {
                r#"repl:1:> "un"
                "deux"
                true
                nil
                repl:1:> "#
            });
    }
}
