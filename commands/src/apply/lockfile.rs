use anyhow::Context;
use camino::Utf8PathBuf;
use std::fs;
use sysinfo::{Pid, System};

pub struct ApplyLock {
    pub path: Utf8PathBuf,
}

impl ApplyLock {
    pub fn from(lockfile: &str) -> Self {
        Self {
            path: Utf8PathBuf::from(lockfile),
        }
    }

    pub fn exists(&self) -> bool {
        if self.path.exists() {
            tracing::debug!("found lockfile: {}", self.path);
            true
        } else {
            false
        }
    }

    pub fn create(&self) -> anyhow::Result<()> {
        if !self.exists() {
            tracing::debug!("creating lock file {}", self.path);
            fs::write(&self.path, self.my_pid())
                .with_context(|| format!("failed to create lock at {}", self.path))?;
        }
        Ok(())
    }

    pub fn remove(&self) -> anyhow::Result<()> {
        if self.exists() {
            tracing::debug!("removing lock file {}", self.path);
            fs::remove_file(&self.path)
                .with_context(|| format!("failed to remove lock at {}", self.path))?;
        }
        Ok(())
    }

    pub fn is_locked(&self) -> anyhow::Result<bool> {
        if self.exists() {
            let sys = System::new_all();
            let processes = sys.processes();
            // Dumb check.
            if processes.contains_key(&self.locked_pid()?) {
                return Ok(true);
            } else {
                tracing::info!("removing stale lockfile");
                self.remove()?;
            }
        }

        Ok(false)
    }

    fn locked_pid(&self) -> anyhow::Result<Pid> {
        let locked_pid_string = fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read lock at {}", self.path))?;
        Ok(Pid::from(locked_pid_string.trim().parse::<usize>()?))
    }

    fn my_pid(&self) -> String {
        std::process::id().to_string()
    }
}
