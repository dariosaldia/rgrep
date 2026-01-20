use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs::{File, set_permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn cli_returns_0_and_prints_match_when_found() {
    // Create root folder
    let root = tempdir().unwrap();
    let root_path = root.path();

    // Create a file containing 'Hello' pattern in it
    let mut file_with_pattern = File::create(root_path.join("with_pattern.txt")).unwrap();
    file_with_pattern
        .write_all(b"This is my\nHello World!\n")
        .unwrap();
    file_with_pattern.flush().unwrap();
    drop(file_with_pattern);

    let mut cmd = Command::cargo_bin("rgrep").unwrap();
    let assert = cmd.arg("Hello").arg(root_path).assert();

    assert
        .success()
        .code(predicate::eq(0))
        .stdout(predicate::function(|x: &str| {
            x.contains("with_pattern.txt:2:Hello World!")
        }));
}

#[test]
fn cli_prints_matches_even_if_error_occurs_and_exits_2() {
    // Create root folder
    let root = tempdir().unwrap();
    let root_path = root.path();

    // Create a file containing 'Hello' pattern in it
    let mut file_with_pattern = File::create(root_path.join("with_pattern.txt")).unwrap();
    file_with_pattern
        .write_all(b"This is my\nHello World!\n")
        .unwrap();
    file_with_pattern.flush().unwrap();
    drop(file_with_pattern);

    // Create another file with no read permissions
    let file_no_permissions_path = root_path.join("no_read_permissions.txt");
    let mut file_no_permissions = File::create(&file_no_permissions_path).unwrap();
    file_no_permissions
        .write_all(b"Just some\nrandom string\n")
        .unwrap();
    file_no_permissions.flush().unwrap();
    drop(file_no_permissions);

    // Remove read permissions from the file to force the error when opening it
    let mut permissions = file_no_permissions_path.metadata().unwrap().permissions();
    permissions.set_mode(permissions.mode() & !0o444);
    set_permissions(file_no_permissions_path, permissions).unwrap();

    let mut cmd = Command::cargo_bin("rgrep").unwrap();
    let assert = cmd.arg("Hello").arg(root_path).assert();

    assert
        .failure()
        .code(predicate::eq(2))
        .stdout(predicate::function(|x: &str| {
            x.contains("with_pattern.txt:2:Hello World!")
        }))
        .stderr(predicate::function(|x: &str| !x.is_empty()));
}
