//! `clip text <file>`: macOS で pbcopy にファイルの中身をそのまま渡すこと、非 macOS で失敗。

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
        .arg("text")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("macOS only"));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_pipes_file_contents_to_pbcopy() {
    let fx = TempDir::new().unwrap();
    let file = fx.path().join("note.txt");
    let bin = fx.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(&file, "hello clip\n").unwrap();

    let captured = fx.path().join("pbcopy.stdin");
    write_exec(
        &bin,
        "pbcopy",
        &format!("#!/bin/sh\ncat >\"{}\"\n", captured.display()),
    );

    clip()
        .arg("text")
        .arg(&file)
        .env("PATH", path_with(&bin))
        .assert()
        .success();

    assert_eq!(fs::read_to_string(captured).unwrap(), "hello clip\n");
}
