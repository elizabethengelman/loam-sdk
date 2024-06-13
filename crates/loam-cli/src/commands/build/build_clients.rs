#![allow(clippy::struct_excessive_bools)]
use crate::commands::build::env_toml;
use clap::Parser;
use soroban_cli::commands::NetworkRunnable;
use soroban_cli::{commands as cli, CommandParser};
use std::collections::BTreeMap as Map;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, clap::ValueEnum)]
pub enum LoamEnv {
    Development,
    Testing,
    Staging,
    Production,
}

#[derive(Parser, Debug, Clone)]
pub struct Cmd {
    #[arg(long, default_value = ".")]
    pub workspace_root: std::path::PathBuf,
    #[arg(env = "LOAM_ENV", value_enum, default_value = "production")]
    pub env: LoamEnv,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    EnvironmentsToml(#[from] env_toml::Error),
    #[error("⛔ ️invalid network: must either specify a network name or both network_passphrase and rpc_url")]
    MalformedNetwork,
    #[error(transparent)]
    ParsingNetwork(#[from] cli::network::Error),
    #[error(transparent)]
    GeneratingKey(#[from] cli::keys::generate::Error),
    #[error("⛔ ️can only have one default account; marked as default: {0:?}")]
    OnlyOneDefaultAccount(Vec<String>),
    #[error("⛔ ️you need to provide at least one account, to use as the source account for contract deployment and other operations")]
    NeedAtLeastOneAccount,
    #[error("⛔ ️No contract named {0:?}")]
    BadContractName(String),
    #[error(transparent)]
    ContractInstall(#[from] cli::contract::install::Error),
    #[error(transparent)]
    ContractDeploy(#[from] cli::contract::deploy::wasm::Error),
    #[error(transparent)]
    ContractBindings(#[from] cli::contract::bindings::typescript::Error),
}

impl Cmd {
    pub async fn run(&self) -> Result<(), Error> {
        let current_env = env_toml::Environment::get(&self.workspace_root, self.loam_env())?;

        if current_env.is_none() {
            return Ok(());
        }
        let current_env = current_env.unwrap();

        self.add_network_to_env(&current_env.network)?;
        self.handle_accounts(&current_env.accounts).await?;
        self.handle_contracts(&current_env.contracts).await?;

        Ok(())
    }

    fn loam_env(&self) -> String {
        format!("{:?}", self.env).to_lowercase()
    }

    /// Parse the network settings from the environments.toml file and set STELLAR_RPC_URL and
    /// STELLAR_NETWORK_PASSPHRASE.
    ///
    /// We could set STELLAR_NETWORK instead, but when importing contracts, we want to hard-code
    /// the network passphrase. So if given a network name, we use soroban-cli to fetch the RPC url
    /// & passphrase for that named network, and still set the environment variables.
    fn add_network_to_env(&self, network: &env_toml::Network) -> Result<(), Error> {
        let rpc_url = &network.rpc_url;
        let network_passphrase = &network.network_passphrase;
        let network_name = &network.name;

        if let Some(name) = network_name {
            let cli::network::Network {
                rpc_url,
                network_passphrase,
            } = (cli::network::Args {
                network: Some(name.clone()),
                rpc_url: None,
                network_passphrase: None,
            })
            .get(&cli::config::locator::Args {
                global: false,
                config_dir: None,
            })?;
            println!("🌐 using {name} network");
            std::env::set_var("STELLAR_RPC_URL", rpc_url);
            std::env::set_var("STELLAR_NETWORK_PASSPHRASE", network_passphrase);
        } else if let Some(rpc_url) = rpc_url {
            if let Some(passphrase) = network_passphrase {
                std::env::set_var("STELLAR_RPC_URL", rpc_url);
                std::env::set_var("STELLAR_NETWORK_PASSPHRASE", passphrase);
                println!("🌐 using network at {rpc_url}");
            } else {
                return Err(Error::MalformedNetwork);
            }
        }

        Ok(())
    }

    async fn handle_accounts(
        &self,
        accounts: &Option<Vec<env_toml::Account>>,
    ) -> Result<(), Error> {
        if accounts.is_none() {
            return Err(Error::NeedAtLeastOneAccount);
        }

        let accounts = accounts.as_ref().unwrap();

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
            cli::keys::generate::Cmd::parse_arg_vec(&[&account.name])
                .unwrap()
                .run()
                .await?
        }

        std::env::set_var("STELLAR_ACCOUNT", &default_account);

        Ok(())
    }

    async fn handle_contracts(
        &self,
        contracts: &Option<Map<Box<str>, env_toml::Contract>>,
    ) -> Result<(), Error> {
        if contracts.is_none() {
            return Ok(());
        }
        let contracts = contracts.as_ref().unwrap();
        for (name, settings) in contracts {
            if settings.workspace.unwrap_or(false) {
                let wasm_path = &self.workspace_root.join(format!("target/loam/{name}.wasm"));
                if !wasm_path.exists() {
                    return Err(Error::BadContractName(name.to_string()));
                }
                println!("📲 installing {:?} wasm bytecode on-chain...", name.clone());
                let hash = cli::contract::install::Cmd::parse_arg_vec(&[
                    "--wasm",
                    wasm_path.to_str().unwrap(),
                ])
                .unwrap()
                .run_against_rpc_server(None, None)
                .await?
                .into_result()
                .unwrap()
                .to_string();
                println!("    ↳ hash: {hash}");

                println!("🪞 instantiating {:?} smart contract", name.clone());
                //  TODO: check if hash is already the installed version, skip the rest if so
                let contract_id =
                    cli::contract::deploy::wasm::Cmd::parse_arg_vec(&["--wasm-hash", &hash])
                        .unwrap()
                        .run_against_rpc_server(None, None)
                        .await?
                        .into_result()
                        .unwrap();
                // TODO: save the contract id for use in subsequent runs
                println!("    ↳ contract_id: {contract_id}");

                println!("🎭 binding {:?} contract", name.clone());
                cli::contract::bindings::typescript::Cmd::parse_arg_vec(&[
                    "--contract-id",
                    &contract_id,
                    "--output-dir",
                    self.workspace_root
                        .join(format!("packages/{}", name.clone()))
                        .to_str()
                        .unwrap(),
                    "--overwrite",
                ])
                .unwrap()
                .run()
                .await?;

                println!("🍽️ importing {:?} contract", name.clone());
                let allow_http = if self.loam_env() == "development" {
                    "\n  allowHttp: true,"
                } else {
                    ""
                };
                let network = std::env::var("STELLAR_NETWORK_PASSPHRASE").unwrap();
                let template = format!(
                    r#"import * as Client from '{name}';
import {{ rpcUrl }} from './util';

export default new Client.Client({{
  networkPassphrase: '{network}',
  contractId: '{contract_id}',
  rpcUrl,{allow_http}
  publicKey: undefined,
}});
"#
                );
                let path = self.workspace_root.join(format!("src/contracts/{name}.ts"));
                std::fs::write(path, template).unwrap();
            };
        }

        Ok(())
    }
}
