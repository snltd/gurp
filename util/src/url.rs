use anyhow::Context;
use url::Url;

pub trait Filename {
    fn filename(&self) -> anyhow::Result<String>;
}

impl Filename for Url {
    fn filename(&self) -> anyhow::Result<String> {
        let mut seggies = self
            .path_segments()
            .with_context(|| format!("cannot get path segments of {}", self.as_str()))?;

        Ok(seggies
            .next_back()
            .with_context(|| format!("cannot get filename of {}", self.as_str()))?
            .to_string())
    }
}
