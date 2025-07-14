use crate::common::types::ApplySummary;
use camino::Utf8PathBuf;
use std::sync::LazyLock;

pub const GURP_LIB: &str = include_str!("../../janet_src/lib/gurp.janet");
pub const JSON_LIB: &str = include_str!("../../janet_src/lib/encode.janet");
pub const GURP_DEFAULTS: &str = include_str!("../../janet_src/lib/defaults.janet");

pub const ONE_RESOURCE_ONE_CHANGE: ApplySummary = ApplySummary {
    resources: 1,
    changes: 1,
    errors: 0,
};

pub const ONE_RESOURCE_NOOP: ApplySummary = ApplySummary {
    resources: 1,
    changes: 1,
    errors: 0,
};

pub const ONE_RESOURCE_NO_CHANGE: ApplySummary = ApplySummary {
    resources: 1,
    changes: 0,
    errors: 0,
};

pub const ONE_RESOURCE_ONE_ERROR: ApplySummary = ApplySummary {
    resources: 1,
    changes: 0,
    errors: 1,
};

pub const NO_RESOURCES_TO_CHANGE: ApplySummary = ApplySummary {
    resources: 0,
    changes: 0,
    errors: 0,
};

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
