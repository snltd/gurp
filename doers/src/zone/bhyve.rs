use crate::zone::config::{EmulConfig, GurpZoneBhyve, ZoneConfig};
use crate::zone::{cloudinit, console_watcher, control, image};
use anyhow::Context;
use common::constants::ZONEADM_BIN;
use common::types::ApplyOpts;
use uuid::Uuid;

pub fn build_zone(
    zone: &str,
    config: &ZoneConfig,
    uuid: &Uuid,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let brand_config = config
        .bhyve
        .as_ref()
        .with_context(|| format!("no bhyve config for {zone}"))?;

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

pub fn zone_config(config: &GurpZoneBhyve, has_cloudinit: bool, uuid: &Uuid) -> String {
    let mut ret = String::new();

    ret.push_str(zone_device!(config.boot_device()));
    ret.push_str(zone_attr!("bootrom", "string", config.boot_rom));
    ret.push_str(zone_attr!("bootdisk", "string", config.boot_volume));
    ret.push_str(zone_attr!("vcpus", "string", config.vcpus));
    ret.push_str(zone_attr!("ram", "string", config.ram));
    ret.push_str(zone_attr!("acpi", "string", config.acpi));

    if has_cloudinit {
        ret.push_str(&cloudinit::zone_config_snippet(uuid));
    };

    ret
}
