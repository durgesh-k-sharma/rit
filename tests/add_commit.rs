use assert_cmd::Command;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_add_commit_workflow() {
    let tmp = TempDir::new().unwrap();

    // Init
    Command::cargo_bin("rit").unwrap()
        .arg("init")
        .arg(tmp.path())
        .assert()
        .success();

    // Write files
    fs::write(tmp.path().join("hello.txt"), "hello world\n").unwrap();

    // Add
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("add")
        .arg("hello.txt")
        .assert()
        .success();

    // Commit
    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .arg("commit")
        .arg("-m")
        .arg("initial commit")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("root-commit"));
    assert!(stdout.contains("initial commit"));

    // Second commit
    fs::write(tmp.path().join("hello.txt"), "hello world updated\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .arg("add")
        .arg("hello.txt")
        .assert()
        .success();

    let output2 = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .arg("commit")
        .arg("-m")
        .arg("second commit")
        .output()
        .unwrap();
    assert!(output2.status.success());
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(!stdout2.contains("root-commit"));
    assert!(stdout2.contains("second commit"));

    // Verify objects are readable by real git-compatible format
    let objects_dir = tmp.path().join(".git").join("objects");
    assert!(objects_dir.exists());
    // At minimum, check that object files were created
    let has_objects = std::fs::read_dir(&objects_dir).unwrap()
        .filter_map(|e| e.ok())
        .any(|e| e.file_name().to_string_lossy().len() == 2);
    assert!(has_objects, "object directory should contain hash dirs");
}

#[test]
fn test_commit_empty_index_errors() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("rit").unwrap()
        .arg("init")
        .arg(tmp.path())
        .assert()
        .success();

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .arg("commit")
        .arg("-m")
        .arg("should fail")
        .assert()
        .failure();
}
