use crate::zone::config::{CloudInitConfig, GurpZoneFilesystem};
use anyhow::{Context, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::Utf8TempDir;
use common::constants::{MKISOFS_BIN, ZONECFG_BIN};
use common::info::dump_config;
use common::types::ApplyOpts;
use serde_json::Value;
use std::fs;
use uuid::Uuid;

// So far as I can tell, the only way to configure a bhyve zone is to use cloudinit. And so far
// as I can tell, the only way to do that is to make a fake CD-ROM ISO image, and temporarily
// attach it to the zone.
//
// At the start of the zone ensure phase, we generate a UUID. That UUID is used as the name of
// an ISO file which is mounted inside the zone on first boot.
//
// The user can give us files to copy in, and also structs that we can turn into YAML on their
// behalf.

pub fn setup(
    config: &CloudInitConfig,
    iso_file: &Utf8Path,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    tracing::debug!("Setting up Cloudinit");

    ensure!(
        Utf8PathBuf::from(MKISOFS_BIN).exists(),
        "{} not found. Perhaps you need to install pkg:/media/xorriso",
        MKISOFS_BIN
    );

    let build_dir = camino_tempfile::tempdir()?;

    populate(&build_dir, config, opts)?;
    create_cloudinit_iso(&build_dir, iso_file)?;

    Ok(())
}

pub fn remove(zone: &str) -> anyhow::Result<()> {
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

pub fn iso_path(uuid: &Uuid) -> Utf8PathBuf {
    Utf8PathBuf::from("/tmp").join(format!("{uuid}.iso"))
}

pub fn zone_config_snippet(uuid: &Uuid) -> String {
    let iso_path = iso_path(uuid);

    let fs = zone_fs!(GurpZoneFilesystem {
        dir: iso_path.clone(),
        special: iso_path.clone(),
        fs_type: "lofs".to_owned(),
        options: Some(vec!["ro".to_owned()])
    });

    format!("{}\n{fs}", zone_attr!("cdrom", "string", iso_path))
}

fn populate(
    build_dir: &Utf8TempDir,
    config: &CloudInitConfig,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    tracing::debug!("Constructing cloud-init CDROM");

    for (file_name, source) in &config.from {
        copy_file(source, &build_dir.path().join(file_name))?;
    }

    for (file_name, content) in &config.from_struct {
        struct_to_file(file_name, build_dir.path(), content, opts)?;
    }

    Ok(())
}

fn copy_file(src: &Utf8Path, dest: &Utf8Path) -> anyhow::Result<()> {
    tracing::debug!("Copying cloudinit {} to {}", src, dest);

    fs::copy(src, dest).with_context(|| format!("failed to copy from {src} to {dest}"))?;

    Ok(())
}

fn struct_to_file(
    file_name: &str,
    build_dir_path: &Utf8Path,
    content_struct: &Value,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let target = build_dir_path.join(file_name);
    tracing::debug!("Creating cloudinit file {}", target);

    let yaml = serde_yaml_bw::to_string(&content_struct)?;

    let cloudinit_content = if file_name == "user-data" {
        format!("#cloud-config\n{}", yaml)
    } else {
        yaml
    };

    if opts.output.dump_configs {
        println!(
            "{}",
            dump_config(
                &cloudinit_content,
                Some(&format!("cloudinit YAML: {file_name}")),
                &opts.output
            )
        );
    }

    fs::write(&target, cloudinit_content)
        .with_context(|| format!("failed to write Cloudinit content to {target}"))?;

    Ok(())
}

fn create_cloudinit_iso(dir: &Utf8TempDir, target: &Utf8Path) -> anyhow::Result<()> {
    let _ = cmd_output!(
        MKISOFS_BIN,
        "-output",
        &target,
        "-volid",
        "cidata",
        "-joliet",
        "-rock",
        format!("{}/", &dir.path())
    )
    .with_context(|| {
        format!(
            "failed to create cloudinit ISO at {target} from {}",
            dir.path()
        )
    })?;

    Ok(())
}
