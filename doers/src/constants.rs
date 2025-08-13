use camino::Utf8PathBuf;
use std::sync::LazyLock;

pub const MANIFEST_DIR: &str = "/opt/site/lib/smf/manifest";
pub const GEM_BIN_DIR: &str = "/opt/ooce/bin";

pub static PROTECTED_DIRS: LazyLock<Vec<Utf8PathBuf>> = LazyLock::new(|| {
    vec![
        Utf8PathBuf::from("/"),
        Utf8PathBuf::from("/bin"),
        Utf8PathBuf::from("/etc"),
        Utf8PathBuf::from("/lib"),
        Utf8PathBuf::from("/sbin"),
        Utf8PathBuf::from("/usr"),
        Utf8PathBuf::from("/usr/lib"),
    ]
});

pub static PROTECTED_FILES: LazyLock<Vec<Utf8PathBuf>> =
    LazyLock::new(|| vec![Utf8PathBuf::from("/bin/ps")]);

pub static PROTECTED_USERS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        "root", "daemon", "bin", "sys", "adm", "lp", "uucp", "nuucp", "dladm", "netadm", "netcfg",
        "listen", "gdm", "unknown", "nobody", "noaccess", "nobody4", "pkg5srv",
    ]
});

pub static PROTECTED_GROUPS: LazyLock<Vec<&str>> =
    LazyLock::new(|| vec!["root", "other", "bin", "sys", "adm", "tty", "daemon"]);
