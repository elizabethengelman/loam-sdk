use crate::util::TestEnv;

#[test]
fn create_two_accounts() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(r#"
[development]
network = { rpc-url = "http://localhost:8000", network-passphrase = "Standalone Network ; February 2017"}

accounts = [
    { name = "alice" },
    { name = "bob" },
]"#);

        env.loam("build")
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "🔐 creating keys for \"alice\"\n🔐 creating keys for \"bob\"\n",
            ));

        assert!(env.cwd.join(".soroban/identity/alice.toml").exists());
        assert!(env.cwd.join(".soroban/identity/bob.toml").exists());

        // check that they're actually funded
        env.soroban("keys")
            .args(&[
                "fund",
                "alice",
                "--network-passphrase",
                "\"Standalone Network ; February 2017\"",
                "--rpc-url",
                "http://localhost:8000/soroban/rpc",
            ])
            .assert()
            .success()
            .stderr(predicates::str::contains("Account already exists"));
    });
}
