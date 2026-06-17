//! `clip path <file>`: macOS で絶対パスを pbcopy へ渡し stdout にも出力すること、
//! 非 macOS で失敗。

use std::fs;

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
        .arg("path")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("macOS only"));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_copies_absolute_path_and_prints_it() {
    let fx = TempDir::new().unwrap();
    let file = fx.path().join("note.txt");
    let bin = fx.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(&file, "x\n").unwrap();

    let captured = fx.path().join("pbcopy.stdin");
    write_exec(
        &bin,
        "pbcopy",
        &format!("#!/bin/sh\ncat >\"{}\"\n", captured.display()),
    );

    let abspath = file.canonicalize().unwrap().display().to_string();

    clip()
        .arg("path")
        .arg(&file)
        .env("PATH", path_with(&bin))
        .assert()
        .success()
        .stdout(predicate::str::contains(&abspath));

    assert_eq!(fs::read_to_string(captured).unwrap(), abspath);
}
