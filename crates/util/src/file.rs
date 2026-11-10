use crate::http;
use crate::users_and_groups;
use crate::{checksum::Checksum, hash};
use anyhow::{Context, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::NamedUtf8TempFile;
use common::types::ApplyOpts;
use nix::sys::stat::{self, FileStat};
use nix::unistd::{Gid, Uid};
use os_types::FileMode;
use serde::Deserialize;
use std::os::unix::fs::PermissionsExt;
use std::{env, fs};
use url::Url;

#[derive(Debug)]
pub struct FileMetadata<'a> {
    pub group: &'a NameOrId,
    pub mode: &'a FileMode,
    pub owner: &'a NameOrId,
}

#[derive(Debug)]
pub struct LocalFile {
    path: Utf8PathBuf,
    _guard: Option<NamedUtf8TempFile>,
}

impl LocalFile {
    pub fn path(&self) -> &Utf8Path {
        &self.path
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum NameOrId {
    Name(String),
    Id(u32),
}

impl Default for NameOrId {
    fn default() -> Self {
        Self::Id(0)
    }
}

impl std::fmt::Display for NameOrId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameOrId::Name(s) => write!(f, "{}", s),
            NameOrId::Id(id) => write!(f, "{}", id),
        }
    }
}

/// FromSource handles those :from properties which can be a value or a URL.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(try_from = "&str")]
pub enum FromSource {
    // If we're in client/server mode, the front-end will have rewritten a file to a URL
    Url(Url),
    Path(Utf8PathBuf),
}

impl TryFrom<&str> for FromSource {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        if let Ok(url) = Url::parse(value) {
            Ok(FromSource::Url(url))
        } else {
            Ok(FromSource::Path(Utf8PathBuf::from(value)))
        }
    }
}

/// If you have FromSource this will make sure you have a local copy of it, and check it with
/// a checksum if you have one.
pub fn local_or_remote(
    source: &FromSource,
    checksum: Option<&Checksum>,
    opts: &ApplyOpts,
) -> anyhow::Result<LocalFile> {
    let tempfile;

    let path = match source {
        FromSource::Path(path) => {
            if let Some(checksum) = checksum {
                let actual = hash::of_file(path, &checksum.algorithm)?;

                ensure!(
                    checksum.resolve_and_compare(&actual, None)?,
                    "incorrect checksum for {path}"
                );
            }
            LocalFile {
                path: path.clone(),
                _guard: None,
            }
        }
        FromSource::Url(url) => {
            tempfile = NamedUtf8TempFile::new_in("/var/tmp")?;
            // url_to_disk checks the checksum itself
            http::url_to_disk(
                &http::RemoteFileCopy {
                    url,
                    path: tempfile.path(),
                    checksum,
                    backup_suffix: None,
                },
                opts,
            )?;

            LocalFile {
                path: tempfile.path().to_owned(),
                _guard: Some(tempfile),
            }
        }
    };

    Ok(path)
}

pub fn ensure_metadata(
    path: &Utf8Path,
    md: FileMetadata,
    opts: &ApplyOpts,
) -> anyhow::Result<bool> {
    // If it's a create noop, the object will not exist
    if !path.exists() {
        return Ok(false);
    }

    let metadata = metadata(path)?;
    let new_uid = new_uid(md.owner, &metadata)?;
    let new_gid = new_gid(md.group, &metadata)?;
    let mut changed = false;

    if new_uid.is_some() {
        changed = true;
    }

    if new_gid.is_some() {
        changed = true;
    }

    if changed && !opts.noop {
        set_user(path, new_uid, new_gid, opts)?;
    }

    // We don't care about file type because we're trying to be generic, so mask off the file
    // type bit.
    let current_mode = FileMode::from_u32(metadata.st_mode as u32 & 0o7777);

    if current_mode != *md.mode {
        changed = true;

        if !opts.noop {
            set_mode(path, &current_mode, md.mode, opts)
                .with_context(|| format!("failed to set mode {} on {path}", md.mode))?;
        }
    }

    Ok(changed)
}

pub fn metadata(path: &Utf8Path) -> anyhow::Result<FileStat> {
    let metadata = stat::stat(path.as_std_path())
        .with_context(|| format!("failed to get metadata for {path}"))?;

    Ok(metadata)
}

fn new_uid(desired_owner: &NameOrId, metadata: &FileStat) -> anyhow::Result<Option<Uid>> {
    let current_uid: Uid = metadata.st_uid.into();
    let desired_uid = users_and_groups::owner_from(desired_owner)?;

    if current_uid == desired_uid {
        tracing::debug!("owner is correct");
        Ok(None)
    } else {
        Ok(Some(desired_uid))
    }
}

fn new_gid(desired_group: &NameOrId, metadata: &FileStat) -> anyhow::Result<Option<Gid>> {
    let current_gid: Gid = metadata.st_gid.into();
    let desired_gid = users_and_groups::group_from(desired_group)
        .with_context(|| format!("failed to get group of {desired_group}"))?;

    if current_gid == desired_gid {
        tracing::debug!("group is correct");
        Ok(None)
    } else {
        Ok(Some(desired_gid))
    }
}

