use anyhow::{bail, ensure};
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE};
use common::types::{ApplyOpts, ApplySummary};
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::io::Write;

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct GurpFileLineEnsure {
    #[serde(rename = "_id")]
    pub id: String,
    pub line: Option<String>,
    pub insert_at: Option<usize>,
    pub replace: Option<String>,
    pub with: Option<String>,
    pub apply_to: Option<String>,
    #[serde(rename = "name")]
    pub path: Utf8PathBuf,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
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

impl GurpFileLineEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        ensure!(
            self.path.exists(),
            "{} does not exist: file-line cannot ensure its contents",
            self.path
        );

        ensure!(self.path.is_file(), "{} is not a regular file", self.path);

        ensure!(
            !(self.line.is_some() && self.replace.is_some()),
            "use either :line or :replace, not both"
        );

        if let Some(line) = &self.line {
            self.apply_line(line, opts)
        } else if let Some(replace) = &self.replace {
            if let Some(with) = &self.with {
                self.apply_replace(replace, with, opts)
            } else {
                bail!(":replace requires :with");
            }
        } else {
            bail!("Need a :line or a :replace :with pair")
        }
    }

    fn apply_line(&self, line: &str, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        if line_exists(&self.path, line)? {
            tracing::debug!("no change: {}", &self.path);
            Ok(ONE_RESOURCE_NO_CHANGE)
        } else {
            tracing::info!("creating: {}", &self.path);

            return_if_noop!(opts);

            if let Some(index) = self.insert_at {
                self.insert_line_at_index(line, index, opts)
            } else {
                let fh = fs::OpenOptions::new().append(true).open(&self.path)?;
                writeln!(&fh, "\n{}\n", line)?;
                Ok(ONE_RESOURCE_ONE_CHANGE)
            }
        }
    }

    fn insert_line_at_index(
        &self,
        line: &str,
        index: usize,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        let raw = fs::read_to_string(&self.path)?;
        let mut lines: Vec<_> = raw.lines().collect();

        if index >= lines.len() {
            tracing::debug!("appending line to {}", &self.path);
            lines.push(line);
        } else {
            tracing::debug!("inserting line at {}:{}", &self.path, index);
            lines.insert(index, line);
        }

        let new_content: String = lines.iter().map(|l| format!("{l}\n")).collect();
        write_content(&self.path, &new_content, opts)
    }

    fn apply_replace(
        &self,
        replace: &str,
        with: &str,
        opts: &ApplyOpts,
    ) -> anyhow::Result<ApplySummary> {
        let orig = fs::read_to_string(&self.path)?;

        if let Some(new_content) = replace_lines(&orig, replace, with, self.apply_to.as_deref())? {
            write_content(&self.path, &new_content, opts)
        } else {
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn line_exists(path: &Utf8PathBuf, line: &str) -> anyhow::Result<bool> {
    let contents = fs::read_to_string(path)?;
    Ok(contents.lines().any(|l| l == line))
}

fn replace_lines(
    orig: &str,
    replace: &str,
    with: &str,
    apply_to: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let rx = Regex::new(replace)?;
    let mut made_change = false;
    let mut lines: Vec<_> = orig.lines().collect();

    let apply_to = apply_to.unwrap_or("all");

    if apply_to == "last" {
        lines.reverse();
    };

    let change_all = apply_to == "all";

    let mut new_lines: Vec<_> = lines
        .iter()
        .map(|line| {
            if (!made_change || change_all) && rx.is_match(line) {
                made_change = true;
                rx.replace_all(line, with).into_owned()
            } else {
                line.to_string()
            }
        })
        .collect();

    if made_change {
        if apply_to == "last" {
            new_lines.reverse();
        }
        Ok(Some(new_lines.iter().map(|l| format!("{l}\n")).collect()))
    } else {
        Ok(None)
    }
}

fn write_content(
    path: &Utf8PathBuf,
    content: &str,
    opts: &ApplyOpts,
) -> anyhow::Result<ApplySummary> {
    return_if_noop!(opts);
    tracing::debug!("writing new content to {path}");
    fs::write(path, content)?;
    Ok(ONE_RESOURCE_ONE_CHANGE)
}

impl GurpFileLineRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let content = fs::read_to_string(&self.path)?;

        if let Some(new_content) =
            remove_lines(&content, &self.match_type, &self.pattern, &self.apply_to)?
        {
            tracing::info!("removing line(s) from {}", &self.path);

            return_if_noop!(opts);

            write_content(&self.path, &new_content, opts)
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
) -> anyhow::Result<Option<String>> {
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

    if seen_match {
        if apply_to == "last" {
            ret.reverse();
        }

        Ok(Some(ret.join("")))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use camino_tempfile_ext::prelude::*;
    use common::constants::ONE_RESOURCE_NOOP;
    use indoc::{formatdoc, indoc};
    use pretty_assertions::assert_eq;
    use tester::deserialized_example;
    use tester::{defopts, defopts_noop, janet2json};

    #[test]
    fn test_file_line_deserialize_ensure_01() {
        assert_eq!(
            GurpFileLineEnsure {
                path: Utf8PathBuf::from("/path/to/file"),
                id: "/NO-ROLE/file-line/_path_to_file".to_owned(),
                line: Some("i-want-to-see-this".to_owned()),
                insert_at: None,
                replace: None,
                with: None,
                apply_to: None,
            },
            deserialized_example::<GurpFileLineEnsure>("file-line/ensure-01.janet")
        );
    }

    #[test]
    fn test_file_line_deserialize_remove_01() {
        assert_eq!(
            GurpFileLineRemove {
                path: Utf8PathBuf::from("/path/to/file"),
                id: "/NO-ROLE/file-line/_path_to_file".to_owned(),
                pattern: "i-do-not-want-to-see-this-anywhere".to_owned(),
                match_type: "exact".to_owned(),
                apply_to: "all".to_owned(),
            },
            deserialized_example::<GurpFileLineRemove>("file-line/remove-01.janet")
        );
    }

    #[test]
    fn test_file_line_deserialize_remove_02() {
        assert_eq!(
            GurpFileLineRemove {
                path: Utf8PathBuf::from("/path/to/file"),
                id: "/NO-ROLE/file-line/_path_to_file".to_owned(),
                pattern: "rust-regex".to_owned(),
                match_type: "regex".to_owned(),
                apply_to: "all".to_owned(),
            },
            deserialized_example::<GurpFileLineRemove>("file-line/remove-02.janet")
        );
    }

    #[test]
    fn test_file_line_deserialize_remove_03() {
        assert_eq!(
            GurpFileLineRemove {
                path: Utf8PathBuf::from("/path/to/file"),
                id: "/NO-ROLE/file-line/_path_to_file".to_owned(),
                pattern: "string-prefix".to_owned(),
                match_type: "starts-with".to_owned(),
                apply_to: "last".to_owned(),
            },
            deserialized_example::<GurpFileLineRemove>("file-line/remove-03.janet")
        );
    }

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
    fn test_file_line_ensure_file_does_not_contain_desired_line_index() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/ensure \"{}\" :line \"new line\" :insert-at 0)
            ", file_to_modify});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(
            "new line\nline_1\nline_2\nline_3\n".to_owned(),
            fs::read_to_string(&file_to_modify).unwrap()
        );
    }

    #[test]
    fn test_file_line_ensure_file_does_not_contain_desired_line_big_index() {
        let (_t, file_to_modify) = test_file();

        let json_def = janet2json(&formatdoc! {"
            (file-line/ensure \"{}\" :line \"new line\" :insert-at 100)
            ", file_to_modify});

        let sut: GurpFileLineEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert_eq!(
            "line_1\nline_2\nline_3\nnew line\n".to_owned(),
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
    fn test_replace_line() {
        let src = "line one\nline two\nline three\nline four\n";

        assert_eq!(
            "LINE one\nLINE two\nLINE three\nLINE four\n".to_owned(),
            replace_lines(src, "line", "LINE", None).unwrap().unwrap()
        );

        assert_eq!(
            "LINE one\nline two\nline three\nline four\n".to_owned(),
            replace_lines(src, "line", "LINE", Some("first"))
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            "line one\nline two\nline three\nLINE four\n".to_owned(),
            replace_lines(src, "line", "LINE", Some("last"))
                .unwrap()
                .unwrap()
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
        let temp_dir = Utf8TempDir::new().unwrap();
        temp_dir
            .child("test-file")
            .write_str("line_1\nline_2\nline_3")
            .unwrap();

        let file_to_modify = temp_dir.path().join("test-file");

        let json_def = janet2json(&formatdoc! {"
            (file-line/remove \"{}\"
                :pattern \"line_4\"
                :match \"exact\"
                :apply-to \"all\")
            ", file_to_modify});

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
            remove_lines(src, "exact", "merp", "all").unwrap().unwrap()
        );

        assert_eq!(
            "gurp\n".to_owned(),
            remove_lines(src, "contains", "er", "all").unwrap().unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\nmerp\nbyerp\n",
            remove_lines(src, "starts_with", "g", "all")
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            "merp\nmerp\nmerp\n",
            remove_lines(src, "regex", "^[a-h].*p$", "all")
                .unwrap()
                .unwrap()
        );
    }

    #[test]
    fn test_remove_first_lines() {
        let src = "merp\nbyerp\nmerp\ngurp\nmerp\nbyerp\n";

        assert_eq!(
            "byerp\nmerp\ngurp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "exact", "merp", "first")
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            "byerp\nmerp\ngurp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "contains", "er", "first")
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "starts_with", "g", "first")
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            "merp\nmerp\ngurp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "regex", "^[a-h].*p$", "first")
                .unwrap()
                .unwrap()
        );
    }

    #[test]
    fn test_remove_last_lines() {
        let src = "merp\nbyerp\nmerp\ngurp\nmerp\nbyerp\n";

        assert_eq!(
            "merp\nbyerp\nmerp\ngurp\nbyerp\n".to_owned(),
            remove_lines(src, "exact", "merp", "last").unwrap().unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\ngurp\nmerp\n".to_owned(),
            remove_lines(src, "contains", "er", "last")
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\nmerp\nbyerp\n".to_owned(),
            remove_lines(src, "starts_with", "g", "last")
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            "merp\nbyerp\nmerp\ngurp\nmerp\n".to_owned(),
            remove_lines(src, "regex", "^[a-h].*p$", "last")
                .unwrap()
                .unwrap()
        );
    }

    fn test_file() -> (Utf8TempDir, Utf8PathBuf) {
        let temp = Utf8TempDir::new().unwrap();
        let file = temp.child("test-file");
        file.write_str("line_1\nline_2\nline_3").unwrap();
        (temp, file.as_path().to_path_buf())
    }
}
