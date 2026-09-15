use std::sync::LazyLock;
use std::time::Duration;
use url::Url;

pub const READY_SVC: &str = "svc:/milestone/multi-user-server:default";
pub const STATE_WAIT_INTERVAL: Duration = Duration::from_secs(1);
pub const STATE_WAIT_TIMEOUT: Duration = Duration::from_secs(60);
pub const READINESS_WAIT_INTERVAL: Duration = Duration::from_secs(2);
pub const READINESS_WAIT_TIMEOUT_NATIVE: Duration = Duration::from_secs(60);
pub const READINESS_WAIT_TIMEOUT_EMULATED: Duration = Duration::from_secs(600);

pub const ZONEADM_FIELDS: usize = 8;

pub static OMNIOS_RELEASES_URL: LazyLock<Url> = LazyLock::new(|| {
    Url::parse("https://downloads.omnios.org/media/stable/").expect("invalid OMNIOS_RELEASES_URL")
});

pub static LX_RELEASES_URL: LazyLock<Url> = LazyLock::new(|| {
    Url::parse("https://api.github.com/repos/omniosorg/lx-images/releases")
        .expect("invalid LS_RELEASES_URL")
});
