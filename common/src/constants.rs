use crate::types::ApplySummary;

pub const GURP_VERSION: &str = env!("CARGO_PKG_VERSION");

// Binaries gurp is allowed to run
pub const APK_BIN: &str = "/sbin/apk";
pub const CRONTAB_BIN: &str = "/bin/crontab";
pub const DISPADMIN_BIN: &str = "/usr/sbin/dispadmin";
pub const DLADM_BIN: &str = "/usr/sbin/dladm";
pub const GEM_BIN: &str = "/opt/ooce/bin/gem";
pub const GROUPADD_BIN: &str = "/usr/sbin/groupadd";
pub const GROUPDEL_BIN: &str = "/usr/sbin/groupdel";
pub const GROUPMOD_BIN: &str = "/usr/sbin/groupmod";
pub const GROUPS_BIN: &str = "/bin/groups";
pub const MKISOFS_BIN: &str = "/bin/mkisofs";
pub const PKG_BIN: &str = "/bin/pkg";
pub const PKGIN_BIN: &str = "/opt/local/bin/pkgin";
pub const PROFILES_BIN: &str = "/bin/profiles";
pub const PS_BIN: &str = "/bin/ps";
pub const QEMU_IMG_BIN: &str = "/opt/ooce/bin/qemu-img";
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
pub const ZFS_LX_BIN: &str = "/native/usr/sbin/zfs";
pub const ZLOGIN_BIN: &str = "/usr/sbin/zlogin";
pub const ZONEADM_BIN: &str = "/usr/sbin/zoneadm";
pub const ZONECFG_BIN: &str = "/usr/sbin/zonecfg";

pub const ONE_RESOURCE_ONE_CHANGE: ApplySummary = ApplySummary {
    resources: 1,
    changes: 1,
};

pub const ONE_RESOURCE_NOOP: ApplySummary = ApplySummary {
    resources: 1,
    changes: 1,
};

pub const ONE_RESOURCE_NO_CHANGE: ApplySummary = ApplySummary {
    resources: 1,
    changes: 0,
};

pub const NO_RESOURCES_TO_CHANGE: ApplySummary = ApplySummary {
    resources: 0,
    changes: 0,
};

pub const IMG_CACHE_DIR: &str = "/var/tmp";
