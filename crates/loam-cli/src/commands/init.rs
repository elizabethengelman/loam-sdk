use soroban_cli::commands::contract::init as soroban_init;
use std::io;

use clap::Parser;

// TO-DO: add exammple contracts
const FRONTEND_TEMPLATE: &str = "https://github.com/loambuild/frontend";

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    pub project_path: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Io error: {0}")]
    IoError(#[from] io::Error),
    #[error("Soroban init error: {0}")]
    SorobanInitError(#[from] soroban_init::Error),
}

impl Cmd {
    #[allow(clippy::unused_self)]
    pub fn run(&self) -> Result<(), Error> {
        let with_examples = vec![];

        soroban_init::Cmd {
            project_path: self.project_path.clone(),
            with_example: with_examples,
            frontend_template: FRONTEND_TEMPLATE.to_string(),
        }.run()?;

        Ok(())
    }
}