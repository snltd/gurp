use crate::zfs;
use crate::zone::config::{GurpZoneBhyve, GurpZoneFilesystem, ImageSource, ZoneConfig};
use crate::zone::constants::READINESS_WAIT_TIMEOUT_BHYVE;
use crate::zone::{cloudinit, control};
use anyhow::{Context, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use common::constants::{
    IMG_CACHE_DIR, QEMU_IMG_BIN, ZFS_BIN, ZLOGIN_BIN, ZONEADM_BIN, ZONECFG_BIN, ZSTD_BIN,
};
use common::types::ApplyOpts;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use url::Url;
use util::http::{self, RemoteFileCopy};
use uuid::Uuid;

const BUFFER_SIZE: usize = 8192;
const WINDOW_SIZE: usize = 4096;

fn boot_device(config: &GurpZoneBhyve) -> Utf8PathBuf {
    Utf8PathBuf::from("/dev/zvol/rdsk").join(&config.boot_volume)
}

pub fn build_zone(
    zone: &str,
    config: &ZoneConfig,
    uuid: &Uuid,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let bhyve_config = config.bhyve.as_ref().context("no bhyve config")?;

    if let Some(image_source) = &config.image {
        let image_path = match image_source {
            ImageSource::Url(url) => image_on_disk(url, opts).context("cannot get bhyve image")?,
            ImageSource::Path(path) => path.to_owned(),
            ImageSource::Name(_) => bail!("image names are not supported for bhyve zones"),
        };

        write_image_to_boot_disk(
            &image_path,
            &bhyve_config.boot_volume,
            &bhyve_config.image_format,
        )
        .context("failed to write boot image")?;
    }

    if bhyve_config.has_cloudinit() {
        cloudinit::setup(bhyve_config, &cloudinit::iso_path(uuid), opts)?;
    }

    cmd_output!(ZONEADM_BIN, "-z", zone, "install")?;

    if config.boot_after_install {
        control::boot_zone(zone)?;
    }

    if bhyve_config.wait_for_boot {
        wait_for_readiness(zone, uuid)?;
    }

    if bhyve_config.has_cloudinit() {
        remove_cloudinit_config(zone)?;
    }

    Ok(())
}

pub fn zone_config(config: &GurpZoneBhyve, uuid: &Uuid) -> String {
    let mut ret = String::new();

    ret.push_str(zone_device!(boot_device(config)));
    ret.push_str(zone_attr!("bootrom", "string", config.boot_rom));
    ret.push_str(zone_attr!("bootdisk", "string", config.boot_volume));
    ret.push_str(zone_attr!("vcpus", "string", config.vcpus));
    ret.push_str(zone_attr!("ram", "string", config.ram));
    ret.push_str(zone_attr!("acpi", "string", config.acpi));

    if config.has_cloudinit() {
        let iso_path = cloudinit::iso_path(uuid);
        ret.push_str(zone_attr!("cdrom", "string", iso_path));
        ret.push_str(&zone_fs!(GurpZoneFilesystem {
            dir: iso_path.clone(),
            special: iso_path.clone(),
            fs_type: "lofs".to_owned(),
            options: Some(vec!["ro".to_owned()])
        }));
    }

    ret
}

fn image_on_disk(image: &Url, opts: &ApplyOpts) -> anyhow::Result<Utf8PathBuf> {
    let cached_path = image_cache_filename(image)?;

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

fn write_image_to_boot_disk(
    image_path: &Utf8Path,
    zvol: &str,
    image_format: &Option<String>,
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
            stream_img_to_boot_zvol(image_path, zvol)
        }
        "raw" => {
            tracing::debug!("no work needed on raw files");
            write_img_to_boot_zvol(image_path, zvol)
        }
        other_format => {
            let raw_image = image_path.with_extension("raw");
            convert_image_to_raw(image_path, &raw_image, other_format)?;
            write_img_to_boot_zvol(&raw_image, zvol)
        }
    }
}

fn image_cache_filename(url: &Url) -> anyhow::Result<Utf8PathBuf> {
    let basename = url
        .path()
        .split("/")
        .last()
        .with_context(|| format!("unable to parse image URL {url}"))?;
    let cache_dir = Utf8PathBuf::from(IMG_CACHE_DIR);
    Ok(cache_dir.join(basename))
}

