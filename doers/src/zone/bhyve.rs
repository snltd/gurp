use crate::zone::config::GurpZoneConfig;
use anyhow::Context;
use anyhow::bail;
use camino::Utf8PathBuf;
use common::constants::QEMU_IMG_BIN;
use std::fs;
use util::http;

pub fn bhyve_zone_config(config: &GurpZoneConfig) -> String {
    let mut ret = String::new();

    if let Some(bhyve) = &config.bhyve {
        ret.push_str(&format!(
            "add device\n\tset match=\"/dev/zvol/rdsk/{}\"\nend\n",
            bhyve.boot_volume
        ));

        ret.push_str(&zone_attr!("bootrom", "string", "BHYVE_RELEASE"));
        ret.push_str(&zone_attr!("bootdisk", "string", &bhyve.boot_volume));
        ret.push_str(&zone_attr!("vcpus", "string", &bhyve.vcpus));
        ret.push_str(&zone_attr!("ram", "string", &bhyve.ram));
        ret.push_str(&zone_attr!("acpi", "string", "false"));
    }

    ret
}

// Creating the volume is the ZFS doer/user's responsibility.
//
pub fn pre_install(config: &GurpZoneConfig) -> anyhow::Result<()> {
    tracing::info!("Running bhyve pre_install");

    let bhyve_config = config
        .bhyve
        .as_ref()
        .context("trying to configure bhyve, but no config")?;

    let img_file = http::image_in_cache(&bhyve_config.image_url)?;
    let img_format = bhyve_config.image_format.as_deref().unwrap_or("qcow");
    let raw_img_file = convert_to_raw(img_file, img_format)?;
    write_img_to_boot_zvol(&raw_img_file, &bhyve_config.boot_volume)?;

    if bhyve_config.cloudinit {
        let cloudinit_dir = cloudinit_iso_dir(config);
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

fn cloudinit_metadata_config(config: &GurpZoneConfig) -> String {
    format!(
        "instance-id: {}\nlocal-hostname: {}\n",
        config.name, config.name
    )
}

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
