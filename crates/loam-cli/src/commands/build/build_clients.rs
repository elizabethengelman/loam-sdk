#![allow(clippy::struct_excessive_bools)]
use clap::Parser;
use futures::future::join_all;
use soroban_cli::commands as cli;
use std::collections::BTreeMap as Map;
use std::ops::Deref;
use std::{env, fmt::Debug, io};

#[derive(Parser, Debug, Clone)]
pub struct Cmd {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("⛔ ️getting the current directory: {0}")]
    GettingCurrentDir(io::Error),
    #[error("⛔ ️parsing environments.toml: {0}")]
    ParsingToml(io::Error),
    #[error("⛔ ️invalid network: must either specify a network name or both network_passphrase and rpc_url")]
    MalformedNetwork,
}

#[derive(Debug, serde::Deserialize)]
struct Environments(Map<Box<str>, Environment>);

impl Deref for Environments {
    type Target = Map<Box<str>, Environment>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, serde::Deserialize)]
struct Environment {
    accounts: Option<Vec<Account>>,
    network: Network,
    contracts: Option<Map<Box<str>, Contract>>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Network {
    name: Option<String>,
    rpc_url: Option<String>,
    network_passphrase: Option<String>,
    // run_locally: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
struct Account {
    name: String,
    // default: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
struct Contract {
    workspace: Option<bool>,
}

// TODO: get from environment
const CURRENT_ENV: &str = "development";

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let working_dir = env::current_dir().map_err(Error::GettingCurrentDir)?;
        let env_toml = working_dir.join("environments.toml");

        if !env_toml.exists() {
            return Ok(());
        }

        let toml_str = std::fs::read_to_string(env_toml).map_err(Error::ParsingToml)?;
        let parsed_toml: Environments = toml::from_str(&toml_str).unwrap();
        let current_env = parsed_toml.get(CURRENT_ENV).unwrap();

        let rpc_url = &current_env.network.rpc_url;
        let network_passphrase = &current_env.network.network_passphrase;
        let network = &current_env.network.name;

        if let Some(name) = network {
            println!("🌐 using {name} network");
        } else if let Some(rpc_url) = rpc_url {
            println!("🌐 using network at {rpc_url}");
        }

        if let Some(accounts) = &current_env.accounts {
            join_all(accounts.into_iter().map(|account| async {
                println!("🔐 creating keys for {:?}", account.name);
                cli::keys::generate::Cmd {
                    name: account.name.clone(),
                    no_fund: false,
                    seed: None,
                    as_secret: false,
                    config_locator: cli::config::locator::Args {
                        global: false,
                        config_dir: None,
                    },
                    hd_path: None,
                    default_seed: false,
                    network: cli::network::Args {
                        rpc_url: rpc_url.clone(),
                        network_passphrase: network_passphrase.clone(),
                        network: network.clone(),
                    },
                }
                .run()
                .await
            }))
            .await;
        }
        if let Some(contracts) = &current_env.contracts {
            join_all(contracts.into_iter().map(|(name, settings)| async {
                if settings.workspace.unwrap_or(false) {
                    println!("📦 building {:?} contract", name.clone());
                };
            }))
            .await;
        }
        Ok(())
    }
}
