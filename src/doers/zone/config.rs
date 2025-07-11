use camino::Utf8PathBuf;
use indoc::formatdoc;
use serde::Deserialize;

// Turns Janet into Rust into zonecfg input

macro_rules! set {
    // indented for subsections
    ($ret:expr, $conf:expr, indent: $indent:expr, $($field:ident),+ $(,)?) => {
        $(
            $ret.push_str(&format!(
                "{}set {}={}\n",
                $indent,
                stringify!($field),
                $conf.$field
            ));
        )+
    };

    ($ret:expr, $conf:expr, $($field:ident),+ $(,)?) => {
        set!($ret, $conf, indent: "", $($field),+);
    };
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpZoneConfig {
    pub brand: String,
    pub autoboot: bool,
    pub zonepath: Utf8PathBuf,
    pub networks: Vec<GurpZoneNetwork>,
    pub datasets: Option<Vec<String>>,
    pub capped_memory: Option<GurpZoneCappedMemory>,
    pub dns: Option<GurpZoneDns>,
    pub fs: Option<Vec<GurpZoneFs>>,
    pub run_cmd: Option<Vec<String>>,
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
#[serde(rename_all = "kebab-case")]
pub struct GurpZoneFs {
    pub dir: Utf8PathBuf,
    pub special: Utf8PathBuf,
    #[serde(rename = "type")]
    pub fs_type: String,
}

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

        set!(ret, self, brand, zonepath, autoboot);

        for network_conf in &self.networks {
            ret.push_str(&self.zone_network(network_conf));
        }

        if let Some(dns_conf) = &self.dns {
            ret.push_str(&self.zone_dns(dns_conf));
        }

        if let Some(fs_conf) = &self.fs {
            for fs in fs_conf {
                ret.push_str(&self.zone_fs(fs));
            }
        }

        if let Some(datasets) = &self.datasets {
            for ds in datasets {
                ret.push_str(&self.zone_dataset(ds));
            }
        }

        if let Some(memcap) = &self.capped_memory {
            for ds in datasets {
                ret.push_str(&self.zone_dataset(ds));
            }
        }

        ret
    }

    // We may want to add "create dataset" logic here
    fn zone_dataset(&self, ds_name: &str) -> String {
        format!("add dataset\n  set name={ds_name}\nend\n")
    }

    fn zone_fs(&self, conf: &GurpZoneFs) -> String {
        formatdoc! { "add fs
          set dir={}
          set special={}
          set type={}
        end\n", conf.dir, conf.special, conf.fs_type }
    }

    fn string_attr(&self, name: &str, value: &str) -> String {
        formatdoc! { "add attr
       set name={}
       set type=string
       set value={}
     end\n", name, value}
    }

    fn zone_dns(&self, conf: &GurpZoneDns) -> String {
        format!(
            "{}{}",
            self.string_attr("dns-domain", &conf.domain),
            self.string_attr("resolvers", &conf.nameservers.join(","))
        )
    }

    fn zone_network(&self, conf: &GurpZoneNetwork) -> String {
        let mut ret = "add network\n".to_owned();
        set!(ret, conf, indent: "  ", physical, global_nic);

        if let Some(addr) = &conf.allowed_address {
            ret.push_str(&format!("  allowed-address={addr}\n"));
        }

        if let Some(defrouter) = &conf.defrouter {
            ret.push_str(&format!("  defrouter={defrouter}\n"));
        }

        ret.push_str("end\n");
        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::doers::zone::GurpZoneEnsure;
    use crate::test_utils::spec_helper::janet2json;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_config() {
        let json_def = janet2json(indoc! {r#"
            (zone/ensure "zone-with-everything"
            :brand "lipkg"
            :zonepath "/zones/serv-fs"
            :autoboot false
            :networks [{:physical "fs_net0"
                       :global-nic "auto"
                       :allowed-address "192.168.1.33/24"
                       :defrouter "192.168.1.1"}]
            :fs [{:dir "/home"
                  :special "/export/home"
                  :type "lofs"}]
            :capped-memory {
                :physical "500M"
                :swap "500M"
            }
            :datasets ["big/zone/fs" "fast/zone/fs"]
            :dns {:domain "lan.id264.net"
                  :nameservers ["192.168.1.53"
                                "192.168.1.1"]})
                    "#
        });

        let expected_conf = indoc! {"
            create -b
            set brand=lipkg
            set zonepath=/zones/serv-fs
            set autoboot=false
            add network
              set physical=fs_net0
              set global_nic=auto
              allowed-address=192.168.1.33/24
              defrouter=192.168.1.1
            end
            add attr
              set name=dns-domain
              set type=string
              set value=lan.id264.net
            end
            add attr
              set name=resolvers
              set type=string
              set value=192.168.1.53,192.168.1.1
            end
            add fs
              set dir=/home
              set special=/export/home
              set type=lofs
            end
            add dataset
              set name=big/zone/fs
            end
            add dataset
              set name=fast/zone/fs
            end
            add capped-memory
              set physical=500M
              set swap=500M
            end
            "};

        let sut: GurpZoneEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(expected_conf, sut.config.to_zonecfg());
    }
}
