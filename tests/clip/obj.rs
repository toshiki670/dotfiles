//! `clip obj <file>`: macOS で osascript を呼ぶこと、非 macOS で `macOS only` 失敗。

use std::fs;

#[cfg(not(target_os = "macos"))]
use predicates::prelude::*;
#[cfg(target_os = "macos")]
use tempfile::TempDir;

use crate::clip;
#[cfg(target_os = "macos")]
use crate::{path_with, write_exec};

#[cfg(not(target_os = "macos"))]
#[test]
fn non_macos_fails_with_macos_only() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("x.txt");
    fs::write(&file, "x\n").unwrap();

    clip()
        .arg("obj")
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
    write_exec(
        &bin,
        "osascript",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >\"{}\"\n",
            args_out.display()
        ),
    );

    clip()
        .arg("obj")
        .arg(&file)
        .env("PATH", path_with(&bin))
        .assert()
        .success();

    let args = fs::read_to_string(args_out).unwrap();
    assert!(args.contains("-e"));
    assert!(args.contains("set the clipboard to POSIX file"));
    assert!(args.contains(&file.canonicalize().unwrap().display().to_string()));
}
