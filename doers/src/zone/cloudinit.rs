use crate::zone::config::GurpZoneBhyve;
use anyhow::{Context, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::Utf8TempDir;
use common::constants::MKISOFS_BIN;
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

pub fn iso_path(uuid: &Uuid) -> Utf8PathBuf {
    Utf8PathBuf::from("/tmp").join(format!("{uuid}.iso"))
}

pub fn setup(config: &GurpZoneBhyve, iso_file: &Utf8Path, opts: &ApplyOpts) -> anyhow::Result<()> {
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

fn populate(
    build_dir: &Utf8TempDir,
    config: &GurpZoneBhyve,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    tracing::debug!("Constructing cloud-init CDROM");

    if let Some(cloudinit_files) = &config.cloudinit_files {
        copy_files(cloudinit_files, build_dir)?;
    }

    if let Some(cloudinit_struct) = &config.cloudinit_struct
        && let Some(obj) = cloudinit_struct.as_object()
    {
        for (file_name, content) in obj {
            struct_to_file(file_name, build_dir, content, opts)?;
        }
    }

    Ok(())
}

fn copy_files(cloudinit_files: &Vec<Utf8PathBuf>, build_dir: &Utf8TempDir) -> anyhow::Result<()> {
    for src_path in cloudinit_files {
        let basename = src_path
            .file_name()
            .with_context(|| format!("Cannot get basename of {}", src_path))?;

        let target_path = build_dir.path().join(basename);

        tracing::debug!("Copying cloudinit {} to {}", src_path, target_path);

        fs::copy(src_path, &target_path)
            .with_context(|| format!("failed to copy from {src_path} to {target_path}"))?;
    }

    Ok(())
}

fn struct_to_file(
    file_name: &str,
    build_dir: &Utf8TempDir,
    content_struct: &Value,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let file_path = build_dir.path().join(file_name);
    tracing::debug!("Creating cloudinit file {}", file_path);
    let yaml = serde_yaml_bw::to_string(&content_struct)?;

    let cloudinit_content = if file_name == "user-data" {
        format!("#cloud-config\n{}", yaml)
    } else {
        yaml
    };

    if opts.dump_config {
        println!(
            "{}",
            dump_config(
                &cloudinit_content,
                Some(&format!("cloudinit YAML: {file_name}")),
                opts
            )
        );
    }

    fs::write(&file_path, cloudinit_content)
        .with_context(|| format!("failed to write Cloudinit content to {file_path}"))?;

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
