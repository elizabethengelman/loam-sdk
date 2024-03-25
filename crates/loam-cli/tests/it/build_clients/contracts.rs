use crate::util::TestEnv;

#[test]
fn contracts_built() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.set_environments_toml(
            r#"
[development.network]
rpc-url = "http://localhost:8000"
network-passphrase = "Standalone Network ; February 2017"

[development.contracts]
hello_world = { workspace = true }
increment = { workspace = true }

"#,
        );

        env.loam("build").assert().success();
        //.stdout(predicates::str::contains(
        //    "📦 building \"hello_world\" contract\n📦 building \"increment\" contract",
        //));
    });
}
