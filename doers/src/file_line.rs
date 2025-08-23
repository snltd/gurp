use camino::Utf8PathBuf;
use common::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::io::Write;

// THINGS TO KNOW / THINGS TO DO.
// File is not managed here. Use a file resource.
// This is super-basic. It appends lines and removes lines. That's it.
// Doesn't even do regex. Exact matches only.
// It reads the entirety of the file into memory.
// Appended lines have a \n at the beginning and end.
// Removing a line puts a newline on the end of the file if there wasn't one already.
// We always read the file. There's no caching or anyhing.
// Files are not backed up.

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpFileLineEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub line: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpFileLineRemove {
    #[serde(rename = "_id")]
    pub id: String,
    pub pattern: String,
    #[serde(rename = "match")]
    pub match_type: String,
    pub apply_to: String,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

fn line_exists(path: &Utf8PathBuf, line: &str) -> anyhow::Result<bool> {
    let contents = fs::read_to_string(path)?;
    Ok(contents.lines().any(|l| l == line))
}

impl GurpFileLineEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if line_exists(&self.path, &self.line)? {
            tracing::debug!("no change: {}", &self.path);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("creating: {}", &self.path);

            return_if_noop!(opts);
            let fh = fs::OpenOptions::new().append(true).open(&self.path)?;
            writeln!(&fh, "\n{}\n", self.line.as_str())?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }
}

impl GurpFileLineRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if line_exists(&self.path, &self.pattern)? {
            tracing::info!("removing: {}", &self.path);

            return_if_noop!(opts);
            let content = fs::read_to_string(&self.path)?;

            let out = remove_lines(&content, &self.match_type, &self.pattern, &self.apply_to)?;

            fs::write(&self.path, out)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("no change: {}", &self.path);
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn remove_lines(
    orig: &str,
    match_type: &str,
    pattern: &str,
    apply_to: &str,
) -> anyhow::Result<String> {
    let rx = if match_type == "regex" {
        Some(Regex::new(pattern)?)
    } else {
        None
    };

    let mut seen_match = false;
    let mut lines: Vec<_> = orig.lines().collect();

    if apply_to == "last" {
        lines.reverse();
    };

    let mut ret: Vec<_> = lines
        .iter()
        .filter(|&line| {
            if apply_to != "all" && seen_match {
                true
            } else {
                match match_type {
                    "exact" => {
                        if line == &pattern {
                            seen_match = true;
                            false
                        } else {
                            true
                        }
                    }

                    "ends_with" => {
                        if line.ends_with(pattern) {
                            seen_match = true;
                            false
                        } else {
                            true
                        }
                    }
                    "starts_with" => {
                        if line.starts_with(pattern) {
                            seen_match = true;
                            false
                        } else {
                            true
                        }
                    }
                    "contains" => {
                        if line.contains(pattern) {
                            seen_match = true;
                            false
                        } else {
                            true
                        }
                    }
                    "regex" => {
                        if let Some(regex) = &rx {
                            if regex.is_match(line) {
                                seen_match = true;
                                false
                            } else {
                                true
                            }
                        } else {
                            unreachable!("regex but no regex")
                        }
                    }
                    other => unreachable!("Impossible match-type: {other}"),
                }
            }
        })
        .map(|line| format!("{line}\n"))
        .collect();

    if apply_to == "last" {
        ret.reverse();
    }

    Ok(ret.join(""))
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;
    use indoc::{formatdoc, indoc};
    // use pretty_assertions::assert_eq;
    use tester::{defopts, defopts_noop, janet2json};

    #[test]
    fn test_file_line_ensure_file_does_not_exist() {
        let json_def = janet2json(indoc! {r#"
            (file-line/ensure "/test-role/file-line/test-does-not-exist"
                :line "some irrelevant text")
                "#});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();
        assert!(sut.apply(&defopts()).is_err());
    }

    #[test]
    fn test_file_line_ensure_file_does_not_contain_desired_line() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/ensure \"{}\" :line \"line_4\")
            ", file_to_modify});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3\nline_4\n\n".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_ensure_file_does_not_contain_desired_line_noop() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/ensure \"{}\" :line \"line_4\")
            ", file_to_modify});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NOOP, sut.apply(&defopts_noop()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_ensure_file_contains_desired_line() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/ensure \"{}\" :line \"line_3\")
            ", file_to_modify});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_remove_file_contains_desired_line() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/remove \"{}\"
                :pattern \"line_2\"
                :match \"exact\"
                :apply-to \"all\" )
            ", file_to_modify});

        let sut: GurpFileLineRemove = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(
            "line_1\nline_3\n".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_remove_file_does_not_contain_desired_line() {
        let temp = TempDir::new().unwrap();
        temp.child("test-file")
            .write_str("line_1\nline_2\nline_3")
            .unwrap();
        let file_to_modify = temp.join("test-file");

        let json_def = janet2json(&formatdoc! {"
            (file-line/remove \"{}\"
                :pattern \"line_4\"
                :match \"exact\"
                :apply-to \"all\")
            ", file_to_modify.to_string_lossy()});

        let sut: GurpFileLineRemove = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_NO_CHANGE, sut.apply(&defopts_noop()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_remove_all_lines() {
        let src = "merp\nbyerp\nmerp\ngurp\nmerp\nbyerp\n";

        assert_eq!(
            "byerp\ngurp\nbyerp\n".to_owned(),
            remove_lines(src, "exact", "merp", "all").unwrap()
        );

        assert_eq!(
            "gurp\n".to_owned(),
            remove_lines(src, "contains", "er", "all").unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\nmerp\nbyerp\n",
            remove_lines(src, "starts_with", "g", "all").unwrap()
        );

        assert_eq!(
            "merp\nmerp\nmerp\n",
            remove_lines(src, "regex", "^[a-h].*p$", "all").unwrap()
        );
    }

    #[test]
    fn test_remove_first_lines() {
        let src = "merp\nbyerp\nmerp\ngurp\nmerp\nbyerp\n";

        assert_eq!(
            "byerp\nmerp\ngurp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "exact", "merp", "first").unwrap()
        );

        assert_eq!(
            "byerp\nmerp\ngurp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "contains", "er", "first").unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "starts_with", "g", "first").unwrap()
        );

        assert_eq!(
            "merp\nmerp\ngurp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "regex", "^[a-h].*p$", "first").unwrap()
        );
    }

    #[test]
    fn test_remove_last_lines() {
        let src = "merp\nbyerp\nmerp\ngurp\nmerp\nbyerp\n";

        assert_eq!(
            "merp\nbyerp\nmerp\ngurp\nbyerp\n".to_owned(),
            remove_lines(src, "exact", "merp", "last").unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\ngurp\nmerp\n".to_owned(),
            remove_lines(src, "contains", "er", "last").unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "starts_with", "g", "last").unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\ngurp\nmerp\n".to_owned(),
            remove_lines(src, "regex", "^[a-h].*p$", "last").unwrap()
        );
    }

    fn test_file() -> (TempDir, Utf8PathBuf) {
        let temp = TempDir::new().unwrap();
        let file = temp.child("test-file");
        file.write_str("line_1\nline_2\nline_3").unwrap();
        (
            temp,
            Utf8PathBuf::from_path_buf(file.path().to_path_buf()).unwrap(),
        )
    }
}
