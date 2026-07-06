//! マシンクラス状態 gate（`profile`）の E2E（#467）。
//!
//! `dotfiles profile` の設定／表示と、`when = { profile = … }` を持つ架空ユニットが現在の profile
//! 状態で配置／skip されること（既定 not-private）・冪等性を検証する。実 yt/mise は名指ししない
//! （架空 fixture `demo` で書く）。
//!
//! 注: `assert_cmd` の `.assert()` は非 TTY なので、profile gate を通った locals 宣言付きユニットは
//! 警告のみで継続する（本ファイルの gate fixture は locals を持たないので注入経路には入らない）。

use crate::dotfiles;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// `profile` 状態を持つ架空ユニットを `work/configs/demo` に用意する（ツリー配置）。
/// `body` は `manifest.toml` の追加行（`when` 等）。
fn write_gated_unit(work: &Path, body: &str) {
    let unit = work.join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        format!("{body}[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/demo\"\n"),
    )
    .unwrap();
    fs::write(unit.join("conf"), "x\n").unwrap();
}

/// `dotfiles profile <name>` が状態を書き、`dotfiles profile`（引数なし）がそれを表示する。
#[test]
fn profile_set_then_show_reports_current() {
    let home = tempfile::tempdir().unwrap();

    // 未設定: 既定（どの profile gate 付き設定も未配置）を、特定 profile 名に依らず表示。
    dotfiles()
        .arg("profile")
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("未設定"));

    // 設定 → 状態ファイルが書かれる。
    dotfiles()
        .args(["profile", "private"])
        .env("HOME", home.path())
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(home.path().join(".config/dotfiles/profile")).unwrap(),
        "private\n",
        "profile 状態ファイルが書かれていない",
    );

    // 表示 → 現在値。
    dotfiles()
        .arg("profile")
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("profile: private"));
}

/// profile 未設定（既定 not-private）では `when = { profile = "private" }` のユニットは skip され、
/// dst を一切触らない（all-or-nothing）。
#[test]
fn profile_gate_skips_unit_when_unset() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    write_gated_unit(work.path(), "when = { profile = \"private\" }\n");

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("demo → skip"))
        .stdout(predicate::str::contains("profile"));

    assert!(
        !home.path().join(".config/demo/conf").exists(),
        "profile 未設定なのに gate されたユニットが配置された",
    );
}

/// `dotfiles profile private` 後は同じユニットが配置される（gate を通る）。冪等性も確認。
#[test]
fn profile_gate_places_unit_when_matched_and_is_idempotent() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    write_gated_unit(work.path(), "when = { profile = \"private\" }\n");

    dotfiles()
        .args(["profile", "private"])
        .env("HOME", home.path())
        .assert()
        .success();

    let placed = home.path().join(".config/demo/conf");
    for _ in 0..2 {
        dotfiles()
            .arg("apply")
            .current_dir(work.path())
            .env("HOME", home.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("demo → ~/.config/demo"));
        assert_eq!(
            fs::read_to_string(&placed).unwrap(),
            "x\n",
            "gate を通ったユニットが配置されていない／冪等でない",
        );
    }
}

/// profile 不一致（`private` を要求するが現在は `work`）では skip する。
#[test]
fn profile_gate_skips_on_mismatch() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    write_gated_unit(work.path(), "when = { profile = \"private\" }\n");

    dotfiles()
        .args(["profile", "work"])
        .env("HOME", home.path())
        .assert()
        .success();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("demo → skip"));

    assert!(
        !home.path().join(".config/demo/conf").exists(),
        "profile 不一致なのに配置された",
    );
}
