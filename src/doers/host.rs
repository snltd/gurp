use crate::utils::janet_helpers as j;
use crate::utils::module;
use crate::utils::types::{Opts, VarMap};
use crate::{debug, verbose};
use anyhow::{Context, anyhow};
use camino::Utf8PathBuf;
// use janetrs::JanetTable;
use janetrs::JanetType::Table;
// use janetrs::client::JanetClient;
// use janetrs::env::CFunOptions;
// use janetrs::{Janet, JanetArgs, TaggedJanet};

use janetrs::{Janet, JanetType, client::JanetClient, env::CFunOptions, janet_fn};

// #[janetrs::janet_fn(arity(fix(1)))]
#[janet_fn]
fn machine_config_handler(config_table: &mut [Janet]) -> Janet {
    // let table: JanetTable = JanetTable::unpack(config_table).unwrap();
    let janet_table = config_table[0];
    // let table = janet_table.try_unwrap().unwrap();
    println!("{:?}", janet_table.len());
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
