use anyhow::Context;
use blake3::Hash;
use camino::Utf8Path;
use std::fs;

pub fn of_bytes(bytes: &[u8]) -> Hash {
    blake3::hash(bytes)
}

pub fn of_string(user_string: &str) -> Hash {
    of_bytes(user_string.as_bytes())
}

pub fn of_file(path: &Utf8Path) -> anyhow::Result<Hash> {
    let mut hasher = blake3::Hasher::new();
    let mut fh = fs::File::open(path).with_context(|| format!("cannot open {path}"))?;
    std::io::copy(&mut fh, &mut hasher)
        .with_context(|| format!("failed to read hash of {path}"))?;
    Ok(hasher.finalize())
}

// Requests a file hash from a Gurp server
pub fn for_remote_file(url: &str) -> anyhow::Result<String> {
    let hash_url = url.replace("/file/", "/file-hash/");
    fetch_hash(&hash_url)
}

// Requests a file hash from a Gurp server
pub fn for_remote_filtered_file(url: &str, pattern: &str) -> anyhow::Result<String> {
    let hash_url = format!(
        "{}?{}",
        url.replace("/file/", "/file-hash-filtered/"),
        pattern
    );
    fetch_hash(&hash_url)
}

fn fetch_hash(hash_url: &str) -> anyhow::Result<String> {
    ureq::get(hash_url)
        .call()
        .with_context(|| format!("failed to download {hash_url}"))?
        .into_body()
        .read_to_string()
        .context("cannot turn fetched body into string")
}

// Users supply SHA256 checksums
pub fn sha256_of_file(path: &Utf8Path) -> anyhow::Result<String> {
    sha256::try_digest(path).with_context(|| format!("failed to get SHA256 of {path}"))
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
