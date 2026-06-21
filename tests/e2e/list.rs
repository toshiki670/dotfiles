//! `dotfiles list` の E2E — 分散 manifest の集約・名前順・属性ラベル・ソース欠落。

use crate::dotfiles;
use predicates::prelude::*;
use std::fs;

/// `dotfiles list` が実 configs の分散 manifest を集約し、配置先一覧を表示する。
#[test]
fn list_aggregates_real_manifests() {
    dotfiles()
        .arg("list")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .assert()
        .success()
        .stdout(predicate::str::contains("bat"))
        .stdout(predicate::str::contains("nvim"))
        .stdout(predicate::str::contains("zellij"))
        .stdout(predicate::str::contains("claude/settings"))
        .stdout(predicate::str::contains("~/.config/"))
        .stdout(predicate::str::contains("~/.claude/settings.json"))
        .stdout(predicate::str::contains("copy"))
        .stdout(predicate::str::contains("json-shallow"));
}

/// `dotfiles list` が単位を名前順に並べ、dst と属性ラベルを表示することを検証する。
#[test]
fn list_shows_units_sorted_with_attrs() {
    let work = tempfile::tempdir().unwrap();

    let beta = work.path().join("configs/beta");
    let alpha = work.path().join("configs/alpha");
    fs::create_dir_all(&beta).unwrap();
    fs::create_dir_all(&alpha).unwrap();
    fs::write(beta.join("manifest.toml"), "dst = \"~/.config/beta\"\n").unwrap();
    fs::write(
        alpha.join("manifest.toml"),
        "dst = \"~/.config/alpha\"\nprivate = true\n",
    )
    .unwrap();

    let out = dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(out).unwrap();

    // 並び順は先頭列（単位名）だけで判定する。dst や属性に名前の部分文字列が
    // 現れても（例: ~/.config/alpha）引っ張られないよう、行を split して先頭トークンで照合する。
    let name_row = |name: &str| {
        stdout
            .lines()
            .position(|l| l.split_whitespace().next() == Some(name))
    };
    let a = name_row("alpha").expect("alpha 行が無い");
    let b = name_row("beta").expect("beta 行が無い");
    assert!(a < b, "名前順（先頭列）に並んでいない:\n{stdout}");
    assert!(
        stdout.contains("~/.config/alpha"),
        "dst が出ていない:\n{stdout}"
    );
    assert!(
        stdout.contains("copy, private"),
        "属性ラベルが出ていない:\n{stdout}",
    );
}

/// `configs/` が無い場所で list するとエラー終了することを検証する。
#[test]
fn list_errors_when_source_missing() {
    let work = tempfile::tempdir().unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("が見つかりません"));
}
