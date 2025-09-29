use std::time::Duration;

pub const LX_RELEASES_URL: &str = "https://api.github.com/repos/omniosorg/lx-images/releases";
pub const ZONEADM_FIELDS: usize = 8;
pub const READY_SVC: &str = "svc:/milestone/multi-user-server:default";
pub const STATE_WAIT_INTERVAL: Duration = Duration::from_secs(1);
pub const STATE_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
pub const READINESS_WAIT_INTERVAL: Duration = Duration::from_secs(2);
pub const READINESS_WAIT_TIMEOUT_NATIVE: Duration = Duration::from_secs(60);
pub const READINESS_WAIT_TIMEOUT_BHYVE: Duration = Duration::from_secs(300);
