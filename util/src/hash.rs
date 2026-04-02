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
    let mut fh = fs::File::open(path)?;
    std::io::copy(&mut fh, &mut hasher)?;
    Ok(hasher.finalize())
}

pub fn for_remote_file(url: &str) -> anyhow::Result<String> {
    let hash_url = url.replace("/file/", "/file-hash/");
    Ok(ureq::get(hash_url).call()?.into_body().read_to_string()?)
}

pub fn for_remote_filtered_file(url: &str, pattern: &str) -> anyhow::Result<String> {
    let hash_url = format!(
        "{}?{}",
        url.replace("/file/", "/file-hash-filtered/"),
        pattern
    );
    Ok(ureq::get(hash_url).call()?.into_body().read_to_string()?)
}
