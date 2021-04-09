use env_logger;
use log::info;

use std::env;
use std::fmt;
use std::process;
use std::str;

const TEST_VAR: &str = "YOU_ARE_STILL_TESTING_NOW";
const DUMMY_RUNTIME_CONFIG: &str = "RUST_LOG=error";
const DUMMY_RUNTIME_CONFIG_PATH: &str = "./runtime_config";

struct DummyConfig;

impl DummyConfig {
    fn new() -> Self {
        std::fs::write(DUMMY_RUNTIME_CONFIG_PATH, DUMMY_RUNTIME_CONFIG)
            .expect("To be able to write config");

        DummyConfig
    }
}

impl Drop for DummyConfig {
    fn drop(&mut self) {
        std::fs::remove_file(DUMMY_RUNTIME_CONFIG_PATH).expect("To be able to remove config");
    }
}

impl fmt::Display for DummyConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        info!("test");
        f.write_str("bar")
    }
}

fn log_and_change_dynamically(dynamic_filter_check: &env_logger::DynamicLogLevel) {
    //Create the dummy file that contains the new log level to be changed at runtime by
    // 'check_filter_config'. NOTE: when 'd' is dropped the file is removed.
    let d = DummyConfig::new();
    info!("{}", d);
    dynamic_filter_check.check_filter_config();
    info!("This log should not show up since the new config is at 'error'");
}

fn main() {
    let mut builder = env_logger::Builder::from_default_env();
    let dynamic_filter_check = builder.try_init_dynamic_level().unwrap();

    if env::var(TEST_VAR).is_ok() {
        log_and_change_dynamically(&dynamic_filter_check);
        return;
    }

    let exe = env::current_exe().unwrap();
    let out = process::Command::new(exe)
        .env(TEST_VAR, "1")
        .env(env_logger::DEFAULT_FILTER_ENV, "debug")
        .env(
            env_logger::FILTER_RUNTIME_CONFIG_PATH_ENV,
            DUMMY_RUNTIME_CONFIG_PATH,
        )
        .output()
        .unwrap_or_else(|e| panic!("Unable to start child process: {}", e));

    if out.status.success() {
        let logs = str::from_utf8(&out.stderr).unwrap().trim();
        let log_lines: Vec<&str> = logs.split("\n").collect();
        assert_eq!(log_lines.len(), 2, "Received {:?}", log_lines);
        assert!(log_lines[0].ends_with("test"));
        assert!(log_lines[1].ends_with("bar"));
        return;
    }

    println!("test failed: {}", out.status);
    println!("--- stdout\n{}", str::from_utf8(&out.stdout).unwrap());
    println!("--- stderr\n{}", str::from_utf8(&out.stderr).unwrap());
    process::exit(1);
}
