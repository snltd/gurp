use camino::Utf8Path;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct FileFilter<'a> {
    pattern: &'a str,
    rx: Regex,
}

impl<'a> FileFilter<'a> {
    pub fn from(pattern: &'a str) -> anyhow::Result<Self> {
        Ok(Self {
            pattern,
            rx: Regex::new(pattern)?,
        })
    }

    pub fn string(&self, content: &str) -> String {
        tracing::debug!("filtering string on '{}'", self.pattern);
        content
            .lines()
            .filter(|l| !self.rx.is_match(l))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn file(&self, path: &Utf8Path) -> anyhow::Result<String> {
        let input = File::open(path)?;
        let reader = BufReader::new(input);
        let mut output = String::new();

        for line in reader.lines() {
            let line = line?;

            if !self.rx.is_match(&line) {
                output.push_str(&line);
                output.push('\n');
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_filter_string_1() {
        let sut = FileFilter::from("line").unwrap();
        assert_eq!(&sut.string(sample()), "The Final Line");
    }

    #[test]
    fn test_filter_string_2() {
        let sut = FileFilter::from("line$").unwrap();
        assert_eq!(
            &sut.string(sample()),
            indoc! { "
                first line
                a third line
                The Final Line"
            }
        );
    }

    fn sample() -> &'static str {
        indoc! { "
            first line
            line #2
            a third line
            The Final Line"
        }
    }
}
