use crate::zone::config::{Brand, ZoneConfig};
use crate::zone::types::ZoneImage;
use crate::zone::{control, illumos, lx};
use anyhow::{Context, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use common::cmd;
use common::constants::{ZLOGIN_BIN, ZONEADM_BIN};
use common::types::ApplyOpts;
use fs_extra::dir::CopyOptions;
use std::process::{Command, Stdio};
use std::{env, fs};

// native container as opposed to emulation.

pub fn build_zone(zone: &str, config: &ZoneConfig, opts: &ApplyOpts) -> anyhow::Result<()> {
    if let Some(clone_source) = &config.clone_from {
        clone(zone, clone_source)?;
    } else {
        install(
            zone,
            &config.brand,
            ZoneImage {
                image_source: config.image.as_ref(),
                checksum: config.image_checksum.as_ref(),
            },
        )?;
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
        ensure!(
            &config.zonepath.exists(),
            format!("cannot find zone root {}", config.zonepath)
        );

        for (src, dest) in files {
            copy_to_zone(&config.zonepath, src, dest)?;
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

    create_zone_brand_fact(&config.zonepath, &config.brand)?;

    Ok(())
}

fn install(zone: &str, brand: &Brand, image: ZoneImage) -> anyhow::Result<()> {
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

/// Creates a file in the zone's /etc/gurp/ saying what brand the zone is
fn create_zone_brand_fact(zonepath: &Utf8Path, brand: &Brand) -> anyhow::Result<()> {
    let etc_dir = zonepath.join("root").join("etc").join("gurp");
    let fact_path = etc_dir.join("zone-brand.fact");

    if !etc_dir.exists() {
        tracing::info!("creating {etc_dir}");
        fs::create_dir_all(&etc_dir)
            .with_context(|| format!("failed to create etc_dir {etc_dir}"))?;
    }

    fs::write(fact_path, format!("{brand}\n"))?;
    Ok(())
}

fn copy_to_zone(zonepath: &Utf8Path, src: &Utf8Path, dest: &str) -> anyhow::Result<()> {
    let zone_root = zonepath.join("root");
    let relative_dest = dest.trim_matches('/');
    let dest_path = zone_root.join(relative_dest);

    let dest_is_dir = dest.ends_with('/') || (dest_path.exists() && dest_path.is_dir());

    let dest_dir = if dest_is_dir {
        &dest_path
    } else {
        dest_path.parent().context("cannot get target parent")?
    };

    if !dest_dir.exists() {
        tracing::info!("creating {dest_dir}");
        fs::create_dir_all(dest_dir)
            .with_context(|| format!("failed to create dest_dir {dest_dir}"))?;
    }

    // If dest resolves to a directory, append the source's filename.
    let dest_path = if dest_is_dir {
        let filename = src.file_name().context("src has no filename")?;
        dest_path.join(filename)
    } else {
        dest_path
    };

    tracing::info!("copying  {} -> {}", src, dest_path);

    if src.is_file() {
        fs::copy(src, &dest_path)
            .with_context(|| format!("failed to copy from {src} to {dest_path}"))?;
    } else if src.is_dir() {
        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;

        fs_extra::dir::copy(src, &dest_path, &options)
            .with_context(|| format!("failed to copy from {src} to {dest_path}"))?;
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

    if opts.output.dump_configs {
        bootstrap_args.push("--dump-config".to_owned());
    }

    if opts.output.colour {
        bootstrap_args.push("--colour".to_owned());
    }

    if opts.output.line_no {
        bootstrap_args.push("--line-no".to_owned());
    }

    if let Some(metrics_host) = &opts.globals.metrics_to {
        bootstrap_args.push(format!("--metrics-to={metrics_host}"));
    }

    ensure!(
        exactly_one_some!(bootstrap_conf.server, bootstrap_conf.file),
        "bootstrap requires exactly one of :file and :server"
    );

    if let Some(server) = &bootstrap_conf.server {
        tracing::info!("bootstrapping {zone} from remote server: {server}");
        bootstrap_args.push(format!("--server={server}"));

        if let Some(hostname) = &bootstrap_conf.hostname {
            bootstrap_args.push(format!("--hostname={hostname}"));
        }
    } else if let Some(file) = &bootstrap_conf.file {
        tracing::info!("bootstrapping {zone} from local file: {file}");
        bootstrap_args.push(file.to_owned());
    } else {
        bail!("bootstrap requires either :file or :server");
    }

    let my_path = env::current_exe().context("can't get my path")?;
    let this_exec = match Utf8PathBuf::from_path_buf(my_path) {
        Ok(path) => path,
        Err(_) => bail!("failed to get Gurp path"),
    };

    copy_to_zone(&conf.zonepath, &this_exec, bootstrap_bin)?;
    let bootstrap_cmd = bootstrap_args.join(" ");

    exec_in(zone, &bootstrap_cmd).with_context(|| {
        format!("in zone {zone}: failed to run bootstrap command: {bootstrap_cmd}")
    })?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8Path;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn copy_file_to_explicit_dest_path() {
        // dest names the file explicitly: /etc/myfile.conf
        // Intermediate dirs should be created, file should appear at that exact path.
        let (_ztmp, zone) = make_zone();
        let src_tmp = TempDir::new().unwrap();
        let src = make_src_file(&src_tmp, "myfile.conf", "data");

        copy_to_zone(&zone, &src, "/etc/myfile.conf").unwrap();

        let dest = zone.join("root/etc/myfile.conf");
        assert!(dest.exists(), "file should exist at explicit dest path");
        assert_eq!(fs::read_to_string(&dest).unwrap(), "data");
    }

    #[test]
    fn copy_file_to_dest_with_trailing_slash_uses_src_filename() {
        // dest ends with '/': the file should land inside that directory
        // keeping its original name.
        let (_ztmp, zone) = make_zone();
        let src_tmp = TempDir::new().unwrap();
        let src = make_src_file(&src_tmp, "myfile.conf", "content");

        // Pre-create the directory so we're testing the trailing-slash logic,
        // not the dir-creation path.
        fs::create_dir_all(zone.join("root/etc")).unwrap();

        copy_to_zone(&zone, &src, "/etc/").unwrap();

        let dest = zone.join("root/etc/myfile.conf");
        assert!(
            dest.exists(),
            "file should be placed inside dir named by dest"
        );
    }

    #[test]
    fn copy_file_creates_missing_intermediate_dirs() {
        // Destination directories don't exist yet — function must create them.
        let (_ztmp, zone) = make_zone();
        let src_tmp = TempDir::new().unwrap();
        let src = make_src_file(&src_tmp, "app.cfg", "cfg");

        copy_to_zone(&zone, &src, "/usr/local/etc/app.cfg").unwrap();

        assert!(zone.join("root/usr/local/etc/app.cfg").exists());
    }

    #[test]
    fn copy_file_to_existing_dir_dest_uses_src_filename() {
        // dest resolves to an existing directory (no trailing slash).
        // Function should detect it's a dir and place the file inside.
        let (_ztmp, zone) = make_zone();
        let src_tmp = TempDir::new().unwrap();
        let src = make_src_file(&src_tmp, "readme.txt", "hi");

        let dest_dir = zone.join("root/opt");
        fs::create_dir_all(&dest_dir).unwrap();

        copy_to_zone(&zone, &src, "/opt").unwrap();

        assert!(zone.join("root/opt/readme.txt").exists());
    }

    #[test]
    fn copy_file_overwrites_existing_file() {
        let (_ztmp, zone) = make_zone();
        let src_tmp = TempDir::new().unwrap();

        // Write original
        let dest_path = zone.join("root/etc/config");
        fs::create_dir_all(dest_path.parent().unwrap()).unwrap();
        fs::write(&dest_path, "old").unwrap();

        let src = make_src_file(&src_tmp, "config", "new");
        copy_to_zone(&zone, &src, "/etc/config").unwrap();

        assert_eq!(fs::read_to_string(&dest_path).unwrap(), "new");
    }

    // --- Directory copying ---

    #[test]
    fn copy_dir_to_dest() {
        let (_ztmp, zone) = make_zone();
        let src_tmp = TempDir::new().unwrap();
        let src = make_src_dir(&src_tmp, "myapp");

        copy_to_zone(&zone, &src, "/opt/myapp").unwrap();

        // fs_extra with copy_inside copies the *contents* into dest_path
        assert!(zone.join("root/opt/myapp/inner.txt").exists());
    }

    #[test]
    fn copy_dir_with_trailing_slash_dest() {
        let (_ztmp, zone) = make_zone();
        let src_tmp = TempDir::new().unwrap();
        let src = make_src_dir(&src_tmp, "myapp");

        fs::create_dir_all(zone.join("root/opt")).unwrap();
        copy_to_zone(&zone, &src, "/opt/").unwrap();

        // dest ends with '/', so src dir name is appended → root/opt/myapp/
        // copy_inside=true copies contents into that dir
        assert!(zone.join("root/opt/myapp/inner.txt").exists());
    }

    #[test]
    fn error_when_src_does_not_exist() {
        let (_ztmp, zone) = make_zone();
        let src = Utf8Path::new("/nonexistent/ghost.txt");

        let err = copy_to_zone(&zone, src, "/etc/ghost.txt").unwrap_err();
        assert!(
            err.to_string().contains("neither a file nor a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn strips_leading_slash_from_dest() {
        // Ensures dest is treated as relative inside the zone root, not absolute.
        let (_ztmp, zone) = make_zone();
        let src_tmp = TempDir::new().unwrap();
        let src = make_src_file(&src_tmp, "foo.txt", "bar");

        copy_to_zone(&zone, &src, "/foo.txt").unwrap();

        // Must land inside zone root, not at the real filesystem root
        assert!(zone.join("root/foo.txt").exists());
        assert!(!std::path::Path::new("/foo.txt").exists());
    }

    fn make_zone() -> (TempDir, camino::Utf8PathBuf) {
        let tmp = TempDir::new().unwrap();
        let zone_path = camino::Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();
        fs::create_dir_all(zone_path.join("root")).unwrap();
        (tmp, zone_path)
    }

    fn make_src_file(dir: &TempDir, name: &str, content: &str) -> camino::Utf8PathBuf {
        let path = camino::Utf8PathBuf::from_path_buf(dir.path().join(name)).unwrap();
        fs::write(&path, content).unwrap();
        path
    }

    fn make_src_dir(dir: &TempDir, name: &str) -> camino::Utf8PathBuf {
        let path = camino::Utf8PathBuf::from_path_buf(dir.path().join(name)).unwrap();
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("inner.txt"), "hello").unwrap();
        path
    }
}
