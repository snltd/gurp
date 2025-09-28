use crate::zone::cloudinit;
use crate::zone::config::{GurpZoneBhyve, GurpZoneConfig, GurpZoneFilesystem};
use crate::zone::constants::{READINESS_WAIT_INTERVAL, READINESS_WAIT_TIMEOUT};
use anyhow::{Context, bail, ensure};
use camino::Utf8PathBuf;
use common::prelude::*;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use util::http;

const BUFFER_SIZE: usize = 8192;
const WINDOW_SIZE: usize = 4096;

pub fn zone_config(config: &GurpZoneBhyve, iso_path: Option<Utf8PathBuf>) -> String {
    let mut ret = String::new();
    let boot_device = format!("/dev/zvol/rdsk/{}", config.boot_volume);

    ret.push_str(zone_device!(boot_device));
    ret.push_str(zone_attr!("bootrom", "string", "BHYVE_RELEASE"));
    ret.push_str(zone_attr!("bootdisk", "string", config.boot_volume));
    ret.push_str(zone_attr!("vcpus", "string", config.vcpus));
    ret.push_str(zone_attr!("ram", "string", config.ram));
    ret.push_str(zone_attr!("acpi", "string", "false"));

    if let Some(iso_path) = iso_path {
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

// Creating the volume is the ZFS doer/user's responsibility.
// If we're given a path, assume the user's gone to the effort of converting it to raw. If it's
// a URL, we'll do it for them.
//
pub fn pre_install(config: &GurpZoneConfig) -> anyhow::Result<()> {
    tracing::debug!("Running bhyve pre_install");

    let bhyve_config = config
        .bhyve
        .as_ref()
        .context("bhyve zone requested, but no config given")?;

    let image_raw_file: Utf8PathBuf;

    if bhyve_config.image_url.is_some() && bhyve_config.image_path.is_some() {
        bail!("bhyve requires exactly one of :image-path or :image-url");
    }

    if let Some(image_path) = &bhyve_config.image_path {
        ensure!(image_path.exists(), "Image file not found: {image_path}");
        image_raw_file = image_path.clone();
    } else if let Some(image_url) = &bhyve_config.image_url {
        let image_cache_file = image_cache_filename(image_url)?;

        let image_format = if let Some(user_format) = &bhyve_config.image_format {
            user_format
        } else {
            image_cache_file
                .extension()
                .context("cannot determine image format")?
        };

        image_raw_file = image_cache_file.with_extension("raw");

        if !image_raw_file.exists() {
            tracing::debug!("No cached raw file at {}", image_raw_file);

            if !image_cache_file.exists() {
                tracing::debug!("No cached image file at {}", image_cache_file);
                http::download_file(image_url, &image_cache_file)?;
            }

            convert_image_to_raw(&image_cache_file, &image_raw_file, image_format)?;
        }
    } else {
        bail!("Did not get bhyve :image-path or :image-url");
    }

    write_img_to_boot_zvol(&image_raw_file, &bhyve_config.boot_volume)?;

    if bhyve_config.cloudinit_files.is_some() || bhyve_config.cloudinit_struct.is_some() {
        cloudinit::setup(
            bhyve_config,
            &config
                .cloudinit_iso_file
                .borrow()
                .clone()
                .context("Could not borrow cloudinit_iso_file")?,
        )?;
    }

    Ok(())
}

fn image_cache_filename(url: &str) -> anyhow::Result<Utf8PathBuf> {
    let basename = url.split("/").last().context("unable to parse image URL")?;
    let cache_dir = Utf8PathBuf::from(IMG_CACHE_DIR);
    Ok(cache_dir.join(basename))
}

fn write_img_to_boot_zvol(raw_img_path: &Utf8PathBuf, vol: &str) -> anyhow::Result<()> {
    let zvol = Utf8PathBuf::from("/dev/zvol/dsk").join(vol);

    if !zvol.exists() {
        bail!("zvol {zvol} not found");
    }

    tracing::debug!("writing {} to {}", raw_img_path, zvol);
    fs::copy(raw_img_path, &zvol)?;
    Ok(())
}

fn convert_image_to_raw(
    img_path: &Utf8PathBuf,
    raw_img_path: &Utf8PathBuf,
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
    );

    Ok(())
}

pub fn wait_for_readiness(zone: &str) -> anyhow::Result<bool> {
    tracing::info!("Waiting for zone to be ready");

    let console_log_dir = Utf8PathBuf::from("/var/tmp");
    let console_log_file = console_log_dir.join(format!("{zone}.log"));

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

    let result = result_rx.recv_timeout(READINESS_WAIT_TIMEOUT + READINESS_WAIT_INTERVAL)?;
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

    let log_file = File::create(log_path)?;
    let mut log_writer = BufWriter::new(log_file);

    let start_time = Instant::now();

    while start_time.elapsed() < READINESS_WAIT_TIMEOUT {
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

    let login_patterns: &[&[u8]] = &[b" login:", b"login:", b"Login:", b"username:", b"Username:"];

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
