use crate::util::TestEnv;

#[test]
fn no_environments_toml_ends_after_contract_build() {
    TestEnv::from("soroban-init-boilerplate", |env| {
        env.loam("build")
            .assert()
            .success()
            .stderr(predicates::str::contains(
                "Finished release [optimized] target(s) in",
            ));
    });
}
