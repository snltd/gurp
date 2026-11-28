use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use common::constants::GURP_DIR_STATIC_FACTS;
use std::fs;

pub fn get(fact_name: &str) -> anyhow::Result<Option<String>> {
    let fact_path = fact_path(&Utf8PathBuf::from(GURP_DIR_STATIC_FACTS), fact_name);

    if fact_path.exists() {
        Ok(Some(fs::read_to_string(fact_path)?))
    } else {
        Ok(None)
    }
}

pub fn create(
    fact_name: &str,
    fact_value: &str,
    zonepath: Option<&Utf8Path>,
) -> anyhow::Result<()> {
    let facts_dir = if let Some(zone_path) = zonepath {
        zone_path
            .join("root")
            .join(GURP_DIR_STATIC_FACTS.trim_start_matches("/"))
    } else {
        Utf8PathBuf::from(GURP_DIR_STATIC_FACTS)
    };

    let fact_path = fact_path(&facts_dir, fact_name);

    if !facts_dir.exists() {
        tracing::info!("creating {facts_dir}");
        fs::create_dir_all(&facts_dir)
            .with_context(|| format!("failed to create etc_dir {facts_dir}"))?;
    }

    tracing::info!("creating fact: {fact_name} = {fact_value} [{fact_path}]");

    fs::write(fact_path, format!("{fact_value}\n"))?;
    Ok(())
}

fn fact_path(dir: &Utf8Path, fact_name: &str) -> Utf8PathBuf {
    dir.join(fact_name)
}
