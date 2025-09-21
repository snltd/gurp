use crate::zone::config::GurpZoneConfig;
use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use common::prelude::*;
use portable_pty::{CommandBuilder, native_pty_system};
use std::fs;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

pub fn bhyve_zone_config(config: &GurpZoneConfig) -> String {
    let mut ret = String::new();

    if let Some(bhyve_config) = &config.bhyve {
        ret.push_str(&format!(
            "add device\n\tset match=\"/dev/zvol/rdsk/{}\"\nend\n",
            bhyve_config.boot_volume
        ));

        ret.push_str(&zone_attr!("bootrom", "string", "BHYVE_RELEASE"));
        ret.push_str(&zone_attr!("bootdisk", "string", &bhyve_config.boot_volume));
        ret.push_str(&zone_attr!("vcpus", "string", &bhyve_config.vcpus));
        ret.push_str(&zone_attr!("ram", "string", &bhyve_config.ram));
        ret.push_str(&zone_attr!("acpi", "string", "false"));
    }

    ret
}

fn image_in_cache(_url: &str) -> anyhow::Result<Utf8PathBuf> {
    // TODO
    Ok(Utf8PathBuf::from(
        "/var/tmp/noble-server-cloudimg-amd64.img",
    ))
}

// Creating the volume is the ZFS doer/user's responsibility.
//
pub fn pre_install(config: &GurpZoneConfig) -> anyhow::Result<()> {
    tracing::info!("Running bhyve pre_install");

    let bhyve_config = config
        .bhyve
        .as_ref()
        .context("trying to configure bhyve, but no config")?;

    let img_file = image_in_cache(&bhyve_config.image_url)?;
    let img_format = bhyve_config.image_format.as_deref().unwrap_or("qcow");
    let raw_img_file = convert_to_raw(img_file, img_format)?;
    write_img_to_boot_zvol(&raw_img_file, &bhyve_config.boot_volume)?;

    if bhyve_config.cloudinit {
        let _cloudinit_dir = cloudinit_iso_dir(config);
    }
    Ok(())
}

fn write_img_to_boot_zvol(raw_img_path: &Utf8PathBuf, vol: &str) -> anyhow::Result<()> {
    let zvol = Utf8PathBuf::from("/dev/zvol/dsk").join(vol);

    if !zvol.exists() {
        bail!("zvol {zvol} not found");
    }

    tracing::debug!("writing {} to {}", raw_img_path, zvol);
    fs::copy(raw_img_path, zvol)?;
    Ok(())
}

fn convert_to_raw(path: Utf8PathBuf, img_format: &str) -> anyhow::Result<Utf8PathBuf> {
    if img_format == "raw" {
        Ok(path)
    } else {
        let raw_path = path.with_extension("raw");
        if raw_path.exists() {
            tracing::debug!("raw image found at {}", raw_path);
        } else {
            if !Utf8PathBuf::from(QEMU_IMG_BIN).exists() {
                bail!("No qemu-img found. You probably need to `pkg install qemu-img`");
            }
            tracing::info!("converting {} to raw image", path);
            let _cmd = cmd_output!(
                QEMU_IMG_BIN,
                "convert",
                "-f",
                img_format,
                "-O",
                "raw",
                path,
                &raw_path
            );
        }

        Ok(raw_path)
    }
}

// So far as I can tell, the only way to configure a bhyve zone is to use cloudinit. And so far
// as I can tell, the only way to do that is to make a fake CD-ROM ISO image, and temporarily
// attach it to the zone.

use indoc::formatdoc;

// At some point I might put in a mechanism to let the user supply templates.
//
fn cloudinit_network_config(config: &GurpZoneConfig) -> anyhow::Result<String> {
    let net = config
        .net
        .first()
        .context("no primary network config for cloudinit template")?;

    let dns = config
        .dns
        .as_ref()
        .context("no DNS config for cloudinit template")?;

    Ok(formatdoc! { "
        network:
          version: 1
          ethernets:
            eth0:
              dhcp4: false
              addresses: [{}/24]
              gateway4: {}
              nameservers:
                addresses: [{}]
                search: [{}]
              match: \".*\"
              set-name: eth0
        " ,
        net.allowed_address.as_ref().context("no address for cloudinit template")?,
        net.defrouter.as_ref().context("no defrouter for cloudinit template")?,
        dns.nameservers.join(", "),
        dns.domain
    })
}

// fn cloudinit_metadata_config(config: &GurpZone) -> String {
//     format!(
//         "instance-id: {}\nlocal-hostname: {}\n",
//         config.name, config.name
//     )
// }

fn cloudinit_iso_dir(config: &GurpZoneConfig) -> anyhow::Result<Utf8PathBuf> {
    let dir = Utf8PathBuf::from("/tmp");
    fs::write(
        dir.join("network-config"),
        cloudinit_network_config(config)?,
    )?;

    Ok(dir)
}

/*
fn fresh_cloudinit_iso_dir() {
    todo!()
}

fn cloudinit_src_dir() {
    todo!()
}

fn create_cloudinit_img_dir() {
    todo!()
}

fn create_cloudinit_iso_from_dir() {
    todo!()
}

fn render_cloudinit_template() {
    todo!()
}

fn cloudinit_tmp() {
    todo!()
}
*/

pub fn wait_for_readiness(zone: &str, opts: &ApplyOpts) -> anyhow::Result<bool> {
    // It's hard to know when a bhyve zone is fully booted. Here we look for a login prompt,
    // which seems universal across distributions.
    //
    // Zlogin requires a PTY, hence the use of portable_pty. We scan the raw bytes of what
    // zlogin sees in a separate thread, because that was the only way I could avoid choking
    // on non-UTF8 data.
    //
    // When the console mentions ttyS0, we "press return", otherwise we might not get a login
    // prompt. When we see the login prompt, we send an escape to end the console session, then
    // pass true back to the main thread and the function returns. If we hit EOF before that, we
    // send back false.
    //
    // I might replace this all of this with a ping...
    //
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(Default::default())?;

    let mut cmd = CommandBuilder::new(ZLOGIN_BIN);
    cmd.arg("-C");
    cmd.arg(zone);

    let mut child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let mut writer = pair.master.take_writer()?;
    let mut reader = pair.master.try_clone_reader()?;

    let (tx, rx) = mpsc::channel::<bool>();
    let dump_config = opts.dump_config;

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

            if dump_config {
                println!("{}", String::from_utf8_lossy(&buf[..n]));
            }

            if window.windows(5).any(|w| w == b"ttyS0") {
                let _ = writeln!(writer);
            }

            if window.windows(7).any(|w| w == b" login:") {
                let _ = writeln!(writer, "~.");
                let _ = tx.send(true);
                return;
            }
        }

        let _ = tx.send(false);
    });

    let ready = rx.recv().unwrap_or(false);
    let _ = child.wait();
    Ok(ready)
}
