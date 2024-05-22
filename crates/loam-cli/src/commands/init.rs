use soroban_cli::commands::contract::init as soroban_init;
// use std::path::Path;
// use std::io;

use clap::Parser;

// TO-DO add exammple contracts
const FRONTEND_TEMPLATE: &str = "https://github.com/loambuild/frontend";
// const WITH_EXAMPLE_LONG_HELP_TEXT: &str =
//     "An optional flag to specify example contracts to include. A hello-world contract will be included by default.";

#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Cmd {
    pub project_path: String,

    // arg is a macro from the clap package
    // #[arg(short, long, num_args = 1.., value_parser=possible_example_values(), long_help=WITH_EXAMPLE_LONG_HELP_TEXT)]
    // pub with_example: Vec<String>,

}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SorobanInit(#[from] soroban_init::Error),

}

impl Cmd {
    #[allow(clippy::unused_self)]
    pub fn run(&self) -> Result<(), Error> {
        let with_examples = vec![];

        // &soroban_init::Cmd {
        //     project_path: self.project_path.clone(),
        //     frontend_template: FRONTEND_TEMPLATE.to_string(),
        //     with_example: with_examples,
        // };

        soroban_init::Cmd {
            project_path: self.project_path.clone(),
            with_example: with_examples,
            frontend_template: FRONTEND_TEMPLATE.to_string(),
        }.run()?;

        Ok(())

    }
}