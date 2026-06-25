use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn test_init_creates_directory_structure() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("my-repo");

    let output = Command::cargo_bin("rit").unwrap()
        .arg("init")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Initialized empty rit repository"));
    assert!(stdout.contains(".git"));

    let git_dir = dir.join(".git");
    assert!(git_dir.join("HEAD").exists());
    assert!(git_dir.join("config").exists());
    assert!(git_dir.join("description").exists());
    assert!(git_dir.join("objects").join("info").is_dir());
    assert!(git_dir.join("objects").join("pack").is_dir());
    assert!(git_dir.join("refs").join("heads").is_dir());
    assert!(git_dir.join("refs").join("tags").is_dir());

    let head = std::fs::read_to_string(git_dir.join("HEAD")).unwrap();
    assert_eq!(head, "ref: refs/heads/main\n");
}

#[test]
fn test_init_twice_errors() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("rit").unwrap()
        .arg("init")
        .arg(tmp.path())
        .assert()
        .success();

    Command::cargo_bin("rit").unwrap()
        .arg("init")
        .arg(tmp.path())
        .assert()
        .failure();
}
