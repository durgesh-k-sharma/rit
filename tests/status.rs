use assert_cmd::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_status_sections() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("rit").unwrap()
        .arg("init")
        .arg(tmp.path())
        .assert()
        .success();

    // Status on empty repo
    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("status")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("On branch main"));

    // Add a file and check status shows staged
    fs::write(tmp.path().join("staged.txt"), "staged\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("add")
        .arg("staged.txt")
        .assert()
        .success();

    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("status")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Changes to be committed"));
    assert!(stdout.contains("new file:"));
    assert!(stdout.contains("staged.txt"));

    // Untracked file
    fs::write(tmp.path().join("untracked.txt"), "untracked\n").unwrap();
    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("status")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Untracked files"));
    assert!(stdout.contains("untracked.txt"));
}
