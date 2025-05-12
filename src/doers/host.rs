// use crate::unpack_janet_table;
// use crate::utils::janet_helpers as j;
// use crate::utils::module;
use crate::debug;
use crate::utils::types::Opts;
use anyhow::{Context, anyhow};
// use camino::Utf8PathBuf;
use janetrs::JanetStruct;
// use crate::unpack_janet_table;
// use janetrs::JanetType::Table;
// use janetrs::client::JanetClient;
// use janetrs::env::CFunOptions;
// use janetrs::{Janet, JanetArgs, TaggedJanet};
use janetrs::TaggedJanet;
use janetrs::{Janet, client::JanetClient, env::CFunOptions};
use std::collections::HashMap;
// use crate::utils::janet_helpers;
use serde_json::{Map, Value};

type HostMetadata = HashMap<String, String>;
type Resource = HashMap<String, String>;
type ResourceType = String;
type HostResources = HashMap<ResourceType, Vec<Resource>>;
type HostVars = HashMap<String, String>;

#[derive(Debug)]
struct HostConfig {
    metadata: HostMetadata,
    resources: HostResources,
    vars: Option<HostVars>,
}

pub fn janet_to_json(j: &Janet) -> Value {
    // I'm going to leave the :s at the beginning of the key names for now, because it will
    // make it clear we're talking about user data.
    match j.unwrap() {
        TaggedJanet::Nil => Value::Null,
        TaggedJanet::Boolean(b) => Value::Bool(b),
        TaggedJanet::Number(n) => match serde_json::Number::from_f64(n) {
            Some(num) => Value::Number(num),
            None => Value::Null,
        },
        TaggedJanet::String(s) => Value::String(s.to_string()),
        TaggedJanet::Symbol(s) => Value::String(s.to_string()),
        TaggedJanet::Keyword(k) => Value::String(k.to_string()),
        TaggedJanet::Array(arr) => {
            let vec = arr.iter().map(janet_to_json).collect();
            Value::Array(vec)
        }
        TaggedJanet::Tuple(tup) => {
            let vec = tup.iter().map(janet_to_json).collect();
            Value::Array(vec)
        }
        TaggedJanet::Table(tab) => {
            let mut map = Map::new();
            for (k, v) in tab.iter() {
                let key = k.to_string();
                map.insert(key, janet_to_json(v));
            }
            Value::Object(map)
        }
        TaggedJanet::Struct(tab) => {
            let mut map = Map::new();
            for (k, v) in tab.iter() {
                let key = k.to_string();
                map.insert(key, janet_to_json(v));
            }
            Value::Object(map)
        }
        // I don't think we'll need any more exotic types
        other => Value::String(format!("<{:?}>", other)),
    }
}

// fn unpack_struct(table: &Janet) -> anyhow::Result<HashMap<String, String>> {
//     match table.unwrap() {
//         TaggedJanet::Struct(res) => {
//             let mut ret = HashMap::new();
//             for (k, v) in res {
//                 ret.insert(
//                     k.unwrap().to_string().replacen(":", "", 1),
//                     v.unwrap().to_string(),
//                 );
//             }

//             Ok(ret)
//         }
//         TaggedJanet::Table(res) => {
//             let mut ret = HashMap::new();
//             for (k, v) in res {
//                 ret.insert(
//                     k.unwrap().to_string().replacen(":", "", 1),
//                     v.unwrap().to_string(),
//                 );
//             }

//             Ok(ret)
//         }
//         _ => Err(anyhow!(format!("Expected Janet struct, got: {:?}", table))),
//     }
// }

// fn extract_resources(table: &Janet) -> anyhow::Result<HostResources> {
//     println!("{:?}", table);
//     println!("{:?}", janet_to_json(table));
//     todo!()
// }

// fn _extract_resources(table: &Janet) -> anyhow::Result<HostResources> {
//     match table.unwrap() {
//         TaggedJanet::Struct(res) => {
//             let mut ret = HashMap::new();
//             for (resource_type, resources) in res {
//                 let resource_type = resource_type.unwrap().to_string().replacen(":", "", 1);
//                 let resource_list = match resources.unwrap() {
//                     TaggedJanet::Array(res_list) => {
//                         res_list.iter().map(|r| unpack_struct(r).unwrap()).collect()
//                     }
//                     _ => {
//                         return Err(anyhow!(format!(
//                             "{} resources are not a Janet array",
//                             resource_type
//                         )));
//                     }
//                 };

//                 ret.insert(resource_type, resource_list);
//             }

//             Ok(ret)
//         }
//         _ => Err(anyhow!("Resources is not a Janet struct")),
//     }
// }

// fn extract_data(table: &Janet) -> anyhow::Result<HostConfig> {
//     let host_metadata = janet_to_json(table);

//     let janet_metadata = table
//         .get(Janet::from(":metadata"))
//         .context("Host config has no metadata")?;

//     let vars = match table.get(Janet::from(":vars")) {
//         Some(janet_vars) => Some(unpack_struct(janet_vars)?),
//         None => None,
//     };

//     let janet_resources = table
//         .get(Janet::from(":resources"))
//         .context("Host config has no resources")?;

//     Ok(HostConfig {
//         metadata: unpack_struct(janet_metadata)?,
//         vars,
//         resources: extract_resources(janet_resources)?,
//     })
// }

