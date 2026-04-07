use crate::zone::constants::ZONEADM_FIELDS;
use crate::zone::control::ZoneadmState;
use crate::zone::types::{ZoneName, ZoneadmZones};
use anyhow::{Context, ensure};
use common::constants::ZONEADM_BIN;

pub fn current_zone_list() -> anyhow::Result<ZoneadmZones> {
    parse_zone_list(&zone_list()?)
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
    // .collect::<anyhow::Result<HashMap<_, _>>>()
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
