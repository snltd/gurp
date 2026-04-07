use crate::zone::config::{Brand, ZoneConfig};
use crate::zone::{control, illumos, lx};
use anyhow::{Context, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use common::cmd;
use common::constants::{ZLOGIN_BIN, ZONEADM_BIN};
use common::types::ApplyOpts;
use fs_extra::dir::CopyOptions;
use std::process::{Command, Stdio};
use std::{env, fs};

// Container as opposed to bhyve.

pub fn build_zone(zone: &str, config: &ZoneConfig, opts: &ApplyOpts) -> anyhow::Result<()> {
    if let Some(clone_source) = &config.clone_from {
        clone(zone, clone_source)?;
    } else {
        install(zone, &config.brand, config.image.as_deref())?;
    }

    if config.boot_after_install {
        control::boot_zone(zone)?;
        match config.brand {
            Brand::Lx => lx::wait_for_readiness(zone)?,
            _ => control::wait_for_readiness(zone)?,
        };
    }

    if config.brand == Brand::Lx
        && let Some(dns_config) = &config.dns
    {
        lx::set_up_dns(&config.zonepath, dns_config)?;
    }

    if let Some(files) = &config.copy_in {
        let zone_root = &config.zonepath;

        ensure!(
            zone_root.exists(),
            format!("cannot find zone root {zone_root}")
        );

        for (src, dest) in files {
            copy_to_zone(zone_root, src, dest)?;
        }
    }

    if config.bootstrap.is_some() {
        bootstrap(zone, config, opts)?;
    }

    if let Some(cmds) = &config.exec_in {
        for cmd in cmds {
            exec_in(zone, cmd)?;
        }
    }

    Ok(())
}

fn install(zone: &str, brand: &Brand, image: Option<&str>) -> anyhow::Result<()> {
    tracing::info!("installing {} [{}]", zone, brand);

    let _ = match brand {
        Brand::Illumos => install_from_image(zone, &illumos::image_path(image)?),
        Brand::Lx => install_from_image(zone, &lx::image_path(image)?),
        _ => cmd_output!(ZONEADM_BIN, "-z", zone, "install"),
    }
    .with_context(|| format!("failed to install {brand} zone {zone}"))?;

    tracing::debug!("zone {zone}: installed");
    Ok(())
}

fn install_from_image(zone: &str, image: &Utf8Path) -> anyhow::Result<String> {
    cmd_output!(ZONEADM_BIN, "-z", zone, "install", "-s", image,)
}

fn clone(zone: &str, source_zone: &str) -> anyhow::Result<()> {
    tracing::info!("zone {zone}: cloning from {source_zone}");

    cmd_output!(ZONEADM_BIN, "-z", zone, "clone", source_zone)
        .with_context(|| format!("failed to clone {zone} from {source_zone}"))?;

    tracing::debug!("zone {zone}: cloned");
    Ok(())
}

fn copy_to_zone(zone_root: &Utf8Path, src: &Utf8Path, dest: &str) -> anyhow::Result<()> {
    let relative_dest = dest.trim_matches('/');
    let mut zone_dest = zone_root.join(relative_dest);

    // If target is a directory, append the source's filename
    // If target.parent() does not exist, make it
    if dest.ends_with('/')
        && let Some(fname) = src.file_name()
    {
        if !zone_dest.exists() {
            fs::create_dir_all(&zone_dest)
                .with_context(|| format!("failed to create zone dir {zone_dest}"))?;
        }

        zone_dest = zone_dest.join(fname);
    }

    tracing::info!("copying {} -> {}", src, zone_dest);

    if src.is_file() {
        fs::copy(src, &zone_dest)
            .with_context(|| format!("failed to copy from {src} to {zone_dest}"))?;
    } else if src.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;

        fs_extra::dir::copy(src, &zone_dest, &options)
            .with_context(|| format!("failed to copy from {src} to {zone_dest}"))?;
    } else {
        bail!("{} is neither a file nor a directory", src);
    }

    Ok(())
}

fn bootstrap(zone: &str, conf: &ZoneConfig, opts: &ApplyOpts) -> anyhow::Result<()> {
    let bootstrap_conf = conf.bootstrap.as_ref().context("no bootstrap config")?;
    let bootstrap_bin = "/var/tmp/gurp";
    let mut bootstrap_args: Vec<String> = Vec::new();

    // Passing the env var breaks zlogin on LX zones
    if let Some(log_level) = env::var_os("RUST_LOG")
        && conf.brand != Brand::Lx
    {
        bootstrap_args.push(format!("RUST_LOG={}", log_level.to_string_lossy()));
    }

    bootstrap_args.push(bootstrap_bin.to_owned());
    bootstrap_args.push("apply".to_owned());

    if opts.dump_config {
        bootstrap_args.push("--dump-config".to_owned());
    }

    if opts.colour {
        bootstrap_args.push("--colour".to_owned());
    }

    if opts.line_no {
        bootstrap_args.push("--line-no".to_owned());
    }

    if let Some(metrics_host) = &opts.metrics_to {
        bootstrap_args.push(format!("--metrics-to={metrics_host}"));
    }

    ensure!(
        exactly_one_some!(bootstrap_conf.server, bootstrap_conf.file),
        "bootstrap requires exactly one of :file and :server"
    );

    if let Some(server) = &bootstrap_conf.server {
        tracing::info!("bootstrapping from remote server: {server}");
        bootstrap_args.push(format!("--server={server}"));

        if let Some(hostname) = &bootstrap_conf.hostname {
            bootstrap_args.push(format!("--hostname={hostname}"));
        }
    } else if let Some(file) = &bootstrap_conf.file {
        tracing::info!("bootstrapping from local file: {file}");
        bootstrap_args.push(file.to_owned());
    } else {
        bail!("bootstrap requires either :file or :server");
    }

    let my_path = env::current_exe().context("can't get my path")?;
    let this_exec = match Utf8PathBuf::from_path_buf(my_path) {
        Ok(path) => path,
        Err(_) => bail!(format!("failed to get Gurp path")),
    };

    copy_to_zone(&conf.zonepath, &this_exec, bootstrap_bin)?;
    exec_in(zone, &bootstrap_args.join(" "))?;
    tracing::info!("END BOOTSTRAP {zone}");

    Ok(())
}

fn exec_in(zone: &str, command: &str) -> anyhow::Result<()> {
    tracing::debug!("zone {zone}; exec '{command}'");
    let mut cmd = Command::new(ZLOGIN_BIN);
    cmd.arg(zone);
    cmd.args(command.split_whitespace().collect::<Vec<_>>());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    tracing::debug!(command = cmd::to_string(&cmd));

    let output = cmd
        .output()
        .with_context(|| format!("failed to run zlogin command {command} against {zone}"))?;

    ensure!(
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned()
    );

    tracing::debug!("zone {zone}; exec '{command}' OK");

    Ok(())
}