fn remove_cloudinit_config(zone: &str) -> anyhow::Result<()> {
    tracing::debug!("removing cloudinit cdrom from zone config");
    // It's safe to do this here. The config won't be re-read until the zone
    // boots
    let _ = cmd_output!(ZONECFG_BIN, "-z", zone, "remove attr name=cdrom")
        .with_context(|| format!("failed to remove cdrom attr from zone {}", zone))?;

    tracing::debug!("removing cloudinit lofs from zone config");
    let _ = cmd_output!(ZONECFG_BIN, "-z", zone, "remove fs type=lofs")
        .with_context(|| format!("failed to remove cdrom lofs from zone {}", zone))?;

    Ok(())
}

// For ZFS images
fn stream_img_to_boot_zvol(raw_img_path: &Utf8Path, zvol: &str) -> anyhow::Result<()> {
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

    if !zstd_status.success() {
        bail!("zstd failed: {zstd_status}");
    }

    if !zfs_status.success() {
        bail!("zfs receive failed: {zfs_status}");
    }

    Ok(())
}

// Using std::fs::copy took forever. Use a big buffer.
fn write_img_to_boot_zvol(raw_img_path: &Utf8Path, zvol: &str) -> anyhow::Result<()> {
    let zvol = Utf8PathBuf::from("/dev/zvol/dsk").join(zvol);
    ensure!(zvol.exists(), "zvol {zvol} not found");

    tracing::debug!("writing {} to {}", raw_img_path, zvol);

    let src =
        fs::File::open(raw_img_path).with_context(|| format!("failed to open {raw_img_path}"))?;
    let dst = fs::OpenOptions::new()
        .write(true)
        .open(&zvol)
        .with_context(|| format!("failed to open {zvol}"))?;

    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, src);
    let mut writer = BufWriter::with_capacity(16 * 1024 * 1024, dst);

    std::io::copy(&mut reader, &mut writer)
        .with_context(|| format!("failed to write image from {raw_img_path} to {zvol}"))?;

    writer.flush().context("failed to flush writer")?;
    Ok(())
}

fn convert_image_to_raw(
    img_path: &Utf8Path,
    raw_img_path: &Utf8Path,
    img_format: &str,
) -> anyhow::Result<()> {
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

pub fn wait_for_readiness(zone: &str, uuid: &Uuid) -> anyhow::Result<bool> {
    tracing::info!("Waiting for zone '{zone}' to be ready");

    let console_log_dir = Utf8PathBuf::from("/var/tmp");
    let console_log_file = console_log_dir.join(format!("gurp-bhyve-{uuid}.log"));

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize::default())
        .context("Failed to create PTY")?;

    let mut cmd = CommandBuilder::new(ZLOGIN_BIN);
    cmd.arg("-C");
    cmd.arg(zone);

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .context("Failed to start zlogin")?;

    drop(pair.slave);

    let writer = pair
        .master
        .take_writer()
        .context("Failed to get PTY writer")?;

    let reader = pair
        .master
        .try_clone_reader()
        .context("Failed to get PTY reader")?;

    let (result_tx, result_rx) = mpsc::channel();

    tracing::info!("Logging console output to {console_log_file}");

    thread::spawn(move || {
        let result = monitor_console(reader, writer, &console_log_file);
        let _ = result_tx.send(result);
    });

    let result = result_rx.recv_timeout(READINESS_WAIT_TIMEOUT_BHYVE)?;
    let _ = child.kill();
    let _ = child.wait();

    result
}

