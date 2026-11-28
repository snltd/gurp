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
use util::facts;

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
                checksum: config.checksum.as_ref(),
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

    facts::create(
        "zone-brand",
        &config.brand.to_string(),
        Some(config.zonepath.as_ref()),
    )?;

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
    let mut bootstrap_args: Vec<String> = Vec::new();

    if let Some(as_host) = &bootstrap_conf.host {
        facts::create("as-host", as_host, Some(conf.zonepath.as_ref()))?;
    } else {
    }

    // Passing the RUST_LOG env var breaks zlogin on LX zones
    if bootstrap_conf.debug {
        if conf.brand == Brand::Lx {
            tracing::warn!("cannot bootstrap LX zones with RUST_LOG=debug");
        } else {
            bootstrap_args.push("RUST_LOG=debug".to_owned());
        }
    } else if let Some(log_level) = env::var_os("RUST_LOG") {
        if conf.brand == Brand::Lx {
            tracing::warn!("cannot bootstrap LX zones with RUST_LOG=debug");
        } else {
            bootstrap_args.push(format!("RUST_LOG={}", log_level.to_string_lossy()));
        }
    }

    bootstrap_args.push(bootstrap_conf.gurp_binary.to_string());
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

    bootstrap_args.push("-Dis-bootstrap".to_owned());

    ensure!(
        exactly_one_some!(bootstrap_conf.server, bootstrap_conf.file),
        "bootstrap requires exactly one of :file and :server"
    );

    if let Some(server) = &bootstrap_conf.server {
        tracing::info!("bootstrapping {zone} from remote server: {server}");
        bootstrap_args.push(format!("--server={server}"));

        if let Some(host) = &bootstrap_conf.host {
            bootstrap_args.push(format!("--host={host}"));
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

    if bootstrap_conf.copy_self {
        copy_to_zone(
            &conf.zonepath,
            &this_exec,
            bootstrap_conf.gurp_binary.as_str(),
        )?;
    }

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
    use snltest::fixture_dir;

    struct CopyTest<'a> {
        source_files: Vec<&'a str>,
        copy: &'a str,
        to: &'a str,
        expect: Vec<&'a str>,
    }

    #[track_caller]
    fn assert_copy(ct: CopyTest) {
        let (_ztempdir, zonepath) = fixture_dir(vec!["root/", "root/etc/", "root/etc/app.cfg"]);
        let (_stempdir, source_dir) = fixture_dir(ct.source_files);

        copy_to_zone(&zonepath, &source_dir.join(ct.copy), ct.to).unwrap();

        for path in ct.expect {
            let (expected_path, expected_content) = if path.contains("=") {
                let (p, c) = path.split_once('=').unwrap();
                (zonepath.join("root").join(p.trim_start_matches("/")), c)
            } else {
                (
                    zonepath.join("root").join(path.trim_start_matches("/")),
                    path.split('/').next_back().unwrap(),
                )
            };

            assert!(
                expected_path.exists(),
                "expected {expected_path} to be created from {}",
                ct.to
            );

            assert_eq!(expected_content, fs::read_to_string(expected_path).unwrap());
        }
    }

    #[test]
    fn test_copy_in_file_when_target_dir_exists() {
        // /etc exists: put file inside it, iterating over all the valid ways to define a target
        for target in [
            "/etc/myfile.conf",
            "etc/myfile.conf",
            "/etc", // trailing slash not required because dir exists
            "etc",
            "/etc/",
            "etc/",
        ] {
            assert_copy(CopyTest {
                source_files: vec!["myfile.conf"],
                copy: "myfile.conf",
                to: target,
                expect: vec!["/etc/myfile.conf"],
            });
        }
    }

    #[test]
    fn test_copy_in_file_when_target_dir_does_not_exist() {
        // create path and put file inside it
        for target in [
            "/usr/local/etc/app.cfg",
            "usr/local/etc/app.cfg",
            "/usr/local/etc/", // trailing slash required
            "usr/local/etc/",  // trailing slash required
        ] {
            assert_copy(CopyTest {
                source_files: vec!["app.cfg"],
                copy: "app.cfg",
                to: target,
                expect: vec!["/usr/local/etc/app.cfg"],
            });
        }
    }

    #[test]
    fn test_copy_in_file_clobbers_existing_file() {
        // clobber existing file
        assert_copy(CopyTest {
            source_files: vec!["app.cfg.new"],
            copy: "app.cfg.new",
            to: "/etc/app.cfg",
            expect: vec!["/etc/app.cfg=app.cfg.new"],
        });
    }

    #[test]
    fn test_copy_in_file_copies_nested_file() {
        assert_copy(CopyTest {
            source_files: vec!["a/b/c/file.txt"],
            copy: "a/b/c/file.txt",
            to: "/",
            expect: vec!["/file.txt=a/b/c/file.txt"],
        });
    }

    #[test]
    fn test_copy_in_file_copies_directory_trees() {
        // copy whole directory
        // a/ would put it *inside* another `a` directory
        for target in ["/", "a", "/a"] {
            assert_copy(CopyTest {
                source_files: vec!["a/b/file_1", "a/b/file_2", "a/c/file_3"],
                copy: "a",
                to: target,
                expect: vec![
                    "/a/b/file_1=a/b/file_1",
                    "/a/b/file_2=a/b/file_2",
                    "/a/c/file_3=a/c/file_3",
                ],
            });
        }
        // with trailing slash, `a` goes *in* `/opt/local`
        for target in ["/opt/local/", "opt/local/"] {
            assert_copy(CopyTest {
                source_files: vec!["a/b/file_1", "a/b/file_2", "a/c/file_3"],
                copy: "a",
                to: target,
                expect: vec![
                    "/opt/local/a/b/file_1=a/b/file_1",
                    "/opt/local/a/b/file_2=a/b/file_2",
                    "/opt/local/a/c/file_3=a/c/file_3",
                ],
            });
        }
        // without trailing slash, `a` *becomes* `/opt/local`
        for target in ["/opt/local", "opt/local"] {
            assert_copy(CopyTest {
                source_files: vec!["a/b/file_1", "a/b/file_2", "a/c/file_3"],
                copy: "a",
                to: target,
                expect: vec![
                    "/opt/local/b/file_1=a/b/file_1",
                    "/opt/local/b/file_2=a/b/file_2",
                    "/opt/local/c/file_3=a/c/file_3",
                ],
            });
        }
    }

    #[test]
    fn test_copy_in_errors_when_src_does_not_exist() {
        let (_ztempdir, zonepath) = fixture_dir(vec!["root/"]);

        assert!(
            copy_to_zone(&zonepath, &Utf8PathBuf::from("/nonexistent/ghost"), "/")
                .unwrap_err()
                .to_string()
                .contains("neither a file nor a directory"),
        );
    }
}
