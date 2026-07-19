//! マシンローカル値（named value）機構の E2E（S4 / #458）。
//!
//! `dotfiles local set` でストアへ設定 →`apply` で `@@name@@` 注入、未設定時の非 TTY 警告と
//! placeholder 残し、`local list` の一覧、`doctor` の未設定警告を架空 fixture（`demo` 単位）で
//! 検証する。
//!
//! 注: `assert_cmd` の `.assert()` は `Command::output()` 経由で子の stdin を継承しない（= 非 TTY）。
//! よって apply は常に非対話経路（警告のみ）を通り、テストがプロンプトでハングしない。

use crate::dotfiles;
use predicates::prelude::*;
use std::fs;

/// locals を宣言した単位へ、事前 `local set` した値が apply で `@@name@@` 置換されることを検証。
#[test]
fn local_set_then_apply_injects_local_value() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "locals = [\"demo.token\"]\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/demo\"\n",
    )
    .unwrap();
    fs::write(unit.join("conf"), "token = @@demo.token@@\n").unwrap();

    // 同じ HOME（= 同じストア）で local set → apply。
    dotfiles()
        .args(["local", "set", "demo.token", "s3cr3t"])
        .env("HOME", home.path())
        .assert()
        .success();
    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let placed = home.path().join(".config/demo/conf");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "token = s3cr3t\n",
        "ストア値が @@name@@ へ注入されていない",
    );
}

/// 値未設定 ＋ 非 TTY では、警告のみで継続し（ブロックしない）placeholder が literal で残る。
#[test]
fn apply_without_value_warns_and_leaves_placeholder() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "locals = [\"demo.token\"]\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/demo\"\n",
    )
    .unwrap();
    fs::write(unit.join("conf"), "token = @@demo.token@@\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success() // 非 TTY は警告のみで継続（ブロックしない）。
        .stderr(predicate::str::contains("未設定"))
        .stderr(predicate::str::contains("demo.token"));

    let placed = home.path().join(".config/demo/conf");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "token = @@demo.token@@\n",
        "未解決の placeholder は literal で残るべき",
    );
}

/// `local set` がストアを 0600 で作り、値を保持することを検証。
#[cfg(unix)]
#[test]
fn local_set_writes_owner_only_store() {
    use std::os::unix::fs::PermissionsExt;

    let home = tempfile::tempdir().unwrap();
    dotfiles()
        .args(["local", "set", "demo.token", "val"])
        .env("HOME", home.path())
        .assert()
        .success();

    let store = home.path().join(".config/dotfiles/local.toml");
    let mode = fs::metadata(&store).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "ストアは 0600（所有者のみ）で書かれる");
    assert!(
        fs::read_to_string(&store).unwrap().contains("val"),
        "ストアに値が保存されていない",
    );
}

/// `local list` が設定済みの名前→値を名前順で出すことを検証。
#[test]
fn local_list_shows_stored_values_in_name_order() {
    let home = tempfile::tempdir().unwrap();

    for (name, value) in [("git.name", "Toshiki"), ("git.email", "me@example.com")] {
        dotfiles()
            .args(["local", "set", name, value])
            .env("HOME", home.path())
            .assert()
            .success();
    }

    let stdout = dotfiles()
        .args(["local", "list"])
        .env("HOME", home.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(stdout).unwrap();

    let email = stdout.find("git.email").expect("git.email が出ていない");
    let name = stdout.find("git.name").expect("git.name が出ていない");
    assert!(
        email < name,
        "名前順（git.email → git.name）で出ていない:\n{stdout}"
    );
    assert!(
        stdout.contains("me@example.com") && stdout.contains("Toshiki"),
        "値が出ていない:\n{stdout}",
    );
}

/// ストアが空（ファイル未作成）なら `list` / `doctor` と同じ作法で「対象なし」を出して成功する。
#[test]
fn local_list_reports_empty_store() {
    let home = tempfile::tempdir().unwrap();

    dotfiles()
        .args(["local", "list"])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("対象なし"));
}

/// `doctor` が未設定 locals を警告し、`local set` 後は「設定済み」になることを検証。
#[test]
fn doctor_warns_unset_then_clears_after_set() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "locals = [\"demo.token\"]\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/demo\"\n",
    )
    .unwrap();

    // 未設定 → stderr に未設定名を報告（exit 0・ブロックしない雛形）。
    dotfiles()
        .arg("doctor")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("demo.token"));

    // 設定後 → 全て設定済み。
    dotfiles()
        .args(["local", "set", "demo.token", "v"])
        .env("HOME", home.path())
        .assert()
        .success();
    dotfiles()
        .arg("doctor")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("全て設定済み"));
}

/// `sensitive`（旧スキーマ。利用者ゼロで削除・#588 スライス1）はもう manifest のフィールドでない
/// ため、バイナリ経由でも未知フィールドとして load 時に弾かれることを検証する。
#[test]
fn apply_rejects_legacy_sensitive_field() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/bad");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "locals = [\"a\"]\nsensitive = [\"a\"]\n[[steps]]\ninput = \".\"\n[[steps]]\noutput = \"~/.config/bad\"\n",
    )
    .unwrap();
    fs::write(unit.join("f"), "x\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("sensitive"));
}
