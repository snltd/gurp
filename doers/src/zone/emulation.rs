// In which we collect functionality shared by bhyve and emu
//
pub trait EmulConfig {
    fn boot_volume(&self) -> String {
        self.boot_volume
    }

    fn image_format(&self) -> String {
        self.image_format
    }

    fn has_cloudinit(&self) -> bool {
        self.has_cloudinit
    }

    fn wait_for_boot(&self) -> bool {
        self.wait_for_boot
    }

    fn boot_after_install(&self) -> bool {
        self.boot_after_install
    }

    fn boot_device(&self) -> Utf8PathBuf {
        Utf8PathBuf::from("/dev/zvol/rdsk").join(&self.boot_volume)
    }
}

pub fn build_zone<T: EmulConfig>(
    zone: &str,
    image: &ZoneConfig,
    boot_after_install: bool,
    brand_config: &T,
    uuid: &Uuid,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    if let Some(image_source) = &config.image {
        let image_path = match image_source {
            ImageSource::Url(url) => {
                image_on_disk(url, opts).with_context(|| format!("cannot get image from {url}"))?
            }
            ImageSource::Path(path) => path.to_owned(),
            ImageSource::Name(_) => bail!("image names are not supported for emulation zones"),
        };

        write_image_to_boot_disk(
            &image_path,
            &brand_config.boot_volume(),
            &brand_config.image_format(),
        )
        .context("failed to write boot image")?;
    }

    if brand_config.has_cloudinit() {
        cloudinit::setup(brand_config, &cloudinit::iso_path(uuid), opts)?;
    }

    cmd_output!(ZONEADM_BIN, "-z", zone, "install")?;

    if brand_config.boot_after_install {
        control::boot_zone(zone)?;
    }

    if brand_config.wait_for_boot {
        wait_for_readiness(zone, uuid)?;
    }

    if brand_config.has_cloudinit() {
        remove_cloudinit_config(zone)?;
    }

    Ok(())
}
