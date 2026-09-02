use crate::zone::config::{EmulConfig, GurpZoneEmu, ImageSource, ZoneConfig};
use crate::zone::{cloudinit, console_watcher, control, image};
use anyhow::Context;
use camino::Utf8Path;
use common::constants::ZONEADM_BIN;
use common::types::ApplyOpts;
use std::fs;
use uuid::Uuid;

pub fn build_zone(
    zone: &str,
    config: &ZoneConfig,
    uuid: &Uuid,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let brand_config = config
        .emu
        .as_ref()
        .with_context(|| format!("no emu config for {zone}"))?;

    if let Some(image_source) = &config.image {
        let image_path = image::path(image_source, opts)?;

        image::write_to_boot_zvol(
            &image_path,
            brand_config.boot_volume(),
            brand_config.image_format(),
        )
        .context("failed to write boot image")?
    }

    if let Some(ci_cfg) = &config.cloudinit {
        cloudinit::setup(ci_cfg, &cloudinit::iso_path(uuid), opts)?;
    }

    cmd_output!(ZONEADM_BIN, "-z", zone, "install")?;

    if let Some(bios_src) = &brand_config.bios {
        setup_bios(bios_src, &config.zonepath, opts)?;
    }

    if config.boot_after_install {
        control::boot_zone(zone)?;
    }

    if brand_config.wait_for_boot() {
        console_watcher::wait_for_readiness(zone, uuid)?;
    }

    if config.has_cloudinit() {
        cloudinit::teardown(zone)?;
    }

    Ok(())
}

fn setup_bios(bios_src: &ImageSource, zonepath: &Utf8Path, opts: &ApplyOpts) -> anyhow::Result<()> {
    let cached_path = image::path(bios_src, opts)?;

    let bios_filename = cached_path
        .file_name()
        .with_context(|| format!("cannot get filename of {cached_path}"))?;

    let bios_target = zonepath.join("root").join(bios_filename);

    fs::copy(&cached_path, &bios_target)
        .with_context(|| format!("failed to copy from {cached_path} to {bios_target}"))?;

    Ok(())
}

pub fn zone_config(config: &GurpZoneEmu, has_cloudinit: bool, uuid: &Uuid) -> String {
    let mut ret = String::new();

    ret.push_str(zone_device!(config.boot_device()));
    ret.push_str(zone_attr!("arch", "string", config.arch));
    ret.push_str(zone_attr!("bootdisk", "string", config.boot_volume));
    ret.push_str(zone_attr!("cpu", "string", config.cpu));
    ret.push_str(zone_attr!("ram", "string", config.ram));
    ret.push_str(zone_attr!("vcpus", "string", config.vcpus));

    if let Some(bios_src) = &config.bios {
        let raw = match bios_src {
            ImageSource::Url(url) => url.to_string(),
            ImageSource::Path(path) => path.to_string(),
            ImageSource::Name(name) => name.clone(), //should never happen
        };

        match raw.rsplit("/").next() {
            Some(name) => ret.push_str(zone_attr!("extra", "string", format!("\"-bios {name}\""))),
            None => tracing::warn!("could not find bios name"), //should never happen
        }
    }

    if let Some(extras) = &config.qemu_args {
        for (i, val) in extras.iter().enumerate() {
            let name = format!("extra{}", i + 1);
            ret.push_str(zone_attr!(name, "string", val));
        }
    }

    if has_cloudinit {
        ret.push_str(&cloudinit::zone_config_snippet(uuid));
    };

    ret
}
