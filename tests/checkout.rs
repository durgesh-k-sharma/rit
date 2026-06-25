use assert_cmd::Command;
use tempfile::TempDir;
use std::fs;

fn rit_init(tmp: &TempDir) {
    Command::cargo_bin("rit").unwrap()
        .arg("init")
        .arg(tmp.path())
        .assert()
        .success();
}

fn rit_commit(tmp: &TempDir, msg: &str) {
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path())
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .arg("commit")
        .arg("-m")
        .arg(msg)
        .assert()
        .success();
}

#[test]
fn test_create_and_switch_branch() {
    let tmp = TempDir::new().unwrap();
    rit_init(&tmp);
    fs::write(tmp.path().join("file.txt"), "v1\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("file.txt").assert().success();
    rit_commit(&tmp, "v1");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("branch").arg("feature").assert().success();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("feature").assert().success();

    let head = fs::read_to_string(tmp.path().join(".git").join("HEAD")).unwrap();
    assert_eq!(head, "ref: refs/heads/feature\n");
}

#[test]
fn test_checkout_updates_working_tree() {
    let tmp = TempDir::new().unwrap();
    rit_init(&tmp);
    fs::write(tmp.path().join("file.txt"), "v1\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("file.txt").assert().success();
    rit_commit(&tmp, "v1 on main");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("-b").arg("side").assert().success();

    fs::write(tmp.path().join("file.txt"), "v2\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("file.txt").assert().success();
    rit_commit(&tmp, "v2 on side");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("main").assert().success();
    assert_eq!(fs::read_to_string(tmp.path().join("file.txt")).unwrap(), "v1\n");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("side").assert().success();
    assert_eq!(fs::read_to_string(tmp.path().join("file.txt")).unwrap(), "v2\n");
}

#[test]
fn test_checkout_creates_new_files() {
    let tmp = TempDir::new().unwrap();
    rit_init(&tmp);
    fs::write(tmp.path().join("shared.txt"), "shared\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("shared.txt").assert().success();
    rit_commit(&tmp, "init");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("-b").arg("feature").assert().success();

    fs::write(tmp.path().join("feature_only.txt"), "feature\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("feature_only.txt").assert().success();
    rit_commit(&tmp, "add feature file");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("main").assert().success();
    assert!(tmp.path().join("shared.txt").exists());
    assert!(!tmp.path().join("feature_only.txt").exists());

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("feature").assert().success();
    assert_eq!(fs::read_to_string(tmp.path().join("feature_only.txt")).unwrap(), "feature\n");
}

#[test]
fn test_safety_check_blocks_overwrite() {
    let tmp = TempDir::new().unwrap();
    rit_init(&tmp);
    fs::write(tmp.path().join("file.txt"), "original\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("file.txt").assert().success();
    rit_commit(&tmp, "original on main");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("-b").arg("feature").assert().success();
    fs::write(tmp.path().join("file.txt"), "on feature\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("file.txt").assert().success();
    rit_commit(&tmp, "feature change");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("main").assert().success();
    fs::write(tmp.path().join("file.txt"), "unstaged change\n").unwrap();

    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("feature")
        .output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("would be overwritten"));
    assert_eq!(fs::read_to_string(tmp.path().join("file.txt")).unwrap(), "unstaged change\n");
}

#[test]
fn test_force_overrides_safety() {
    let tmp = TempDir::new().unwrap();
    rit_init(&tmp);
    fs::write(tmp.path().join("file.txt"), "original\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("file.txt").assert().success();
    rit_commit(&tmp, "original on main");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("-b").arg("feature").assert().success();
    fs::write(tmp.path().join("file.txt"), "on feature\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("file.txt").assert().success();
    rit_commit(&tmp, "feature change");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("main").assert().success();
    fs::write(tmp.path().join("file.txt"), "unstaged change\n").unwrap();

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("--force").arg("feature").assert().success();
    assert_eq!(fs::read_to_string(tmp.path().join("file.txt")).unwrap(), "on feature\n");
}

#[test]
fn test_delete_branch() {
    let tmp = TempDir::new().unwrap();
    rit_init(&tmp);
    fs::write(tmp.path().join("f.txt"), "x\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("f.txt").assert().success();
    rit_commit(&tmp, "init");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("branch").arg("temp").assert().success();
    assert!(tmp.path().join(".git").join("refs").join("heads").join("temp").exists());

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("branch").arg("-d").arg("temp").assert().success();
    assert!(!tmp.path().join(".git").join("refs").join("heads").join("temp").exists());
}

#[test]
fn test_cannot_delete_checked_out_branch() {
    let tmp = TempDir::new().unwrap();
    rit_init(&tmp);
    fs::write(tmp.path().join("f.txt"), "x\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("f.txt").assert().success();
    rit_commit(&tmp, "init");

    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("branch").arg("-d").arg("main")
        .output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot delete branch"));
}

#[test]
fn test_log_decorations() {
    let tmp = TempDir::new().unwrap();
    rit_init(&tmp);
    fs::write(tmp.path().join("f.txt"), "x\n").unwrap();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("add").arg("f.txt").assert().success();
    rit_commit(&tmp, "init");

    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("-b").arg("feature").assert().success();
    Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("checkout").arg("main").assert().success();

    let output = Command::cargo_bin("rit").unwrap()
        .current_dir(tmp.path()).arg("log").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("HEAD -> main"));
    assert!(stdout.contains("feature"));
}
