use crate::file::actions;
use anyhow::{Context, bail, ensure};
use camino::Utf8PathBuf;
use common::constants::{ONE_RESOURCE_NO_CHANGE, ONE_RESOURCE_ONE_CHANGE, OPENSSL_BIN};
use common::types::{ApplyOpts, ApplySummary};
use os_types::GurpId;
use serde::Deserialize;
use std::fmt::Debug;
use std::fs;
use url::Url;
use util::{hash, http};

const SYSTEM_CERT_DIR: &str = "/etc/ssl/certs";

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "kebab-case")]
pub struct SystemCertEnsure {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: String,
    pub from: Option<Utf8PathBuf>,
    pub from_url: Option<Url>,
    pub content: Option<String>,
    #[serde(default)]
    pub url_is_server: bool,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SystemCertRemove {
    #[serde(rename = "_id")]
    pub id: GurpId,
    pub name: String,
}

impl SystemCertEnsure {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let mut changed = false;

        ensure!(
            self.single_source(),
            "system-cert '{}' must have exactly one of :content, :from, or :from-url",
            self.name
        );

        let target_path = Utf8PathBuf::from(SYSTEM_CERT_DIR).join(&self.name);

        // It's only a cert: whatever the source, read it into memory

        let source_str = if let Some(source) = &self.from {
            fs::read_to_string(source)
                .with_context(|| format!("failed to read cert from {source}"))?
        } else if let Some(url) = &self.from_url {
            http::url_to_string(url).with_context(|| format!("failed to fetch cert from {url}"))?
        } else if let Some(content) = &self.content {
            content.clone()
        } else {
            bail!("no source for system-cert {}", self.name);
        };

        if target_path.exists() {
            if hash::of_string(&source_str) == hash::of_file(&target_path)? {
                tracing::debug!("no change to system-cert at {target_path}");
            } else {
                changed = true;
                tracing::info!("updating content of system-cert at {target_path}");
            }
        } else {
            changed = true;
            tracing::info!("creating new cert at {target_path}");
        }

        if changed {
            actions::write_text_file(&target_path, &source_str, None, opts)?;
            rehash_cert_dir(opts)?;
            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }

    fn single_source(&self) -> bool {
        [
            self.content.as_ref().map(|_| ()),
            self.from_url.as_ref().map(|_| ()),
            self.from.as_ref().map(|_| ()),
        ]
        .iter()
        .filter(|v| v.is_some())
        .count()
            == 1
    }
}

impl SystemCertRemove {
    pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
        let path = Utf8PathBuf::from(SYSTEM_CERT_DIR).join(&self.name);

        if path.exists() {
            tracing::info!("removing cert at {path}");

            if !opts.noop {
                fs::remove_file(&path)
                    .with_context(|| format!("failed to remove system-cert at {path}"))?;
            }

            rehash_cert_dir(opts)?;

            Ok(ONE_RESOURCE_ONE_CHANGE)
        } else {
            tracing::debug!("no system cert at {path}");
            Ok(ONE_RESOURCE_NO_CHANGE)
        }
    }
}

fn rehash_cert_dir(opts: &ApplyOpts) -> anyhow::Result<()> {
    cmd_change_or_noop!(opts, OPENSSL_BIN, "rehash").context("failed to run openssl rehash")?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use camino::Utf8PathBuf;
    use pretty_assertions::assert_eq;
    use tester::{deserialized_example, janet2json, raw_example};

    #[test]
    fn test_deserialize_system_cert_from_file() {
        let expected = SystemCertEnsure {
            id: GurpId::new("/NO-ROLE/system-cert/from-file").unwrap(),
            name: "from-file".to_owned(),
            from: Some(Utf8PathBuf::from("/example/dir/files/ca/example")),
            from_url: None,
            content: None,
            url_is_server: false,
        };

        let json = janet2json(&indoc::formatdoc! { r#"
            (do
                (setdyn :gurp-config-root "/example/dir")
                {})
            "#,
            raw_example("system-cert/ensure-from-file.janet"),
        });

        let actual = serde_json::from_str(&json).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_deserialize_system_cert_from_url() {
        assert_eq!(
            SystemCertEnsure {
                id: GurpId::new("/NO-ROLE/system-cert/from-url").unwrap(),
                name: "from-url".to_owned(),
                from: None,
                from_url: Some(Url::parse("https://cert-service/api").unwrap()),
                content: None,
                url_is_server: false,
            },
            deserialized_example("system-cert/ensure-from-url.janet")
        );
    }

    #[test]
    fn test_deserialize_remove_system_cert() {
        assert_eq!(
            SystemCertRemove {
                id: GurpId::new("/NO-ROLE/system-cert/unwanted-cert").unwrap(),
                name: "unwanted-cert".to_owned(),
            },
            deserialized_example("system-cert/remove-cert.janet")
        );
    }

    #[test]
    fn test_not_exactly_one_source_fails() {
        let file_and_url = SystemCertEnsure {
            id: GurpId::new("/NO-ROLE/system-cert/irrelevant").unwrap(),
            name: "bad-input".to_owned(),
            from: Some(Utf8PathBuf::from("/dir/ca/example")),
            from_url: Some(Url::parse("https://cert-service/api").unwrap()),
            content: None,
            url_is_server: false,
        };

        assert!(file_and_url.apply(&ApplyOpts::default()).is_err());

        let file_and_content = SystemCertEnsure {
            id: GurpId::new("/NO-ROLE/system-cert/irrelevant").unwrap(),
            name: "bad-input".to_owned(),
            from: Some(Utf8PathBuf::from("/dir/ca/example")),
            from_url: None,
            content: Some(r"---BEGIN CERT---\nblah blah\---END CERT---".to_owned()),
            url_is_server: false,
        };

        assert!(file_and_content.apply(&ApplyOpts::default()).is_err());

        let no_source = SystemCertEnsure {
            id: GurpId::new("/NO-ROLE/system-cert/irrelevent").unwrap(),
            name: "example".to_owned(),
            from: None,
            from_url: None,
            content: None,
            url_is_server: false,
        };

        assert!(no_source.apply(&ApplyOpts::default()).is_err());
    }
}
