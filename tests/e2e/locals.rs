//! `dotfiles` のマシンローカル値（named value, §9 / S4 #458）の E2E。
//!
//! `locals` 宣言ユニットの配置ファイルへ `@@name@@` をストア値で注入する挙動を hermetic fixture
//! で検証する: ①ストアに値あり → 置換 ②値なし＋非 TTY（assert_cmd は非 TTY）→ 警告のみ・継続・
//! placeholder はリテラル維持 ③`secret set` がストアを 0600 で作成し以降の apply が置換 ④`doctor`
//! が未設定/設定済みを診断 ⑤`sensitive ⊄ locals` を load 時に弾く。値は決して表示しない。

use crate::dotfiles;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// locals 宣言の copy ユニット（dst=ディレクトリ、`user` に `@@` placeholder）を書き出す。
fn write_locals_unit(work: &Path) {
    let unit = work.join("configs/gituser");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/git\"\nlocals = [\"git.email\", \"git.name\"]\n",
    )
    .unwrap();
    fs::write(
        unit.join("user"),
        "[user]\n\temail = @@git.email@@\n\tname = @@git.name@@\n",
    )
    .unwrap();
}

/// HOME 配下のストア（`~/.config/dotfiles/local.toml`）に内容を書く。
fn write_store(home: &Path, body: &str) {
    let dir = home.join(".config/dotfiles");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("local.toml"), body).unwrap();
}

/// ①ストアに値があれば、配置ファイルの `@@name@@` がその値で置換される。
#[test]
fn apply_substitutes_locals_from_store() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    write_locals_unit(work.path());
    write_store(home.path(), "[git]\nemail = \"me@x\"\nname = \"Me\"\n");

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let out = fs::read_to_string(home.path().join(".config/git/user")).unwrap();
    assert!(
        out.contains("email = me@x"),
        "ストア値が置換されるべき:\n{out}"
    );
    assert!(
        out.contains("name = Me"),
        "ストア値が置換されるべき:\n{out}"
    );
    assert!(
        !out.contains("@@"),
        "全 placeholder が解決され @@ が残らないべき:\n{out}"
    );
}

/// ②値なし＋非 TTY: apply は成功（ブロックしない）し警告を出す。placeholder はリテラルで残る。
#[test]
fn apply_non_tty_warns_and_keeps_placeholder_literal() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    write_locals_unit(work.path()); // ストアは作らない。

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("未設定"))
        .stderr(predicate::str::contains("git.email"));

    let out = fs::read_to_string(home.path().join(".config/git/user")).unwrap();
    assert!(
        out.contains("@@git.email@@") && out.contains("@@git.name@@"),
        "未解決 placeholder はリテラルのまま残るべき（空置換しない）:\n{out}"
    );
}

/// ③`secret set` がストアを 0600 で作成し、続く apply が置換する。値は表示されない。
#[cfg(unix)]
#[test]
fn secret_set_writes_store_0600_then_apply_substitutes() {
    use std::os::unix::fs::PermissionsExt;
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    write_locals_unit(work.path());

    dotfiles()
        .args(["secret", "set", "git.email", "me@x"])
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("git.email"))
        .stdout(predicate::str::contains("me@x").not()); // 値は出さない。

    let store = home.path().join(".config/dotfiles/local.toml");
    let mode = fs::metadata(&store).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600, "ストアは 0600 であるべき");

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success();

    let out = fs::read_to_string(home.path().join(".config/git/user")).unwrap();
    assert!(
        out.contains("email = me@x"),
        "secret set 値で置換されるべき:\n{out}"
    );
    // git.name は未設定なので placeholder のまま。
    assert!(
        out.contains("@@git.name@@"),
        "未設定値は placeholder のまま:\n{out}"
    );
}

/// ④`doctor` が未設定 locals を警告し、設定後は設定済みを報告する。
#[test]
fn doctor_reports_unset_then_set() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    write_locals_unit(work.path());

    dotfiles()
        .arg("doctor")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("未設定"))
        .stdout(predicate::str::contains("git.email"));

    write_store(home.path(), "[git]\nemail = \"me@x\"\nname = \"Me\"\n");
    dotfiles()
        .arg("doctor")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("すべて設定済み"));
}

/// ⑤`sensitive ⊄ locals` の manifest は load 時に弾く（typo の footgun を防ぐ）。
#[test]
fn apply_errors_on_sensitive_not_in_locals() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/bad");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/bad\"\nlocals = [\"git.email\"]\nsensitive = [\"git.token\"]\n",
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
