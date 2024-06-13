use std::collections::BTreeMap as Map;
use std::io;
use std::ops::Deref;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("⛔ ️parsing environments.toml: {0}")]
    ParsingToml(io::Error),
    #[error("⛔ ️no settings for current LOAM_ENV ({0:?}) found in environments.toml")]
    NoSettingsForCurrentEnv(String),
}

#[derive(Debug, serde::Deserialize)]
struct Environments(Map<Box<str>, Environment>);

impl Deref for Environments {
    type Target = Map<Box<str>, Environment>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Environment {
    pub accounts: Option<Vec<Account>>,
    pub network: Network,
    pub contracts: Option<Map<Box<str>, Contract>>,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Network {
    pub name: Option<String>,
    pub rpc_url: Option<String>,
    pub network_passphrase: Option<String>,
    // run_locally: Option<bool>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Account {
    pub name: String,
    pub default: Option<bool>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Contract {
    pub workspace: Option<bool>,
}

impl Environment {
    pub fn get(workspace_root: &PathBuf, loam_env: String) -> Result<Option<Environment>, Error> {
        let env_toml = workspace_root.join("environments.toml");

        if !env_toml.exists() {
            return Ok(None);
        }

        let toml_str = std::fs::read_to_string(env_toml).map_err(Error::ParsingToml)?;
        let parsed_toml: Environments = toml::from_str(&toml_str).unwrap();
        let current_env = parsed_toml.get(loam_env.as_str());
        if current_env.is_none() {
            return Err(Error::NoSettingsForCurrentEnv(loam_env));
        };
        Ok(current_env.cloned())
    }
}
