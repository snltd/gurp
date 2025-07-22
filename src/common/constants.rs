use crate::common::types::ApplySummary;
use camino::Utf8PathBuf;
use std::sync::LazyLock;

pub const GURP_LIB: &str = include_str!("../../janet_src/lib/gurp.janet");
pub const JSON_LIB: &str = include_str!("../../janet_src/lib/encode.janet");
pub const GURP_DEFAULTS: &str = include_str!("../../janet_src/lib/defaults.janet");

pub const MANIFEST_DIR: &str = "/opt/site/lib/smf/manifest";

pub const CRONTAB_BIN: &str = "/bin/crontab";
pub const DISPADMIN_BIN: &str = "/usr/sbin/dispadmin";
pub const GEM_BIN: &str = "/opt/ooce/bin/gem";
pub const PKG_BIN: &str = "/bin/pkg";
pub const PS_BIN: &str = "/bin/ps";
pub const SHARECTL_BIN: &str = "/usr/sbin/sharectl";
pub const SMBADM_BIN: &str = "/usr/sbin/smbadm";
pub const SVCADM_BIN: &str = "/usr/sbin/svcadm";
pub const SVCCFG_BIN: &str = "/usr/sbin/svccfg";
pub const SVCPROP_BIN: &str = "/usr/bin/svcprop";
pub const SVCS_BIN: &str = "/bin/svcs";
pub const USERADD_BIN: &str = "/usr/sbin/useradd";
pub const USERDEL_BIN: &str = "/usr/sbin/userdel";
pub const USERMOD_BIN: &str = "/usr/sbin/usermod";
pub const ZFS_BIN: &str = "/usr/sbin/zfs";
pub const ZLOGIN_BIN: &str = "/usr/sbin/zlogin";
pub const ZONEADM_BIN: &str = "/usr/sbin/zoneadm";
pub const ZONECFG_BIN: &str = "/usr/sbin/zonecfg";

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
