use assert_cmd::Command;
use assert_fs::TempDir;
use fs_extra::dir::{copy, CopyOptions};
use std::path::PathBuf;

pub struct TestEnv {
    pub temp_dir: TempDir,
    pub cwd: PathBuf,
}

impl TestEnv {
    pub fn new(template: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures");

        copy(
            template_dir.join(template),
            &temp_dir,
            &CopyOptions::new(),
        ).unwrap();

        Self {
            cwd: temp_dir.join(template),
            temp_dir,
        }
    }

    pub fn from<F: FnOnce(&TestEnv)>(template: &str, f: F) {
        let test_env = TestEnv::new(template);
        f(&test_env);
    }

    pub fn loam(&self, cmd: &str) -> Command {
        let mut loam = Command::cargo_bin("loam").unwrap();
        loam.current_dir(&self.cwd);
        loam.arg(cmd);
        loam
    }

    pub fn soroban(&self, cmd: &str) -> Command {
        let mut soroban = Command::new("soroban");
        soroban.current_dir(&self.cwd);
        soroban.arg(cmd);
        soroban
    }

    pub fn set_environments_toml(&self, contents: impl AsRef<[u8]>) {
        std::fs::write(
            self.cwd.join("environments.toml"),
            contents,
        ).unwrap();
    }
}

