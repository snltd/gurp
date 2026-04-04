use anyhow::{Context, bail};
use serde_json::Value;

pub fn formatted(raw_json: &str) -> anyhow::Result<String> {
    match pretty(raw_json) {
        Ok(json) => Ok(json),
        Err(e) => {
            tracing::error!("JSON processing error: {}", e);
            tracing::error!(raw_json);
            bail!("END");
        }
    }
}

pub fn pretty(raw_json: &str) -> anyhow::Result<String> {
    let value: Value = serde_json::from_str(raw_json).context("could not deserialize JSON")?;
    Ok(serde_json::to_string_pretty(&value)?)
}
