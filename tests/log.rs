use assert_cmd::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_log_shows_commits() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("rit").unwrap()
        .arg("init")
        .arg(tmp.path())
        .assert()
        .success();

    // Log on empty repo should error
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("log")
        .assert()
        .failure();

    // Create two commits
    fs::write(tmp.path().join("a.txt"), "a\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("add")
        .arg("a.txt")
        .assert()
        .success();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .arg("commit")
        .arg("-m")
        .arg("first commit")
        .assert()
        .success();

    fs::write(tmp.path().join("b.txt"), "b\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("add")
        .arg("b.txt")
        .assert()
        .success();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .arg("commit")
        .arg("-m")
        .arg("second commit")
        .assert()
        .success();

    // Log should show both commits, second first
    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("log")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("commit "));
    assert!(stdout.contains("Author: Test <test@example.com>"));
    assert!(stdout.contains("second commit"));
    assert!(stdout.contains("first commit"));

    // Verify order: "second" appears before "first"
    let pos_second = stdout.find("second commit").unwrap();
    let pos_first = stdout.find("first commit").unwrap();
    assert!(pos_second < pos_first, "commits should be in reverse chronological order");
}
