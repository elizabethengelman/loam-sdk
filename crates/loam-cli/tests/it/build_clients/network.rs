use crate::util::{AssertExt, TestEnv};

#[test]
fn run_network_from_rpc_and_passphrase() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(
            r#"
production.accounts = [
    { name = "alice" },
]

[production.network]
rpc-url = "http://localhost:8000/rpc"
network-passphrase = "Standalone Network ; February 2017"
"#,
        );

        let stderr = env.loam("build").assert().success().stderr_as_str();
        assert!(stderr.contains("🌐 using network at http://localhost:8000/rpc\n"));
    });
}

#[test]
fn run_named_network() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        // create a network named "lol"
        env.soroban("network")
            .args(&[
                "add",
                "lol",
                "--rpc-url",
                "http://localhost:8000/soroban/rpc",
                "--network-passphrase",
                "Standalone Network ; February 2017",
            ])
            .assert()
            .success();

        env.set_environments_toml(
            r#"
production.accounts = [
    { name = "alice" },
]

production.network.name = "lol"
"#,
        );

        let stderr = env.loam("build").assert().success().stderr_as_str();
        assert!(stderr.contains("🌐 using lol network\n"));
    });
}
