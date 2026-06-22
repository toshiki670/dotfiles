//! マシンローカル値（named value）機構の E2E（S4 / #458）。
//!
//! `dotfiles secret set` でストアへ設定 →`apply` で `@@name@@` 注入、未設定時の非 TTY 警告と
//! placeholder 残し、`doctor` の未設定警告、実 config（configs/git）の user 注入を検証する。
//! sensitive の非エコー対話（TTY）は pty 依存で自動化困難なため手動検証（PR 説明参照）。
//!
//! 注: `assert_cmd` の `.assert()` は `Command::output()` 経由で子の stdin を継承しない（= 非 TTY）。
//! よって apply は常に非対話経路（警告のみ）を通り、テストがプロンプトでハングしない。

use crate::dotfiles;
use predicates::prelude::*;
use std::fs;

/// locals を宣言した単位へ、事前 `secret set` した値が apply で `@@name@@` 置換されることを検証。
#[test]
fn secret_set_then_apply_injects_local_value() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo\"\nlocals = [\"demo.token\"]\n",
    )
    .unwrap();
    fs::write(unit.join("conf"), "token = @@demo.token@@\n").unwrap();

    // 同じ HOME（= 同じストア）で secret set → apply。
    dotfiles()
        .args(["secret", "set", "demo.token", "s3cr3t"])
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
        "dst = \"~/.config/demo\"\nlocals = [\"demo.token\"]\n",
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

/// `secret set` がストアを 0600 で作り、値を保持することを検証。
#[cfg(unix)]
#[test]
fn secret_set_writes_owner_only_store() {
    use std::os::unix::fs::PermissionsExt;

    let home = tempfile::tempdir().unwrap();
    dotfiles()
        .args(["secret", "set", "demo.token", "val"])
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

/// `doctor` が未設定 locals を警告し、`secret set` 後は「設定済み」になることを検証。
#[test]
fn doctor_warns_unset_then_clears_after_set() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo\"\nlocals = [\"demo.token\"]\n",
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
        .args(["secret", "set", "demo.token", "v"])
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

/// load 時検証 `sensitive ⊆ locals` がバイナリ経由でも apply を失敗させることを検証。
#[test]
fn apply_rejects_sensitive_not_in_locals() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/bad");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/bad\"\nlocals = [\"a\"]\nsensitive = [\"b\"]\n",
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

/// 実 config: configs/git の user.email/user.name が apply 時にストア値で注入される。
#[test]
fn apply_injects_real_git_user() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let home = tempfile::tempdir().unwrap();

    dotfiles()
        .args(["secret", "set", "git.email", "me@example.com"])
        .env("HOME", home.path())
        .assert()
        .success();
    dotfiles()
        .args(["secret", "set", "git.name", "Toshiki"])
        .env("HOME", home.path())
        .assert()
        .success();
    dotfiles()
        .arg("apply")
        .current_dir(repo_root)
        .env("HOME", home.path())
        .assert()
        .success();

    let user = fs::read_to_string(home.path().join(".config/git/configs/user")).unwrap();
    assert!(
        user.contains("email = me@example.com") && user.contains("name = Toshiki"),
        "git user へストア値が注入されていない:\n{user}",
    );
    assert!(
        !user.contains("@@git."),
        "未置換の placeholder が残っている:\n{user}",
    );
}
