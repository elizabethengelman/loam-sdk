#![allow(clippy::struct_excessive_bools)]
use clap::Parser;
use soroban_cli::commands::NetworkRunnable;
use soroban_cli::{commands as cli, fee, wasm};
use std::collections::BTreeMap as Map;
use std::io::Write;
use std::ops::Deref;
use std::{fmt::Debug, io};

#[derive(Parser, Debug, Clone)]
pub struct Cmd {
    #[arg(long, default_value = ".")]
    pub workspace_root: std::path::PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("⛔ ️parsing environments.toml: {0}")]
    ParsingToml(io::Error),
    #[error("⛔ ️invalid network: must either specify a network name or both network_passphrase and rpc_url")]
    MalformedNetwork,
    #[error(transparent)]
    GeneratingKey(#[from] cli::keys::generate::Error),
    #[error("⛔ ️can only have one default account; marked as default: {0:?}")]
    OnlyOneDefaultAccount(Vec<String>),
    #[error("⛔ ️you need to provide at least one account, to use as the source account for contract deployment and other operations")]
    NeedAtLeastOneAccount,
    #[error(transparent)]
    ContractInstall(#[from] cli::contract::install::Error),
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
    default: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
struct Contract {
    workspace: Option<bool>,
}

// TODO: get from environment
const CURRENT_ENV: &str = "development";

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let env_toml = self.workspace_root.join("environments.toml");

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

        let default_account = if let Some(accounts) = &current_env.accounts {
            let default_account_candidates = accounts
                .iter()
                .filter_map(|account| {
                    account
                        .default
                        .unwrap_or(false)
                        .then(|| account.name.clone())
                })
                .collect::<Vec<_>>();
            let default_account = match default_account_candidates.len() {
                0 if accounts.is_empty() => return Err(Error::NeedAtLeastOneAccount),
                0 => accounts[0].name.clone(),
                1 => default_account_candidates[0].to_string(),
                _ => return Err(Error::OnlyOneDefaultAccount(default_account_candidates)),
            };
            for account in accounts {
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
                .await?
            }
            default_account
        } else {
            return Err(Error::NeedAtLeastOneAccount);
        };
        if let Some(contracts) = &current_env.contracts {
            for (name, settings) in contracts {
                if settings.workspace.unwrap_or(false) {
                    print!(
                        "📲 installing {:?} wasm bytecode on-chain... ",
                        name.clone()
                    );
                    let hash = cli::contract::install::Cmd {
                        wasm: wasm::Args {
                            wasm: self.workspace_root.join(format!("target/loam/{name}.wasm")),
                        },
                        fee: fee::Args {
                            fee: 100u32,
                            cost: false,
                            instructions: None,
                        },
                        config: cli::config::Args {
                            source_account: default_account.clone(),
                            hd_path: None,
                            locator: cli::config::locator::Args {
                                global: false,
                                config_dir: None,
                            },
                            network: cli::network::Args {
                                rpc_url: rpc_url.clone(),
                                network_passphrase: network_passphrase.clone(),
                                network: network.clone(),
                            },
                        },
                        ignore_checks: false,
                    }
                    .run_against_rpc_server(None, None)
                    .await?;
                    print!("hash: {hash:?}");
                    io::stdout().flush().unwrap();
                    println!("🪞 instantiating {:?} smart contract", name.clone());
                    println!("🎭 binding {:?} contract", name.clone());
                    println!("🍽️ importing {:?} contract", name.clone());
                };
            }
        }
        Ok(())
    }
}
