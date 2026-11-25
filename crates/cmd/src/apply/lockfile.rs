use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

pub struct ApplyLock {
    pub path: Utf8PathBuf,
}

impl ApplyLock {
    pub fn from(lockfile: &Utf8Path) -> Self {
        Self {
            path: lockfile.to_owned(),
        }
    }

    pub fn acquire(&self) -> anyhow::Result<bool> {
        if !self.path.exists() {
            self.create()?;
            return Ok(true);
        }

        let locked_pid = fs::read_to_string(&self.path)?;

        if self.pid_is_gurp(&locked_pid)? {
            tracing::error!("gurp is running as PID {locked_pid}");
            Ok(false)
        } else {
            self.release()?;
            self.create()?;
            Ok(true)
        }
    }

    pub fn release(&self) -> anyhow::Result<()> {
        tracing::debug!("removing lock file {}", self.path);
        fs::remove_file(&self.path).map_err(|e| e.into())
    }

    fn create(&self) -> anyhow::Result<()> {
        tracing::debug!("creating lock file {}", self.path);
        fs::write(&self.path, std::process::id().to_string()).map_err(|e| e.into())
    }

    fn pid_is_gurp(&self, pid_as_str: &str) -> anyhow::Result<bool> {
        let proc_dir = Utf8PathBuf::from("/proc").join(pid_as_str);
        if proc_dir.exists() {
            let cmd = fs::read_to_string(proc_dir.join("cmdline"))?;
            if cfg!(test) {
                Ok(cmd.contains("gurp"))
            } else {
                Ok(cmd.ends_with("gurp"))
            }
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use camino_tempfile_ext::prelude::*;

    #[test]
    fn test_create_and_remove_lock() {
        let lockdir = Utf8TempDir::new().unwrap();
        let lockfile = lockdir.child("gurp.lock");
        let lock_under_test = ApplyLock::from(lockfile.as_path());
        let my_pid = std::process::id().to_string();

        assert!(!lockfile.exists());
        assert!(lock_under_test.acquire().unwrap());
        assert!(lockfile.exists());
        assert_eq!(fs::read_to_string(lockfile.as_path()).unwrap(), my_pid);
        assert!(lock_under_test.release().is_ok());
        assert!(!lockfile.exists());
    }

    #[test]
    fn test_create_lock_when_stale_lock_exists() {
        let lockdir = Utf8TempDir::new().unwrap();
        let lockfile = lockdir.child("gurp.lock");
        lockfile.write_str("1").unwrap();
        let lock_under_test = ApplyLock::from(lockfile.as_path());
        let my_pid = std::process::id().to_string();

        assert!(lockfile.exists());
        assert!(lock_under_test.acquire().unwrap());
        assert!(lockfile.exists());
        assert_eq!(fs::read_to_string(lockfile.as_path()).unwrap(), my_pid);
        assert!(lock_under_test.release().is_ok());
        assert!(!lockfile.exists());
    }

    #[test]
    fn test_create_lock_when_lock_is_correct() {
        let my_pid = std::process::id().to_string();
        let lockdir = Utf8TempDir::new().unwrap();
        let lockfile = lockdir.child("gurp.lock");
        lockfile.write_str(&my_pid).unwrap();
        let lock_under_test = ApplyLock::from(lockfile.as_path());

        assert!(lockfile.exists());
        assert!(!lock_under_test.acquire().unwrap());
    }
}
