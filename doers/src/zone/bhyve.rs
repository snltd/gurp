use crate::zone::cloudinit;
use crate::zone::config::{GurpZoneBhyve, GurpZoneConfig, GurpZoneFilesystem};
use crate::zone::constants::READINESS_WAIT_INTERVAL;
use anyhow::{Context, bail, ensure};
use camino::Utf8PathBuf;
use common::prelude::*;
use portable_pty::{CommandBuilder, native_pty_system};
use std::fs;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
use util::http;

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
    tracing::info!("Running bhyve pre_install");

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

    if bhyve_config.cloudinit_file.is_some() && bhyve_config.cloudinit_struct.is_some() {
        bail!("bhyve requires at most one of :cloudinit-file or :cloudinit-struct");
    }

    if bhyve_config.cloudinit_file.is_some() || bhyve_config.cloudinit_struct.is_some() {
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

// So far as I can tell, the only way to configure a bhyve zone is to use cloudinit. And so far
// as I can tell, the only way to do that is to make a fake CD-ROM ISO image, and temporarily
// attach it to the zone.

pub fn wait_for_readiness(zone: &str) -> anyhow::Result<bool> {
    //
    // It's hard to know when a bhyve zone is fully booted. Here we look for a login prompt,
    // which seems universal across distributions. You somtimes have to hit return to get one
    // though.
    //
    // zlogin requires a PTY, hence the use of portable_pty. We scan the raw bytes of what
    // zlogin sees in a separate thread, because that was the only way I could avoid choking
    // on non-UTF8 data.
    //
    // When the console mentions ttyS0, we "press return". When we see the login prompt, we
    // send an escape to end the console session, then pass true back to the main thread and
    // the function returns. If we hit EOF before that, we send back false.
    //
    // I might replace this all of this with a ping...
    //
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(Default::default())?;

    tracing::info!("Waiting for zone to be ready");

    let mut cmd = CommandBuilder::new(ZLOGIN_BIN);
    cmd.arg("-C");
    cmd.arg(zone);

    let mut child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let mut writer = pair.master.take_writer()?;
    let mut reader = pair.master.try_clone_reader()?;

    let (tx, rx) = mpsc::channel::<bool>();

    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        let mut window = Vec::new();

        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }

            window.extend_from_slice(&buf[..n]);
            if window.len() > 4096 {
                window.drain(..window.len() - 4096);
            }

            if window.windows(5).any(|w| w == b"ttyS0") {
                tracing::debug!("Seen ttyS0; sending CR");
                let _ = writeln!(writer);
            }

            if window.windows(7).any(|w| w == b" login:") {
                tracing::debug!("Seen login prompt; sending ~.");
                let _ = writeln!(writer, "~.");
                let _ = tx.send(true);
                return;
            }

            sleep(READINESS_WAIT_INTERVAL);
        }

        let _ = tx.send(false);
    });

    let ready = rx.recv().unwrap_or(false);
    let _ = child.wait();
    Ok(ready)
}