#[janetrs::janet_fn(arity(fix(1)))]
fn machine_config_handler(config_table: &mut [Janet]) -> Janet {
    let values = janet_to_json(&config_table[0]);

    let metadata: HostMetadata = serde_json::from_value(values[":metadata"].clone()).unwrap();
    let vars = values[":vars"].clone();
    let resources = values[":resources"].clone();

    println!("{:#?}", vars);

    Janet::nil()
}

fn hbar() -> String {
    "-".repeat(80)
}

fn setup_bindings(client: &mut JanetClient, opts: &Opts) {
    debug!(opts, "Setting up CFunction binding for machine-config");
    client.add_c_fn(CFunOptions::new(
        c"run-machine-configuration",
        machine_config_handler_c,
    ));
}

pub fn configure(
    janet_host_config: String,
    client: &mut JanetClient,
    opts: &Opts,
) -> anyhow::Result<bool> {
    setup_bindings(client, opts);
    // verbose!(opts, "Configuring {}", host_config.name);
    debug!(
        opts,
        "Janet host config follows:\n{}\n{}{}",
        hbar(),
        janet_host_config,
        hbar()
    );

    let result = client.run(janet_host_config)?;
    Ok(true)
}

/*
// The host doer is the boss. It collects a top-level Janet host definition, which it turns into
// a HostConfig struct. This is used as the root of the host configuration, fanning out across the
// included modules.

#[derive(Debug, PartialEq)]
pub struct HostConfig {
    pub path: Utf8PathBuf,
    pub name: String,
    pub vars: Option<VarMap>,
    pub modules: Vec<String>,
}

pub fn configure(host_config: HostConfig, opts: &Opts) -> anyhow::Result<bool> {
    verbose!(opts, "Configuring {}", host_config.name);
    debug!(opts, "Raw host config: {:?}", host_config);

    let module_dirs = match &opts.module_dirs {
        Some(path) => path.to_owned(),
        None => host_config
            .path
            .parent()
            .context("could not find host file parent")?
            .to_string(),
    };

    for module in &host_config.modules {
        if let Some(path) = module::find(module, &module_dirs, opts) {
            module::process(&path, opts)?;
        } else {
            return Err(anyhow!("Failed to load module '{}'", module));
        }
    }

    Ok(true)
}

pub fn define_host_config(
    path: &Utf8PathBuf,
    client: &mut JanetClient,
    server_desc: &str,
) -> anyhow::Result<HostConfig> {
    // The user defines a host as a chunk of Janet. We parse it here, and we'll do the heavy lifting
    // of finding and running the Janet which configures the host.
    //
    // So all we need to do is turn a definition into a name and a table.
    let inject_macro = "(defmacro host [hostname & args] ~[,hostname (table ,;args)])";
    let server_desc_lisp = format!("{}\n{}", inject_macro, server_desc);
    let result = client.run(server_desc_lisp)?;

    let (host_name, config_table) = j::unpack_object(&result)?;

    let modules = config_table
        .get(janetrs::Janet::keyword("modules".into()))
        .context("Cannot find a list of modules")?;

    let modules = j::unpack_tuple_of_strings(modules)?;

    let vars = match config_table.get(janetrs::Janet::keyword("vars".into())) {
        Some(janet_vars) => Some(j::unpack_var_struct(janet_vars)?),
        None => None,
    };

    Ok(HostConfig {
        path: path.clone(),
        name: host_name,
        vars,
        modules,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::janet_runner;

    #[test]
    fn test_define_host_config() {
        let path = Utf8PathBuf::from("dummy_path");
        let user_input = r#"(host "serv"
                            :vars {
                                :var_a "value_a"
                                :var_b 123456789
                            }
                            :modules [
                                "physical"
                                "zfs_snapshot"])"#;

        assert_eq!(
            HostConfig {
                path: path.clone(),
                name: "serv".to_owned(),
                vars: Some(VarMap::from([
                    ("var_a".to_owned(), "value_a".to_owned()),
                    ("var_b".to_owned(), "123456789".to_owned())
                ])),
                modules: vec!["physical".to_owned(), "zfs_snapshot".to_owned()],
            },
            define_host_config(&path, &mut janet_runner::janet_client(), user_input).unwrap()
        );
    }

    #[test]
    fn test_define_host_config_no_vars() {
        let path = Utf8PathBuf::from("dummy_path");
        let user_input = r#"(host "serv"
                            :modules [
                                "physical"
                                "zfs_snapshot"])"#;

        assert_eq!(
            HostConfig {
                path: path.clone(),
                name: "serv".to_owned(),
                vars: None,
                modules: vec!["physical".to_owned(), "zfs_snapshot".to_owned()],
            },
            define_host_config(&path, &mut janet_runner::janet_client(), user_input).unwrap()
        );
    }

    #[test]
    fn test_define_host_config_no_modules() {
        let path = Utf8PathBuf::from("dummy_path");
        let user_input = r#"(host "serv"
                            :vars {
                                :var_a "value_a"
                                :var_b 123456789
                            })"#;

        assert!(define_host_config(&path, &mut janet_runner::janet_client(), user_input).is_err());
    }

    #[test]
    fn test_define_host_config_no_host_name() {
        let path = Utf8PathBuf::from("dummy_path");
        let user_input = "()";
        assert!(define_host_config(&path, &mut janet_runner::janet_client(), user_input).is_err());
    }
}
*/
