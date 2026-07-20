use anyhow::Context;
use blake3::Hash;
use camino::Utf8Path;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};

/// Returns a blake3 hash of a byte array
pub fn of_bytes(bytes: &[u8]) -> Hash {
    blake3::hash(bytes)
}

/// Returns a blake3 hash of a string
pub fn of_string(user_string: &str) -> Hash {
    of_bytes(user_string.as_bytes())
}

/// Returns a blake3 hash of an open file handle
pub fn of_reader(mut r: impl Read) -> anyhow::Result<blake3::Hash> {
    let mut hasher = blake3::Hasher::new();
    io::copy(&mut r, &mut hasher)?;
    Ok(hasher.finalize())
}

/// Returns a blake3 hash for a local file
pub fn of_file(path: &Utf8Path) -> anyhow::Result<blake3::Hash> {
    let file = File::open(path).with_context(|| format!("failed to open {path}"))?;
    of_reader(file)
}

/// Requests and returns a blake3 file hash from a Gurp server
pub fn for_file_on_server(url: &str) -> anyhow::Result<String> {
    let hash_url = url.replace("/file/", "/file-hash/");
    fetch_hash_from_server(&hash_url)
}

/// Requests and returns a blake3 has for a file on a Gurp server after applying a filter
pub fn for_filtered_file_on_server(url: &str, pattern: &str) -> anyhow::Result<String> {
    fetch_hash_from_server(&format!(
        "{}?{}",
        url.replace("/file/", "/file-hash-filtered/"),
        pattern
    ))
}

/// Gets a Blake3 hash from the Gurp server
fn fetch_hash_from_server(hash_url: &str) -> anyhow::Result<String> {
    tracing::debug!("fetching hash: {hash_url}");

    ureq::get(hash_url)
        .call()
        .with_context(|| format!("failed to download {hash_url}"))?
        .into_body()
        .read_to_string()
        .with_context(|| format!("cannot turn hash from {hash_url} into string"))
}

/// Returns a sha256 hash of a local file handle
pub fn sha256_of_reader(mut reader: impl Read) -> anyhow::Result<String> {
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = reader.read(&mut buf).context("sha256 buf read failed")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Returns a sha256 hash of a local file
pub fn sha256_of_file(path: &Utf8Path) -> anyhow::Result<String> {
    let file = File::open(path).with_context(|| format!("failed to open {path}"))?;
    sha256_of_reader(file).with_context(|| format!("failed to get sha256 of {path}"))
}

#[cfg(test)]
mod test {
    use super::*;
    use tester::fixture;

    #[test]
    fn test_hash_of_string() {
        assert_eq!(
            "2fec886c2436e97948e0d75c80bcccf6beefa05a3aea2353f4068513d65ec485".to_owned(),
            of_string("merp merp").to_string()
        );
    }

    #[test]
    fn test_hash_of_file() {
        assert_eq!(
            "40a2c4e17aa9abec2dd26709c045190b959922b5fffb4ff676568225c0525eca",
            of_file(&fixture("file-filter-test")).unwrap().to_string()
        );
    }
}
