//! `dotfiles apply` の onchange フック（S5 / #459）の E2E。
//!
//! フックは manifest がコマンド（argv）をデータとして宣言し、binary は汎用に実行する。実バイナリ
//! （bat 等）に依存せず、PATH 先頭スタブ（[`crate::write_stub`]）と temp HOME で挙動を hermetic に
//! 検証する。中心は **onchange gate**: ユニットのソースが前回適用時と同じならフックを skip、変化
//! （または初回）なら実行する（受け入れ条件④）。あわせて os ユニット gate が hooks を覆うこと
//! （条件③）・空コマンドの load 時拒否・ghostty symlink を汎用エンジンで実行（条件②, macOS 限定）・
//! list の hooks 表示を確認する。

use crate::{dotfiles, write_stub};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// `bat` フック実行のマーカー（スタブが 1 回呼ばれるごとに 1 行追記）の行数。未作成なら 0。
fn marker_lines(home: &Path) -> usize {
    fs::read_to_string(home.join("bat-ran"))
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

/// `bat cache --build` フック（コマンド argv）を宣言した bat ユニット（dst＋theme ファイル）を
/// `work` に書き出す。`theme_body` を変えるとユニットのソースハッシュが変わり、onchange が再実行を促す。
#[cfg(unix)]
fn write_bat_unit(work: &Path, theme_body: &str) {
    let unit = work.join("configs/bat");
    fs::create_dir_all(unit.join("themes")).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/bat\"\nhooks = [[\"bat\", \"cache\", \"--build\"]]\n",
    )
    .unwrap();
    fs::write(unit.join("themes/ayu.tmTheme"), theme_body).unwrap();
}

/// onchange gate（条件④）: 初回は実行、ソース不変の再 apply は skip、ソース変化で再実行。
/// `bat` スタブは呼ばれるたび `$HOME/bat-ran` へ 1 行追記するので、行数で実行回数を測る。
#[cfg(unix)]
#[test]
fn hook_runs_on_first_apply_skips_when_unchanged_reruns_on_change() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "bat", "printf 'x\\n' >> \"$HOME/bat-ran\"\n");
    write_bat_unit(work.path(), "v1");

    // 初回 apply: ソース未記録 → フック実行（マーカー 1 行）。
    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hook: bat cache --build"))
        .stdout(predicate::str::contains("ran"));
    assert_eq!(marker_lines(home.path()), 1, "初回はフックが実行されるべき");

    // 2 回目 apply（ソース不変）: onchange で skip（マーカー 1 行のまま）。
    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ソース不変"));
    assert_eq!(
        marker_lines(home.path()),
        1,
        "ソース不変ならフックは再実行されないべき",
    );

    // theme を変更 → ソースハッシュが変わり再実行（マーカー 2 行）。
    write_bat_unit(work.path(), "v2");
    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ran"));
    assert_eq!(
        marker_lines(home.path()),
        2,
        "ソース変化時のみフックが再実行されるべき",
    );
}

/// 条件③: os ユニット gate が false のユニットは配置ごと skip され、その hooks も走らない。
/// bat スタブは PATH にあるが、os 不一致でユニットが skip されるためマーカーは作られない。
#[cfg(unix)]
#[test]
fn os_gate_skips_unit_hooks() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "bat", "printf 'x\\n' >> \"$HOME/bat-ran\"\n");
    let unit = work.path().join("configs/bat");
    fs::create_dir_all(unit.join("themes")).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/bat\"\nos = \"nonsuch-os\"\nhooks = [[\"bat\", \"cache\", \"--build\"]]\n",
    )
    .unwrap();
    fs::write(unit.join("themes/ayu.tmTheme"), "v1").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("skip"))
        .stdout(predicate::str::contains("bat"));

    assert_eq!(
        marker_lines(home.path()),
        0,
        "os gate=false のユニットでは hooks も走らないべき",
    );
}

/// 空のコマンド（argv）を持つ hook は load 時に弾く（apply 失敗）。実体化できない typo を黙殺しない。
#[test]
fn empty_hook_command_fails_at_load() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo\"\nhooks = [[]]\n",
    )
    .unwrap();
    fs::write(unit.join("f.txt"), "x\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("hooks"))
        .stderr(predicate::str::contains("非空"));
}

/// 条件②（macOS 限定）: 汎用エンジンが manifest 宣言の symlink コマンドを実行し、config を配置して
/// macOS 参照先へ symlink を張る（ツール固有の Rust コードは無く、コマンドはデータ）。
#[cfg(target_os = "macos")]
#[test]
fn ghostty_macos_symlink_links_config() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/ghostty");
    fs::create_dir_all(&unit).unwrap();
    // 実 configs/ghostty と同じ symlink コマンド（argv をデータとして宣言）。
    fs::write(
        unit.join("manifest.toml"),
        r#"dst = "~/.config/ghostty"
os = "darwin"
hooks = [
  ["sh", "-c", "mkdir -p \"$HOME/Library/Application Support/com.mitchellh.ghostty\" && ln -sf \"$HOME/.config/ghostty/config\" \"$HOME/Library/Application Support/com.mitchellh.ghostty/config\""],
]
"#,
    )
    .unwrap();
    fs::write(unit.join("config"), "theme = dark\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    // config が ~/.config/ghostty へ配置される。
    let placed = home.path().join(".config/ghostty/config");
    assert_eq!(fs::read_to_string(&placed).unwrap(), "theme = dark\n");

    // macOS 参照先が config への symlink になっている。
    let link = home
        .path()
        .join("Library/Application Support/com.mitchellh.ghostty/config");
    let target = fs::read_link(&link).expect("symlink が作られていない");
    assert_eq!(
        target, placed,
        "symlink が ~/.config/ghostty/config を指していない",
    );
}

/// `dotfiles list` が hooks 属性（件数）を表示する。
#[test]
fn list_shows_hooks_attr() {
    let work = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/bat");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/bat\"\nhooks = [[\"bat\", \"cache\", \"--build\"]]\n",
    )
    .unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hooks=1"));
}
