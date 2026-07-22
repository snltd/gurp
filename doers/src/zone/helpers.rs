use crate::zone::config::ImageChecksum;
use crate::zone::constants::ZONEADM_FIELDS;
use crate::zone::control::ZoneadmState;
use crate::zone::types::{ZoneName, ZoneadmZones};
use anyhow::{Context, ensure};
use camino::Utf8PathBuf;
use common::constants::IMG_CACHE_DIR;
use common::constants::ZONEADM_BIN;
use common::types::{ApplyOpts, FileChecksum};
use url::Url;
use util::http::{self, RemoteFileCopy};

pub fn current_zone_list() -> anyhow::Result<ZoneadmZones> {
    parse_zone_list(&zone_list()?)
}

pub fn get_image(img_url: &Url, checksum: Option<&ImageChecksum>) -> anyhow::Result<Utf8PathBuf> {
    let seggies = img_url
        .path_segments()
        .with_context(|| format!("cannot get path segments of {}", img_url.as_str()))?;

    let img_fname = seggies
        .last()
        .with_context(|| format!("cannot get filename of {}", img_url.as_str()))?;

    let img_path = Utf8PathBuf::from(IMG_CACHE_DIR).join(img_fname);

    if img_path.exists() {
        tracing::debug!("found image at {img_path}");
    } else {
        let file_checksum = checksum
            .map(|cksum| literal_checksum(img_url, cksum))
            .transpose()?;

        tracing::debug!("no image at {img_path}: downloading");

        http::remote_file_to_disk(
            &RemoteFileCopy {
                url: img_url,
                path: &img_path,
                backup_suffix: None,
                checksum: file_checksum.as_ref(),
            },
            &ApplyOpts::default(),
        )?;
    }

    Ok(img_path)
}

fn literal_checksum(img_url: &str, checksum: &ImageChecksum) -> anyhow::Result<FileChecksum> {
    let literal_checksum = if checksum.value.starts_with(".") {
        http::remote_file_to_string(&format!("{img_url}{}", checksum.value))?
    } else {
        // we've been given the literal value
        checksum.value.clone()
    };

    Ok(FileChecksum {
        algorithm: checksum.sumtype.clone(),
        value: literal_checksum,
    })
}

fn zone_list() -> anyhow::Result<String> {
    cmd_output!(ZONEADM_BIN, "list", "-cp").context("failed to get zone list")
}

fn parse_zone_list(raw: &str) -> anyhow::Result<ZoneadmZones> {
    fn chunks_to_struct(chunks: &[&str]) -> anyhow::Result<(ZoneName, ZoneadmState)> {
        ensure!(
            chunks.len() == ZONEADM_FIELDS,
            "expected {ZONEADM_FIELDS} zoneadm fields. Got {}",
            chunks.len()
        );

        Ok((
            chunks[1].to_owned(),
            ZoneadmState {
                status: chunks[2].into(),
                path: chunks[3].into(),
                brand: chunks[5].into(),
                ip: chunks[6].into(),
            },
        ))
    }

    raw.lines()
        .map(|line| chunks_to_struct(&line.split(":").collect::<Vec<_>>()))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::zone::types::ZoneadmZones;
    use camino::Utf8PathBuf;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    #[test]
    fn test_zone_list() {
        let raw = indoc!(
        "0:global:running:/::ipkg:shared:0
        -:clean-zone:installed:/zones/clean-zone:311a4f36-779f-4d14-bc9d-c85cb9817327:lipkg:excl:216");

        let expected: ZoneadmZones = HashMap::from([
            (
                "global".to_owned(),
                ZoneadmState {
                    status: "running".to_owned(),
                    path: Utf8PathBuf::from("/"),
                    brand: "ipkg".to_owned(),
                    ip: "shared".to_owned(),
                },
            ),
            (
                "clean-zone".to_owned(),
                ZoneadmState {
                    status: "installed".to_owned(),
                    path: Utf8PathBuf::from("/zones/clean-zone"),
                    brand: "lipkg".to_owned(),
                    ip: "excl".to_owned(),
                },
            ),
        ]);

        assert_eq!(expected, parse_zone_list(raw).unwrap());
    }
}