fn set_user(
    path: &Utf8Path,
    uid: Option<Uid>,
    gid: Option<Gid>,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    if let Some(uid) = uid {
        if let Some(gid) = gid {
            tracing::info!("{path}: setting user: group to {}:{}", uid, gid);
        } else {
            tracing::info!("{path}: setting user to {}", uid);
        }
    } else if let Some(gid) = gid {
        tracing::info!("{path}: setting group to {}", gid);
    } else {
        bail!("UID and GID both empty");
    }

    if !opts.noop {
        nix::unistd::chown(path.as_std_path(), uid, gid)
            .with_context(|| format!("failed to change owner of {}", path))?;
    }

    Ok(())
}

fn set_mode(
    path: &Utf8Path,
    current_mode: &FileMode,
    desired_mode: &FileMode,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    tracing::info!("{path}: changing mode {current_mode} -> {desired_mode}");

    if !opts.noop {
        fs::set_permissions(path, fs::Permissions::from_mode(desired_mode.as_u32()))
            .with_context(|| format!("failed to set permissions on {path}"))?;
    }

    Ok(())
}

pub fn current_dir() -> anyhow::Result<Utf8PathBuf> {
    let cwd = env::current_dir().context("cannot get current dir from env")?;
    Utf8PathBuf::try_from(cwd).map_err(|e| e.into())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::checksum::HashValue;
    use hash::HashAlgorithm;
    use httpmock::prelude::*;
    use snltest::fixture;

    #[test]
    fn test_local_or_remote_local_with_good_checksum() {
        let checksum = Checksum {
            algorithm: HashAlgorithm::Sha512,
            value: HashValue::Literal(
                "bd283809bc808e1b7733da97f5f035b6beaa7514a500d028743767733e4abf54fda3556120f115391492cbd0bd3dbf0fc11946ad657778e26e53a3cbec8ee53a".to_owned()
            ),
        };

        let result = local_or_remote(
            &FromSource::Path(fixture!("file/flat-archive.tar.gz")),
            Some(&checksum),
            &ApplyOpts::default(),
        );

        let ok = result.unwrap();
        assert!(&ok.path.exists());
    }

    #[test]
    fn test_local_or_remote_local_without_checksum() {
        let result = local_or_remote(
            &FromSource::Path(fixture!("file/flat-archive.tar.gz")),
            None,
            &ApplyOpts::default(),
        );

        let ok = result.unwrap();
        assert!(&ok.path.exists());
    }

    #[test]
    fn test_local_or_remote_local_with_bad_checksum() {
        let checksum = Checksum {
            algorithm: HashAlgorithm::Sha256,
            value: HashValue::Literal(
                "0000000000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            ),
        };

        let result = local_or_remote(
            &FromSource::Path(fixture!("file/flat-archive.tar.gz")),
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

    #[test]
    fn test_local_or_remote_remote_with_good_checksum() {
        let server = MockServer::start();
        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/flat-archive.tar.gz");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body_from_file(fixture!("file/flat-archive.tar.gz"));
        });

        let checksum = Checksum {
            algorithm: HashAlgorithm::Sha512,
            value: HashValue::Literal(
                "bd283809bc808e1b7733da97f5f035b6beaa7514a500d028743767733e4abf54fda3556120f115391492cbd0bd3dbf0fc11946ad657778e26e53a3cbec8ee53a".to_owned()
            ),
        };

        let result = local_or_remote(
            &FromSource::Url(Url::parse(&server.url("/flat-archive.tar.gz")).unwrap()),
            Some(&checksum),
            &ApplyOpts::default(),
        );

        let ok = result.unwrap();
        assert!(&ok.path.exists());
        conf_mock.assert();
    }

    #[test]
    fn test_local_or_remote_remote_without_checksum() {
        let server = MockServer::start();
        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/flat-archive.tar.gz");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body_from_file(fixture!("file/flat-archive.tar.gz"));
        });

        let result = local_or_remote(
            &FromSource::Url(Url::parse(&server.url("/flat-archive.tar.gz")).unwrap()),
            None,
            &ApplyOpts::default(),
        );

        let ok = result.unwrap();
        assert!(&ok.path.exists());
        conf_mock.assert();
    }

    #[test]
    fn test_local_or_remote_remote_with_bad_checksum() {
        let server = MockServer::start();
        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/flat-archive.tar.gz");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body_from_file(fixture!("file/flat-archive.tar.gz"));
        });

        let checksum = Checksum {
            algorithm: HashAlgorithm::Sha256,
            value: HashValue::Literal(
                "0000000000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            ),
        };

        let result = local_or_remote(
            &FromSource::Url(Url::parse(&server.url("/flat-archive.tar.gz")).unwrap()),
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

        conf_mock.assert();
    }
}
