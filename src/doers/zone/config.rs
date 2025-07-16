use camino::Utf8PathBuf;
use indoc::formatdoc;
use serde::Deserialize;

// Turns Janet into Rust into zonecfg input

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpZoneConfig {
    pub clone_from: Option<String>,
    pub brand: String,
    pub autoboot: bool,
    pub zonepath: Utf8PathBuf,
    pub networks: GurpZoneNetworks,
    pub datasets: Option<Vec<String>>,
    pub capped_memory: Option<GurpZoneCappedMemory>,
    pub dns: Option<GurpZoneDns>,
    pub fs: Option<GurpZoneFilesystems>,
    pub exec: Option<Vec<String>>,
    pub boot_after_install: bool,
    pub bootstrap_from: Option<Utf8PathBuf>,
    pub recreate: u8,
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

type GurpZoneFilesystems = Vec<GurpZoneFilesystem>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GurpZoneFilesystem {
    pub dir: Utf8PathBuf,
    pub special: Utf8PathBuf,
    #[serde(rename = "type")]
    pub fs_type: String,
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

        for network_conf in &self.networks {
            ret.push_str(&self.zone_network(network_conf));
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

        ret
    }

    // We may want to add "create dataset" logic here
    fn zone_dataset(&self, ds_name: &str) -> String {
        format!("add dataset\n\tset name={ds_name}\nend\n")
    }

    fn zone_fs(&self, conf: &GurpZoneFilesystem) -> String {
        formatdoc! { "add fs
        \tset dir={}
        \tset special={}
        \tset type={}
        end\n", conf.dir, conf.special, conf.fs_type }
    }

    fn zone_capped_memory(&self, conf: &GurpZoneCappedMemory) -> String {
        formatdoc! { "add capped-memory
        \tset physical={}
        \tset swap={}
        end\n", conf.physical, conf.swap}
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

    fn zone_network(&self, conf: &GurpZoneNetwork) -> String {
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
    use crate::doers::zone::GurpZoneEnsure;
    use crate::test_utils::spec_helper::janet2json;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

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
            "};

        let sut: GurpZoneEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(expected_conf, sut.config.to_zonecfg());
    }
}
