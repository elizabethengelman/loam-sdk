use crate::util::TestEnv;
use predicates::prelude::*;

#[test]
fn contracts_built() {
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
hello_world.workspace = true
soroban_auth_contract.workspace = true
soroban_custom_types_contract.workspace = true
soroban_increment_contract.workspace = true
"#,
        );

        env.loam("build").assert().success().stdout(
            predicates::str::contains("creating keys for \"alice\"\n")
                .and(predicates::str::contains(
                    "using network at http://localhost:8000/rpc\n",
                ))
                .and(predicates::str::contains(
                    "installing \"soroban_auth_contract\" wasm bytecode on-chain...",
                ))
                .and(predicates::str::contains(
                    "installing \"soroban_custom_types_contract\" wasm bytecode on-chain",
                ))
                .and(predicates::str::contains(
                    "installing \"hello_world\" wasm bytecode on-chain",
                ))
                .and(predicates::str::contains(
                    "installing \"soroban_increment_contract\" wasm bytecode on-chain",
                ))
                .and(predicates::str::contains(
                    "instantiating \"soroban_auth_contract\" smart contract",
                ))
                .and(predicates::str::contains(
                    "instantiating \"soroban_custom_types_contract\" smart contract",
                ))
                .and(predicates::str::contains(
                    "instantiating \"hello_world\" smart contract",
                ))
                .and(predicates::str::contains(
                    "instantiating \"soroban_increment_contract\" smart contract",
                ))
                .and(predicates::str::contains(
                    "binding \"soroban_auth_contract\" contract",
                ))
                .and(predicates::str::contains(
                    "binding \"soroban_custom_types_contract\" contract",
                ))
                .and(predicates::str::contains(
                    "binding \"hello_world\" contract",
                ))
                .and(predicates::str::contains(
                    "binding \"soroban_increment_contract\" contract",
                ))
                .and(predicates::str::contains(
                    "importing \"soroban_auth_contract\" contract",
                ))
                .and(predicates::str::contains(
                    "importing \"soroban_custom_types_contract\" contract",
                ))
                .and(predicates::str::contains(
                    "importing \"hello_world\" contract",
                ))
                .and(predicates::str::contains(
                    "importing \"soroban_increment_contract\" contract",
                )),
        );

        // check that contracts are actually deployed, bound, and imported
        assert!(env.cwd.join("packages/soroban_auth_contract").exists());
        assert!(env
            .cwd
            .join("packages/soroban_custom_types_contract")
            .exists());
        assert!(env.cwd.join("packages/hello_world").exists());
        assert!(env.cwd.join("packages/soroban_increment_contract").exists());

        assert!(env
            .cwd
            .join("src/contracts/soroban_auth_contract.ts")
            .exists());
        assert!(env
            .cwd
            .join("src/contracts/soroban_custom_types_contract.ts")
            .exists());
        assert!(env.cwd.join("src/contracts/hello_world.ts").exists());
        assert!(env
            .cwd
            .join("src/contracts/soroban_increment_contract.ts")
            .exists());
    });
}

// #[test]
// fn contract_with_bad_name_prints_useful_error() {
//     TestEnv::from("soroban-init-boilerplate", |env| {
//         env.set_environments_toml(
//             r#"
// development.accounts = [
//     { name = "alice" },
// ]
//
// [development.network]
// rpc-url = "http://localhost:8000"
// network-passphrase = "Standalone Network ; February 2017"
//
// development.contracts.hello.workspace = true
// "#,
//         );
//
//         env.loam("build")
//             .assert()
//             .failure()
//             .stderr(predicates::str::contains("No contract named \"hello\"!"));
//     });
// }
