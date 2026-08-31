use crate::zone::emulation;

pub fn build_zone(zone: &str, config: &ZoneConfig, uuid: &Uuid, opts: &ApplyOpts) {
    emulation::build_zone(
        zone,
        config.image,
        config.boot_after_install,
        config
            .emu
            .with_context(|| format!("no emu config for {zone}")),
        uuid,
        opts,
    )
}
