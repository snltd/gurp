use anyhow::Context;
use camino::Utf8Path;
use serde::Deserialize;
use sha2::{Digest, Sha256, Sha512};
use std::fs::File;
use std::io::{self, Read, Write};
use url::Url;

/// Gurp uses Blake3 internally, because it's fast and lightweight, but we offer other
/// algorithms for file checksums.

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum HashAlgorithm {
    Blake3,
    Sha256,
    Sha512,
}

enum ServerHashType {
    Raw,
    Filtered,
}

struct DigestWriter<D>(D);

impl<D: Digest> Write for DigestWriter<D> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn of_bytes(bytes: &[u8], algorithm: HashAlgorithm) -> String {
    match algorithm {
        HashAlgorithm::Blake3 => blake3::hash(bytes).to_string(),
        HashAlgorithm::Sha256 => hash_bytes_with::<Sha256>(bytes),
        HashAlgorithm::Sha512 => hash_bytes_with::<Sha512>(bytes),
    }
}

pub fn of_string(user_string: &str, algorithm: HashAlgorithm) -> String {
    of_bytes(user_string.as_bytes(), algorithm)
}

pub fn of_reader(mut reader: impl Read, algorithm: HashAlgorithm) -> anyhow::Result<String> {
    match algorithm {
        HashAlgorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            io::copy(&mut reader, &mut hasher)?;
            Ok(hasher.finalize().to_string())
        }
        HashAlgorithm::Sha256 => hash_reader_with::<Sha256>(reader),
        HashAlgorithm::Sha512 => hash_reader_with::<Sha512>(reader),
    }
}

pub fn of_file(path: &Utf8Path, algorithm: HashAlgorithm) -> anyhow::Result<String> {
    let file = File::open(path).with_context(|| format!("failed to open {path}"))?;
    of_reader(file, algorithm)
}

pub fn for_file_on_server(url: &Url) -> anyhow::Result<String> {
    fetch_hash_from_server(&server_hash_url(url, ServerHashType::Raw)?)
}

pub fn for_filtered_file_on_server(url: &Url, pattern: &str) -> anyhow::Result<String> {
    let mut url = server_hash_url(url, ServerHashType::Filtered)?;
    url.set_query(Some(pattern));
    fetch_hash_from_server(&url)
}

fn hash_bytes_with<D: Digest>(data: &[u8]) -> String {
    hex::encode(D::digest(data))
}

fn hash_reader_with<D: Digest>(mut reader: impl Read) -> anyhow::Result<String> {
    let mut writer = DigestWriter(D::new());
    io::copy(&mut reader, &mut writer).context("hash read failed")?;
    Ok(hex::encode(writer.0.finalize()))
}

/// Gets a Blake3 hash from the Gurp server
fn fetch_hash_from_server(hash_url: &Url) -> anyhow::Result<String> {
    tracing::debug!("fetching hash: {hash_url}");

    ureq::get(hash_url.as_str())
        .call()
        .with_context(|| format!("failed to download {hash_url}"))?
        .into_body()
        .read_to_string()
        .with_context(|| format!("cannot turn hash from {hash_url} into string"))
}

fn server_hash_url(url: &Url, segment: ServerHashType) -> anyhow::Result<Url> {
    let mut ret = url.clone();

    let hash_path: Vec<_> = url
        .path_segments()
        .with_context(|| format!("cannot compute hash URL from {url}"))?
        .map(|s| {
            if s == "file" {
                match segment {
                    ServerHashType::Raw => "file-hash",
                    ServerHashType::Filtered => "file-hash-filtered",
                }
            } else {
                s
            }
        })
        .collect();

    ret.set_path(&hash_path.join("/"));
    Ok(ret)
}

#[cfg(test)]
mod test {
    use super::*;
    use snltest::fixture;

    #[test]
    fn test_hash_of_string() {
        assert_eq!(
            "2fec886c2436e97948e0d75c80bcccf6beefa05a3aea2353f4068513d65ec485".to_owned(),
            of_string("merp merp", HashAlgorithm::Blake3).to_string()
        );
    }

    #[test]
    fn test_hash_of_file() {
        assert_eq!(
            "40a2c4e17aa9abec2dd26709c045190b959922b5fffb4ff676568225c0525eca",
            of_file(&fixture!("file-filter-test", HashAlgorith::Blake3))
                .unwrap()
                .to_string()
        );
    }

    #[test]
    fn test_server_hash_url() {
        assert_eq!(
            Url::parse("http://server.localnet/file-hash/test1").unwrap(),
            server_hash_url(
                &Url::parse("http://server.localnet/file/test1").unwrap(),
                ServerHashType::Raw
            )
            .unwrap()
        );
    }
}
