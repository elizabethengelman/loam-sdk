use crate::util::TestEnv;

#[test]
fn run_network_from_rpc_and_passphrase() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(r#"
[development]
network = { rpc-url = "http://localhost:8000", network-passphrase = "Standalone Network ; February 2017"}
"#);

        env.loam("build")
            .assert()
            .success()
            .stdout("🌐 using network at http://localhost:8000\n");
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
[development]
network = { name = "lol" }
"#,
        );

        env.loam("build")
            .assert()
            .success()
            .stdout("🌐 using lol network\n");
    });
}