fn monitor_console(
    mut reader: Box<dyn Read + Send>,
    mut writer: Box<dyn Write + Send>,
    log_path: &Utf8PathBuf,
) -> anyhow::Result<bool> {
    let mut buf = vec![0u8; BUFFER_SIZE];
    let mut window = Vec::new();
    let mut cr_sent = false;
    let mut login_prompt_seen = false;

    let log_file =
        File::create(log_path).with_context(|| format!("failed to open +w log at {log_path}"))?;
    let mut log_writer = BufWriter::new(log_file);

    let start_time = Instant::now();

    while start_time.elapsed() < READINESS_WAIT_TIMEOUT_BHYVE {
        match reader.read(&mut buf) {
            Ok(0) => {
                tracing::debug!("EOF reached on console");
                break;
            }
            Ok(n) => {
                let data = &buf[..n];

                if let Err(e) = log_writer.write_all(data) {
                    tracing::warn!("Failed to write to console log: {}", e);
                } else {
                    let _ = log_writer.flush();
                }

                if !data.is_empty() {
                    let preview = if data.len() > 100 { &data[..100] } else { data };
                    let preview_str = String::from_utf8_lossy(preview);
                    tracing::debug!("Console data: {:?}", preview_str);
                }

                window.extend_from_slice(data);
                if window.len() > WINDOW_SIZE {
                    window.drain(..window.len() - WINDOW_SIZE);
                }

                if let Some(result) =
                    scan_output(&window, &mut writer, &mut cr_sent, &mut login_prompt_seen)
                {
                    let _ = log_writer.flush();
                    return Ok(result);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(e) => {
                bail!("Read error: {}", e);
            }
        }

        thread::sleep(Duration::from_millis(50));
    }

    let _ = log_writer.flush();

    bail!("Timeout reached waiting for zone readiness");
}

fn scan_output(
    window: &[u8],
    writer: &mut Box<dyn Write + Send>,
    cr_sent: &mut bool,
    login_prompt_seen: &mut bool,
) -> Option<bool> {
    if find_pattern(window, b"Zone halted") {
        tracing::debug!("Zone halted detected");
        hangup(writer);
        return Some(false);
    }

    if !*cr_sent && find_pattern(window, b"ttyS0") {
        tracing::debug!("ttyS0 detected; sending CR");
        hit_return(writer);
        *cr_sent = true;
    }

    let login_patterns: &[&[u8]] = &[
        b"login:",
        b"Login:",
        b"username:",
        b"Username:",
        b"cloud-init is configuring this system",
    ];

    for pattern in login_patterns {
        if find_pattern(window, pattern) && !*login_prompt_seen {
            tracing::debug!("Login prompt detected",);
            *login_prompt_seen = true;

            thread::sleep(Duration::from_millis(500));
            hangup(writer);
            return Some(true);
        }
    }

    None
}

fn find_pattern(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }

    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn hit_return(writer: &mut Box<dyn Write + Send>) {
    let _ = writer.write_all(b"\r");
    let _ = writer.flush();
}

fn hangup(writer: &mut Box<dyn Write + Send>) {
    let _ = writer.write_all(b"~.\r");
    let _ = writer.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_pattern() {
        assert!(find_pattern(b"hello world", b"world"));
        assert!(find_pattern(b"ubuntu login: ", b" login:"));
        assert!(!find_pattern(b"hello", b"world"));
        assert!(find_pattern(b"Zone halted", b"Zone halted"));
    }

    #[test]
    fn test_check_console_patterns_login() {
        let mut writer: Box<dyn Write + Send> = Box::new(Vec::new());
        let mut cr_sent = false;
        let mut login_seen = false;

        let window = b"Welcome to Linux\nubuntu login: ";
        let result = scan_output(window, &mut writer, &mut cr_sent, &mut login_seen);

        assert!(matches!(result, Some(true)));
        assert!(login_seen);
    }

    #[test]
    fn test_check_console_patterns_halted() {
        let mut writer: Box<dyn Write + Send> = Box::new(Vec::new());
        let mut cr_sent = false;
        let mut login_seen = false;

        let window = b"Shutting down\nZone halted\n";
        let result = scan_output(window, &mut writer, &mut cr_sent, &mut login_seen);

        assert!(matches!(result, Some(false)));
    }

    #[test]
    fn test_check_console_patterns_ttys0() {
        let mut writer: Box<dyn Write + Send> = Box::new(Vec::new());
        let mut cr_sent = false;
        let mut login_seen = false;

        let window = b"Starting ttyS0...";
        let result = scan_output(window, &mut writer, &mut cr_sent, &mut login_seen);

        assert!(result.is_none());
        assert!(cr_sent);
    }
}
