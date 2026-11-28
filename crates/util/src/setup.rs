use crate::file::{self, FileMetadata, NameOrId};
use anyhow::Context;
use camino::Utf8PathBuf;
use common::constants::{
    GURP_DIR, GURP_DIR_CACHE, GURP_DIR_CONSOLE_LOG, GURP_DIR_REPORT, GURP_DIR_STATIC_FACTS,
    GURP_DIR_TMP,
};
use common::types::ApplyOpts;
use nix::unistd;
use os_types::FileMode;
use std::fs;

pub fn gurp_dirs() -> anyhow::Result<()> {
    if unistd::geteuid().is_root() {
        set_up_gurp_dirs()
    } else {
        Ok(())
    }
}

fn set_up_gurp_dirs() -> anyhow::Result<()> {
    for dir in [
        GURP_DIR,
        GURP_DIR_CACHE,
        GURP_DIR_CONSOLE_LOG,
        GURP_DIR_REPORT,
        GURP_DIR_STATIC_FACTS,
        GURP_DIR_TMP,
    ] {
        let path = Utf8PathBuf::from(dir);
        let metadata = FileMetadata {
            owner: &NameOrId::Name("root".to_owned()),
            group: &NameOrId::Name("root".to_owned()),
            mode: &FileMode::new("0700").context("can't make file mode in set_up_gurp_dirs()")?,
        };

        if !path.exists() {
            fs::create_dir_all(&path).context(format!("cannot create dir: {path}"))?;
        }

        file::ensure_metadata(&path, metadata, &ApplyOpts::default())
            .context(format!("can't ensure mode of {path}"))?;
    }

    Ok(())
}
