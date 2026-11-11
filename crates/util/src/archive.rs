use crate::checksum::Checksum;
use crate::file::{self, FromSource};
use camino::Utf8Path;
use common::constants::GTAR_BIN;
use common::types::ApplyOpts;

pub fn unpack(
    source: &FromSource,
    in_dir: &Utf8Path,
    checksum: Option<&Checksum>,
    opts: &ApplyOpts,
) -> anyhow::Result<bool> {
    let archive_file = file::local_or_remote(source, checksum, opts)?;
    cmd_change_or_noop!(opts, GTAR_BIN, "-C", in_dir, "-xf", archive_file.path()).map(|_| true)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::checksum::HashValue;
    use crate::hash::HashAlgorithm;
    use camino_tempfile::Utf8TempDir;
    use url::Url;

    fn test_remote_archive(checksum: Option<&Checksum>) -> bool {
        let archive_url = Url::parse(
            "https://github.com/snltd/gurp/releases/download/v2.5.0/gurp-v2.5.0-omnios.tar.gz",
        )
        .unwrap();

        let target_dir = Utf8TempDir::new().unwrap();

        assert!(
            unpack(
                &FromSource::Url(archive_url),
                target_dir.path(),
                checksum,
                &ApplyOpts::default(),
            )
            .unwrap()
        );

        target_dir.path().join("gurp").exists()
    }

    #[test]
    #[ignore]
    fn test_unpack_remote_no_checksum() {
        assert!(test_remote_archive(None));
    }

    #[test]
    #[ignore]
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
    #[ignore]
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
            &FromSource::Url(archive_url),
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
