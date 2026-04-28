use crate::zone::bhyve;
use camino::Utf8PathBuf;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

pub type CopyInFiles = HashMap<Utf8PathBuf, String>;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Brand {
    Bhyve,
    Illumos,
    Ipkg,
    Lipkg,
    Lx,
    Pkgsrc,
    Sparse,
}

impl fmt::Display for Brand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Brand::Bhyve => "bhyve",
                Brand::Illumos => "illumos",
                Brand::Ipkg => "ipkg",
                Brand::Lipkg => "lipkg",
                Brand::Lx => "lx",
                Brand::Pkgsrc => "pkgsrc",
                Brand::Sparse => "sparse",
            }
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ZoneConfig {
    pub attr: Option<GurpZoneAttrs>,
    pub autoboot: bool,
    pub bhyve: Option<GurpZoneBhyve>,
    pub boot_after_install: bool,
    pub bootstrap: Option<BootstrapConf>,
    pub brand: Brand,
    pub capped_memory: Option<GurpZoneCappedMemory>,
    pub clone_from: Option<String>,
    pub copy_in: Option<CopyInFiles>,
    pub datasets: Option<Vec<String>>,
    pub dns: Option<GurpZoneDns>,
    pub exec_in: Option<Vec<String>>,
    pub final_state: Option<String>,
    pub fs: Option<GurpZoneFilesystems>,
    pub hostid: Option<String>,
    pub ip_type: Option<String>,
    pub limitpriv: Option<Vec<String>>,
    pub image: Option<String>,
    pub net: GurpZoneNetworks,
    pub pool: Option<String>,
    pub rctl: Option<GurpZoneRctls>,
    pub recreate: u8,
    pub zonepath: Utf8PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BootstrapConf {
    pub server: Option<String>,
    pub file: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpZoneBhyve {
    pub boot_volume: String,
    pub cloudinit_files: Option<Vec<Utf8PathBuf>>,
    pub cloudinit_struct: Option<Value>,
    pub image_format: Option<String>,
    pub ram: String,
    pub vcpus: u8,
    pub wait_for_boot: bool,
    pub acpi: bool,
    pub boot_rom: String,
}

#[derive(Debug, Deserialize)]
pub struct GurpZoneRctl {
    pub name: String,
    #[serde(rename = "priv")]
    pub rctl_priv: String,
    pub limit: u64,
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct GurpZoneCappedMemory {
    pub physical: String,
    pub swap: String,
}

#[derive(Debug, Deserialize)]
pub struct GurpZoneDns {
    pub domain: Option<String>,
    pub nameservers: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AttrValue {
    Str(String),
    Bool(bool),
    Number(u32),
    Float(f64),
}

impl fmt::Display for AttrValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttrValue::Str(s) => write!(f, "{s}"),
            AttrValue::Bool(b) => write!(f, "{b}"),
            AttrValue::Number(n) => write!(f, "{n}"),
            AttrValue::Float(n) => write!(f, "{n}"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GurpZoneAttr {
    pub name: String,
    #[serde(rename = "type")]
    pub attr_type: String,
    pub value: AttrValue,
}

type GurpZoneFilesystems = Vec<GurpZoneFilesystem>;
type GurpZoneAttrs = Vec<GurpZoneAttr>;
type GurpZoneRctls = Vec<GurpZoneRctl>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpZoneFilesystem {
    pub dir: Utf8PathBuf,
    pub special: Utf8PathBuf,
    #[serde(rename = "type")]
    pub fs_type: String,
    pub options: Option<Vec<String>>,
}

type GurpZoneNetworks = Vec<GurpZoneNetwork>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpZoneNetwork {
    pub physical: String,
    pub global_nic: String,
    pub allowed_address: Option<String>,
    pub defrouter: Option<String>,
}

impl GurpZoneBhyve {
    pub fn has_cloudinit(&self) -> bool {
        self.cloudinit_files.is_some() || self.cloudinit_struct.is_some()
    }
}

impl ZoneConfig {
    pub fn to_zonecfg(&self, uuid: &Uuid) -> String {
        let mut ret = "create -b\n".to_owned();

        ret.push_str(&format!("set brand={}\n", &self.brand));
        ret.push_str(&format!("set zonepath={}\n", &self.zonepath));
        ret.push_str(&format!("set autoboot={}\n", &self.autoboot));

        if let Some(ip_type) = &self.ip_type {
            ret.push_str(&format!("set ip-type={ip_type}\n"));
        }

        if let Some(pool) = &self.pool {
            ret.push_str(&format!("set pool={pool}\n"));
        }

        if let Some(hostid) = &self.hostid {
            ret.push_str(&format!("set hostid={hostid}\n"));
        }

        if let Some(limitpriv) = &self.limitpriv {
            ret.push_str(&format!("set limitpriv={}\n", limitpriv.join(",")));
        }

        for network_conf in &self.net {
            ret.push_str(&self.zone_net(network_conf));
        }

        if let Some(conf) = &self.dns {
            ret.push_str(&self.zone_dns(conf));
        }

        if let Some(fs_conf) = &self.fs {
            for conf in fs_conf {
                ret.push_str(&zone_fs!(conf));
            }
        }

        if let Some(datasets) = &self.datasets {
            for ds in datasets {
                ret.push_str(zone_dataset!(ds));
            }
        }

        if let Some(cap) = &self.capped_memory {
            ret.push_str(zone_capped_memory!(cap));
        }

        if let Some(attrs) = &self.attr {
            for attr in attrs {
                ret.push_str(zone_attr!(attr.name, attr.attr_type, attr.value));
            }
        }

        if let Some(rctls) = &self.rctl {
            for rctl in rctls {
                ret.push_str(zone_rctl!(rctl));
            }
        }

        if let Some(bhyve_config) = &self.bhyve {
            ret.push_str(&bhyve::zone_config(bhyve_config, uuid));
        }

        ret
    }

    fn zone_dns(&self, conf: &GurpZoneDns) -> String {
        let mut ret = String::new();

        if let Some(domain) = &conf.domain {
            ret.push_str(zone_attr!("dns-domain", "string", domain))
        }

        if let Some(nameservers) = &conf.nameservers {
            ret.push_str(zone_attr!("resolvers", "string", nameservers.join(",")))
        }

        ret
    }

    fn zone_net(&self, conf: &GurpZoneNetwork) -> String {
        let mut ret = "add net\n".to_owned();
        ret.push_str(&format!("\tset physical={}\n", conf.physical));
        ret.push_str(&format!("\tset global-nic={}\n", conf.global_nic));

        if let Some(addr) = &conf.allowed_address {
            ret.push_str(&format!("\tset allowed-address={addr}\n"));
        }

        if let Some(defrouter) = &conf.defrouter {
            ret.push_str(&format!("\tset defrouter={defrouter}\n"));
        }

        ret.push_str("end\n");
        ret
    }
}

#[cfg(test)]
mod test {
    use crate::zone::GurpZoneEnsure;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use tester::janet2json;
    use uuid::Uuid;

    #[test]
    fn test_config() {
        let json_def = janet2json(indoc! {r#"
            (zone/ensure "test-zone"
                :brand "lipkg"
                :autoboot false
                (zone/network "test_net0"
                           :allowed-address "192.168.1.33/24"
                           :defrouter "192.168.1.1")
                (zone/fs "/home" :special "/export/home")
                :capped-memory {
                    :physical "500M"
                    :swap "500M"
                }
                (zone/attr "numeric-attr" :value 123)
                (zone/attr "bool-attr" :type "boolean" :value false)
                (zone/attr "string-attr" :value "la-de-da")
                (zone/rctl "zone.max-swap"
                    :priv "privileged"
                    :limit 524288000
                    :action "deny")
                :datasets ["big/zone/fs" "fast/zone/fs"]
                :dns {:domain "lan.id264.net"
                      :nameservers ["192.168.1.53"
                                    "192.168.1.1"]})
                    "#
        });

        let expected_conf = indoc! {"
            create -b
            set brand=lipkg
            set zonepath=/zones/test-zone
            set autoboot=false
            add net
            \tset physical=test_net0
            \tset global-nic=auto
            \tset allowed-address=192.168.1.33/24
            \tset defrouter=192.168.1.1
            end
            add attr
            \tset name=dns-domain
            \tset type=string
            \tset value=lan.id264.net
            end
            add attr
            \tset name=resolvers
            \tset type=string
            \tset value=192.168.1.53,192.168.1.1
            end
            add fs
            \tset dir=/home
            \tset special=/export/home
            \tset type=lofs
            end
            add dataset
            \tset name=big/zone/fs
            end
            add dataset
            \tset name=fast/zone/fs
            end
            add capped-memory
            \tset physical=500M
            \tset swap=500M
            end
            add attr
            \tset name=numeric-attr
            \tset type=uint
            \tset value=123
            end
            add attr
            \tset name=bool-attr
            \tset type=boolean
            \tset value=false
            end
            add attr
            \tset name=string-attr
            \tset type=string
            \tset value=la-de-da
            end
            add rctl
            \tset name=zone.max-swap
            \tset value=(priv=privileged,limit=524288000,action=deny)\nend\n"};

        let sut: GurpZoneEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(expected_conf, sut.config.to_zonecfg(&Uuid::new_v4()));
    }
}
