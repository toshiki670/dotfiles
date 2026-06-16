//! `copy-obj` の E2E テスト（assert_cmd + predicates + rstest）。

use assert_cmd::Command;
use predicates::prelude::*;
use rstest::rstest;
use std::fs;
use tempfile::TempDir;

fn copy_obj() -> Command {
    Command::cargo_bin("copy-obj").unwrap()
}

#[rstest]
#[case("--help")]
#[case("-h")]
fn help_flag_succeeds(#[case] flag: &str) {
    copy_obj().arg(flag).assert().success();
}

#[rstest]
#[case("--version")]
#[case("-V")]
fn version_flag_prints_name_and_version(#[case] flag: &str) {
    copy_obj()
        .arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("copy-obj"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_required_file_arg_fails_with_usage() {
    copy_obj()
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("required").or(predicate::str::contains("FILE")));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn exits_with_macos_only_on_non_macos() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("x.txt");
    fs::write(&file, "x\n").unwrap();

    copy_obj()
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("macOS only"));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_invokes_osascript_for_existing_file() {
    let fx = TempDir::new().unwrap();
    let file = fx.path().join("sample.txt");
    let bin = fx.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(&file, "sample\n").unwrap();

    let args_out = fx.path().join("osascript.args");
    let stub = bin.join("osascript");
    fs::write(
        &stub,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >\"{}\"\n",
            args_out.display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let existing = std::env::var_os("PATH").unwrap_or_default();
    let path =
        std::env::join_paths(std::iter::once(bin.clone()).chain(std::env::split_paths(&existing)))
            .unwrap();

    copy_obj().arg(&file).env("PATH", path).assert().success();

    let args = fs::read_to_string(args_out).unwrap();
    assert!(args.contains("-e"));
    assert!(args.contains("set the clipboard to POSIX file"));
    assert!(args.contains(&file.canonicalize().unwrap().display().to_string()));
}
