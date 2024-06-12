use crate::util::TestEnv;
use predicates::prelude::*;

#[test]
fn contracts_built() {
    let contracts = [
        "soroban_auth_contract",
        "soroban_custom_types_contract",
        "hello_world",
        "soroban_increment_contract",
    ];
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(
            format!(
                r#"
development.accounts = [
    {{ name = "alice" }},
]

[development.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[development.contracts]
{}
"#,
                contracts
                    .iter()
                    .map(|c| format!("{c}.workspace = true"))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
            .as_str(),
        );

        let mut contract_stdout_predicates = contracts
            .iter()
            .map(|c| {
                predicates::str::contains(format!("installing \"{c}\" wasm bytecode on-chain"))
                    .and(predicates::str::contains(format!(
                        "instantiating \"{c}\" smart contract"
                    )))
                    .and(predicates::str::contains(format!(
                        "binding \"{c}\" contract"
                    )))
                    .and(predicates::str::contains(format!(
                        "importing \"{c}\" contract"
                    )))
            })
            .collect::<Vec<_>>();

        env.loam("build").assert().success().stdout(
            predicates::str::contains("creating keys for \"alice\"\n")
                .and(predicates::str::contains(
                    "using network at http://localhost:8000/rpc\n",
                ))
                .and(contract_stdout_predicates.pop().unwrap())
                .and(contract_stdout_predicates.pop().unwrap())
                .and(contract_stdout_predicates.pop().unwrap())
                .and(contract_stdout_predicates.pop().unwrap()),
        );

        // check that contracts are actually deployed, bound, and imported
        for contract in contracts {
            assert!(env.cwd.join(format!("packages/{}", contract)).exists());
            assert!(env
                .cwd
                .join(format!("src/contracts/{}.ts", contract))
                .exists());
        }
    });
}

#[test]
fn contract_with_bad_name_prints_useful_error() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(
            r#"
development.accounts = [
    { name = "alice" },
]

[development.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[development.contracts]
hello.workspace = true
"#,
        );

        env.loam("build")
            .assert()
            .failure()
            .stderr(predicates::str::contains("No contract named \"hello\""));
    });
}
