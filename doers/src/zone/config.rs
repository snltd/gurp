use camino::Utf8PathBuf;
use indoc::formatdoc;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

// Turns Janet into Rust into zonecfg input

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpZoneConfig {
    pub attr: Option<GurpZoneAttrs>,
    pub autoboot: bool,
    pub boot_after_install: bool,
    pub bootstrap_from: Option<Utf8PathBuf>,
    pub brand: String,
    pub capped_memory: Option<GurpZoneCappedMemory>,
    pub clone_from: Option<String>,
    pub copy_in: Option<HashMap<Utf8PathBuf, String>>,
    pub datasets: Option<Vec<String>>,
    pub dns: Option<GurpZoneDns>,
    pub exec_in: Option<Vec<String>>,
    pub final_state: Option<String>,
    pub fs: Option<GurpZoneFilesystems>,
    #[serde(rename = "lx-image")]
    pub image: Option<String>,
    pub net: GurpZoneNetworks,
    pub rctl: Option<GurpZoneRctls>,
    pub recreate: u8,
    pub zonepath: Utf8PathBuf,
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
    pub domain: String,
    pub nameservers: Vec<String>,
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

impl GurpZoneConfig {
    pub fn to_zonecfg(&self) -> String {
        let mut ret = "create -b\n".to_owned();

        ret.push_str(&format!("set brand={}\n", &self.brand));
        ret.push_str(&format!("set zonepath={}\n", &self.zonepath));
        ret.push_str(&format!("set autoboot={}\n", &self.autoboot));

        for network_conf in &self.net {
            ret.push_str(&self.zone_net(network_conf));
        }

        if let Some(conf) = &self.dns {
            ret.push_str(&self.zone_dns(conf));
        }

        if let Some(fs_conf) = &self.fs {
            for conf in fs_conf {
                ret.push_str(&self.zone_fs(conf));
            }
        }

        if let Some(datasets) = &self.datasets {
            for ds in datasets {
                ret.push_str(&self.zone_dataset(ds));
            }
        }

        if let Some(conf) = &self.capped_memory {
            ret.push_str(&self.zone_capped_memory(conf));
        }

        if let Some(attrs) = &self.attr {
            for attr in attrs {
                ret.push_str(&self.zone_attr(attr));
            }
        }

        if let Some(rctls) = &self.rctl {
            for rctl in rctls {
                ret.push_str(&self.zone_rctl(rctl));
            }
        }

        ret
    }

    // We may want to add "create dataset" logic here
    fn zone_dataset(&self, ds_name: &str) -> String {
        format!("add dataset\n\tset name={ds_name}\nend\n")
    }

    fn zone_fs(&self, conf: &GurpZoneFilesystem) -> String {
        let mut ret = formatdoc! { "add fs
        \tset dir={}
        \tset special={}
        \tset type={}\n" , conf.dir, conf.special, conf.fs_type };

        if let Some(options) = &conf.options {
            ret.push_str(&format!("\tset options={}\n", options.join(",")));
        }

        ret.push_str("end\n");
        ret
    }

    fn zone_capped_memory(&self, conf: &GurpZoneCappedMemory) -> String {
        formatdoc! { "add capped-memory
        \tset physical={}
        \tset swap={}
        end\n", conf.physical, conf.swap}
    }

    fn zone_attr(&self, conf: &GurpZoneAttr) -> String {
        formatdoc! { "add attr
     \tset name={}
     \tset type={}
     \tset value={}
     end\n", conf.name, conf.attr_type, conf.value}
    }

    fn zone_rctl(&self, conf: &GurpZoneRctl) -> String {
        formatdoc! { "add rctl
     \tset name={}
     \tset value=(priv={},limit={},action={})
     end\n", conf.name, conf.rctl_priv, conf.limit, conf.action}
    }

    fn string_attr(&self, name: &str, value: &str) -> String {
        formatdoc! { "add attr
     \tset name={}
     \tset type=string
     \tset value={}
     end\n", name, value}
    }

    fn zone_dns(&self, conf: &GurpZoneDns) -> String {
        format!(
            "{}{}",
            self.string_attr("dns-domain", &conf.domain),
            self.string_attr("resolvers", &conf.nameservers.join(","))
        )
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

    #[test]
    fn test_config() {
        let json_def = janet2json(indoc! {r#"
            (zone/ensure "test-zone"
                :brand "lipkg"
                :autoboot false
                (zone-network "test_net0"
                           :allowed-address "192.168.1.33/24"
                           :defrouter "192.168.1.1")
                (zone-fs "/home" :special "/export/home")
                :capped-memory {
                    :physical "500M"
                    :swap "500M"
                }
                (zone-attr "numeric-attr" :value 123)
                (zone-attr "bool-attr" :type "boolean" :value false)
                (zone-attr "string-attr" :value "la-de-da")
                (zone-rctl "zone.max-swap"
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

        assert_eq!(expected_conf, sut.config.to_zonecfg());
    }
}
