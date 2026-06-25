use assert_cmd::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_cat_file_type_and_pretty_print() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("rit").unwrap()
        .arg("init")
        .arg(tmp.path())
        .assert()
        .success();

    fs::write(tmp.path().join("test.txt"), "hello\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("add")
        .arg("test.txt")
        .assert()
        .success();

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .arg("commit")
        .arg("-m")
        .arg("test commit")
        .assert()
        .success();

    // Get the commit SHA from log
    let log_output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("log")
        .output()
        .unwrap();
    let log_stdout = String::from_utf8_lossy(&log_output.stdout);
    let commit_sha = log_stdout.lines()
        .find(|l| l.starts_with("commit "))
        .map(|l| l.trim_start_matches("commit ").trim())
        .unwrap()
        .to_string();

    // -t on commit
    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .args(["cat-file", "-t", &commit_sha])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "commit");

    // -p on commit (should contain tree line)
    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .args(["cat-file", "-p", &commit_sha])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tree "));
    assert!(stdout.contains("author "));
    assert!(stdout.contains("test commit"));
}
