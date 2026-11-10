use crate::http;
use crate::{checksum::Checksum, hash};
use anyhow::ensure;
use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::NamedUtf8TempFile;
use common::constants::GTAR_BIN;
use common::types::ApplyOpts;
use serde::Deserialize;
use url::Url;

// If we're in client/server mode, the front-end will have rewritten a file to a URL
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(try_from = "&str")]
pub enum ArchiveSource {
    Url(Url),
    Path(Utf8PathBuf),
}

impl TryFrom<&str> for ArchiveSource {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        if let Ok(url) = Url::parse(value) {
            Ok(ArchiveSource::Url(url))
        } else {
            Ok(ArchiveSource::Path(Utf8PathBuf::from(value)))
        }
    }
}

pub fn unpack(
    source: &ArchiveSource,
    in_dir: &Utf8Path,
    checksum: Option<&Checksum>,
    opts: &ApplyOpts,
) -> anyhow::Result<bool> {
    let tempfile;

    let archive_file = match source {
        ArchiveSource::Path(path) => {
            if let Some(checksum) = checksum {
                let actual = hash::of_file(path, &checksum.algorithm)?;

                ensure!(
                    checksum.resolve_and_compare(&actual, None)?,
                    "incorrect checksum for archive"
                );
            }
            path.as_path()
        }
        ArchiveSource::Url(url) => {
            tempfile = NamedUtf8TempFile::new_in("/var/tmp")?;
            http::url_to_disk(
                &http::RemoteFileCopy {
                    url,
                    path: tempfile.path(),
                    checksum,
                    backup_suffix: None,
                },
                opts,
            )?;

            tempfile.path()
        }
    };

    cmd_change_or_noop!(opts, GTAR_BIN, "-C", in_dir, "-xf", archive_file).map(|_| true)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::checksum::HashValue;
    use camino_tempfile::Utf8TempDir;
    use hash::HashAlgorithm;

    fn test_remote_archive(checksum: Option<&Checksum>) -> bool {
        let archive_url = Url::parse(
            "https://github.com/snltd/gurp/releases/download/v2.5.0/gurp-v2.5.0-omnios.tar.gz",
        )
        .unwrap();

        let target_dir = Utf8TempDir::new().unwrap();

        assert!(
            unpack(
                &ArchiveSource::Url(archive_url),
                target_dir.path(),
                checksum,
                &ApplyOpts::default(),
            )
            .unwrap()
        );

        target_dir.path().join("gurp").exists()
    }

    #[test]
    fn test_unpack_remote_no_checksum() {
        assert!(test_remote_archive(None));
    }

    #[test]
    fn test_unpack_remote_good_checksum() {
        let checksum = Checksum {
            algorithm: HashAlgorithm::Sha256,
            value: HashValue::Literal(
                "402a8a3d0054ddb12eff3a2ff35269722f9555b4666d2ce1d69a08bab774a2df".to_owned(),
            ),
        };

        assert!(test_remote_archive(Some(&checksum)));
    }

    #[test]
    fn test_unpack_remote_bad_checksum() {
        let archive_url = Url::parse(
            "https://github.com/snltd/gurp/releases/download/v2.5.0/gurp-v2.5.0-omnios.tar.gz",
        )
        .unwrap();

        let checksum = Checksum {
            algorithm: HashAlgorithm::Sha256,
            value: HashValue::Literal(
                "0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            ),
        };

        let target_dir = Utf8TempDir::new().unwrap();

        let result = unpack(
            &ArchiveSource::Url(archive_url),
            target_dir.path(),
            Some(&checksum),
            &ApplyOpts::default(),
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .starts_with("incorrect checksum")
        );
    }
}
