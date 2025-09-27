use crate::zone::config::GurpZoneBhyve;
use anyhow::{bail, ensure};
use camino::Utf8PathBuf;
use common::constants::MKISOFS_BIN;
use std::fs;

// So far as I can tell, the only way to configure a bhyve zone is to use cloudinit. And so far
// as I can tell, the only way to do that is to make a fake CD-ROM ISO image, and temporarily
// attach it to the zone.

pub fn setup(config: &GurpZoneBhyve, iso_file: &Utf8PathBuf) -> anyhow::Result<()> {
    tracing::debug!("Setting up Cloudinit");

    ensure!(
        Utf8PathBuf::from(MKISOFS_BIN).exists(),
        "{} not found. Perhaps you need to install pkg:/media/xorriso",
        MKISOFS_BIN
    );

    // TODO replace with tempdir
    let cloudinit_iso_dir = Utf8PathBuf::from("/tmp/cloudinit");

    if cloudinit_iso_dir.exists() {
        fs::remove_dir_all(&cloudinit_iso_dir)?;
    }

    fs::create_dir(&cloudinit_iso_dir)?;
    tracing::debug!("Created Cloudinit dir at {cloudinit_iso_dir}");

    populate(&cloudinit_iso_dir, config)?;
    create_cloudinit_iso(&cloudinit_iso_dir, iso_file)?;

    Ok(())
}

// For now let's do hardcoded defaults. We'll add the option to use a dir another time
fn populate(dir: &Utf8PathBuf, config: &GurpZoneBhyve) -> anyhow::Result<()> {
    tracing::debug!("Constructing cloud-init CDROM in {dir}");

    if let Some(cloudinit_file) = &config.cloudinit_file {
        tracing::debug!("Copying {} to {}", cloudinit_file, dir);
        // fs::copy(cloudinit_file, &cloudinit_target)?;
    } else if let Some(cloudinit_struct) = &config.cloudinit_struct {
        if let Some(obj) = cloudinit_struct.as_object() {
            for (file_name, content) in obj {
                let cloudinit_target = dir.join(file_name);
                tracing::debug!("Creating {}", cloudinit_target);
                let cloudinit_yaml = serde_yaml_bw::to_string(&content)?;

                let cloudinit_content = if file_name == "user-data" {
                    format!("#cloud-config\n{}", cloudinit_yaml)
                } else {
                    cloudinit_yaml
                };

                fs::write(&cloudinit_target, cloudinit_content)?;
            }
        }
    } else {
        bail!("No cloudinit info");
    }

    Ok(())
}

fn create_cloudinit_iso(dir: &Utf8PathBuf, target: &Utf8PathBuf) -> anyhow::Result<()> {
    let _ = cmd_output!(
        MKISOFS_BIN,
        "-output",
        &target,
        "-volid",
        "cidata",
        "-joliet",
        "-rock",
        format!("{dir}/")
    );

    Ok(())
}
