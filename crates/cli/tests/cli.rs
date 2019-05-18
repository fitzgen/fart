use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

fn fart_bin() -> Command {
    Command::cargo_bin("fart").unwrap()
}

#[test]
fn help() {
    fart_bin().arg("help").assert().success();
}

#[test]
fn new() {
    let dir = TempDir::new().unwrap();
    fart_bin()
        .arg("new")
        .arg("my-project")
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicates::str::contains("Created new fart project: "))
        .stderr(predicates::str::contains("my-project"));
    assert!(dir.path().join("my-project").is_dir());
    assert!(dir
        .path()
        .join("my-project")
        .join("src")
        .join("main.rs")
        .is_file());
}
