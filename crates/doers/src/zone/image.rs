use crate::zfs;
use crate::zone::config::ImageSource;
use anyhow::{Context, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use common::constants::{IMG_CACHE_DIR, QEMU_IMG_BIN, ZFS_BIN, ZSTD_BIN};
use common::types::ApplyOpts;
use std::fs;
use std::io::{BufReader, BufWriter, Write};
use std::process::{Command, Stdio};
use url::Url;
use util::http::{self, RemoteFileCopy};

// Fetches and caches images and bios files.

const BUFFER_CAP: usize = 16777216;

pub fn path(image_source: &ImageSource, opts: &ApplyOpts) -> anyhow::Result<Utf8PathBuf> {
    let path = match image_source {
        ImageSource::Url(url) => {
            from_local_cache(url, opts).with_context(|| format!("cannot get image from {url}"))?
        }
        ImageSource::Path(path) => path.to_owned(),
        ImageSource::Name(_) => bail!("image names are not supported for emulation zones"),
    };

    Ok(path)
}

pub fn write_to_boot_zvol(
    image_path: &Utf8Path,
    boot_zvol: &str,
    image_format: Option<&str>,
) -> anyhow::Result<()> {
    ensure!(image_path.exists(), "Image file not found: {image_path}");

    let image_format = if let Some(user_format) = image_format {
        user_format
    } else {
        image_path
            .extension()
            .with_context(|| format!("cannot determine image format for {image_path}"))?
    };

    match image_format {
        "zst" => {
            tracing::debug!("no work needed on zst files");
            zst_to_zvol(image_path, boot_zvol)
        }
        "raw" => {
            tracing::debug!("no work needed on raw files");
            raw_to_zvol(image_path, boot_zvol)
        }
        other_format => {
            let raw_image = image_path.with_extension("raw");
            to_raw(image_path, &raw_image, other_format)?;
            raw_to_zvol(&raw_image, boot_zvol)
        }
    }
}

fn from_local_cache(image: &Url, opts: &ApplyOpts) -> anyhow::Result<Utf8PathBuf> {
    let cached_path = local_cached_path(image)?;

    if !cached_path.exists() {
        tracing::debug!("No cached image file at {}", cached_path);
        tracing::info!("downloading image from {image}");
        http::url_to_disk(
            &RemoteFileCopy {
                url: image,
                path: &cached_path,
                backup_suffix: None,
                checksum: None,
            },
            opts,
        )?;
    }

    Ok(cached_path)
}

fn local_cached_path(url: &Url) -> anyhow::Result<Utf8PathBuf> {
    let basename = url
        .path()
        .split("/")
        .last()
        .with_context(|| format!("unable to parse image URL {url}"))?;

    let cache_dir = Utf8PathBuf::from(IMG_CACHE_DIR);

    Ok(cache_dir.join(basename))
}

// For ZFS images
fn zst_to_zvol(raw_img_path: &Utf8Path, zvol: &str) -> anyhow::Result<()> {
    if zfs::zfs_exists(zvol)? {
        tracing::info!("ZFS volume {zvol} exists: removing");
        zfs::remove_filesystem(zvol, &ApplyOpts::default())?;
    }

    let mut zstd = Command::new(ZSTD_BIN)
        .args(["-d", raw_img_path.as_str(), "--stdout"])
        .stdout(Stdio::piped())
        .spawn()?;

    let mut zfs = Command::new(ZFS_BIN)
        .args(["receive", zvol])
        .stdin(zstd.stdout.take().unwrap())
        .spawn()?;

    let zfs_status = zfs.wait()?;
    let zstd_status = zstd.wait()?;

    ensure!(zstd_status.success(), "zstd failed: {zstd_status}");

    ensure!(zfs_status.success(), "zfs receive failed: {zfs_status}");

    Ok(())
}

// Using std::fs::copy took forever. Use a big buffer.
fn raw_to_zvol(raw_img_path: &Utf8Path, zvol: &str) -> anyhow::Result<()> {
    let zvol = Utf8PathBuf::from("/dev/zvol/dsk").join(zvol);
    ensure!(zvol.exists(), "zvol {zvol} not found");

    tracing::debug!("writing {} to {}", raw_img_path, zvol);

    let src =
        fs::File::open(raw_img_path).with_context(|| format!("failed to open {raw_img_path}"))?;
    let dst = fs::OpenOptions::new()
        .write(true)
        .open(&zvol)
        .with_context(|| format!("failed to open {zvol}"))?;

    let mut reader = BufReader::with_capacity(BUFFER_CAP, src);
    let mut writer = BufWriter::with_capacity(BUFFER_CAP, dst);

    if let Err(e) = std::io::copy(&mut reader, &mut writer) {
        bail!("failed to write image from {raw_img_path} to {zvol}: {e}");
    }

    writer.flush().context("failed to flush writer")?;

    Ok(())
}

fn to_raw(img_path: &Utf8Path, raw_img_path: &Utf8Path, img_format: &str) -> anyhow::Result<()> {
    let qemu_img = Utf8PathBuf::from(QEMU_IMG_BIN);

    ensure!(
        qemu_img.exists(),
        "No {qemu_img}. You probably need to `pkg install qemu-img`",
    );

    tracing::info!("converting {} to raw image", img_path);

    let _ = cmd_output!(
        QEMU_IMG_BIN,
        "convert",
        "-f",
        img_format,
        "-O",
        "raw",
        img_path,
        &raw_img_path
    )
    .with_context(|| format!("failed to convert {img_path} to {img_format}"));

    Ok(())
}
