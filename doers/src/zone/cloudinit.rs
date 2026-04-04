use crate::zone::config::GurpZoneBhyve;
use anyhow::{Context, ensure};
use camino::Utf8PathBuf;
use camino_tempfile::Utf8TempDir;
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

    let cloudinit_iso_dir = camino_tempfile::tempdir()?;
    populate(&cloudinit_iso_dir, config)?;
    create_cloudinit_iso(&cloudinit_iso_dir, iso_file)?;
    Ok(())
}

fn populate(dir: &Utf8TempDir, config: &GurpZoneBhyve) -> anyhow::Result<()> {
    tracing::debug!("Constructing cloud-init CDROM");

    if let Some(cloudinit_files) = &config.cloudinit_files {
        for file in cloudinit_files {
            let src_path = Utf8PathBuf::from(file);
            let basename = src_path
                .file_name()
                .with_context(|| format!("Cannot get basename of {}", src_path))?;

            let target_path = dir.path().join(basename);

            tracing::debug!("Copying {} to {}", src_path, target_path);

            fs::copy(&src_path, &target_path)
                .with_context(|| format!("failed to copy from {src_path} to {target_path}"))?;
        }
    }

    if let Some(cloudinit_struct) = &config.cloudinit_struct
        && let Some(obj) = cloudinit_struct.as_object()
    {
        for (file_name, content) in obj {
            let cloudinit_target = dir.path().join(file_name);
            tracing::debug!("Creating {}", cloudinit_target);
            let cloudinit_yaml = serde_yaml_bw::to_string(&content)?;

            let cloudinit_content = if file_name == "user-data" {
                format!("#cloud-config\n{}", cloudinit_yaml)
            } else {
                cloudinit_yaml
            };

            fs::write(&cloudinit_target, cloudinit_content).with_context(|| {
                format!("failed to write Cloudinit content to {cloudinit_target}")
            })?;
        }
    }

    Ok(())
}

fn create_cloudinit_iso(dir: &Utf8TempDir, target: &Utf8PathBuf) -> anyhow::Result<()> {
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
