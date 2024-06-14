use crate::util::{AssertExt, TestEnv};

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
production.accounts = [
    {{ name = "alice" }},
]

[production.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[production.contracts]
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

        let stdout = env.loam("build").assert().success().stdout_as_str();
        assert!(stdout.contains("creating keys for \"alice\"\n"));
        assert!(stdout.contains("using network at http://localhost:8000/rpc\n"));

        for c in contracts {
            assert!(stdout.contains(&format!("installing \"{c}\" wasm bytecode on-chain")));
            assert!(stdout.contains(&format!("instantiating \"{c}\" smart contract")));
            assert!(stdout.contains(&format!("binding \"{c}\" contract")));
            assert!(stdout.contains(&format!("importing \"{c}\" contract")));
        }

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
production.accounts = [
    { name = "alice" },
]

[production.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"

[production.contracts]
hello.workspace = true
"#,
        );

        env.loam("build")
            .assert()
            .failure()
            .stderr(predicates::str::contains("No contract named \"hello\""));
    });
}
