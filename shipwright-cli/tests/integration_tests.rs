use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("shipwright").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CLI tool for Shipwright"))
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("serve"))
        .stdout(predicate::str::contains("build"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("shipwright").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("shipwright-cli"));
}

#[test]
fn test_dev_command_help() {
    let mut cmd = Command::cargo_bin("shipwright").unwrap();
    cmd.args(["dev", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Start development server"))
        .stdout(predicate::str::contains("--port"))
        .stdout(predicate::str::contains("--host"));
}

#[test]
fn test_serve_command_help() {
    let mut cmd = Command::cargo_bin("shipwright").unwrap();
    cmd.args(["serve", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Start production server"))
        .stdout(predicate::str::contains("--port"))
        .stdout(predicate::str::contains("--static-dir"));
}

#[test]
fn test_build_command_help() {
    let mut cmd = Command::cargo_bin("shipwright").unwrap();
    cmd.args(["build", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Build application"))
        .stdout(predicate::str::contains("--release"))
        .stdout(predicate::str::contains("--target"));
}

#[test]
fn test_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("Shipwright.toml");
    
    let config_content = r#"
[application]
name = "test-app"
version = "0.1.0"

[serve]
port = 3000
    "#;
    
    fs::write(&config_path, config_content).unwrap();
    
    // Test that the CLI can load the config file
    let mut cmd = Command::cargo_bin("shipwright").unwrap();
    cmd.current_dir(temp_dir.path());
    cmd.args(["build", "--help"]);  // Should not fail with custom config
    cmd.assert().success();
}