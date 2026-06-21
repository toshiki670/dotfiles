//! `dotfiles apply` の claude/settings 実 config（S3 / #457）の E2E。
//!
//! 実ソース configs/claude/settings を一時 work へ隔離コピーし、PATH で rtk の有無を
//! 決定的に与えて apply する。json-shallow overlay（base ＋ rtk(when.dep) ＋ preserve=true）が
//! 実ファイルで「ローカル（非管理キー）全温存・共有上書き・rtk 条件付き hook」を満たすことを
//! 検証する（overlay/strategy/when/preserve の純機構は [`crate::overlay`] の hermetic 群が網羅。
//! ここは実 config の結線確認）。rtk スタブ／PATH 制御は unix 限定。

use crate::{dotfiles, write_stub};
use std::fs;
use std::path::Path;

/// 実ソース configs/claude/settings の全ファイルを work/configs/claude/settings へコピーする。
#[cfg(unix)]
fn copy_real_claude_settings(work: &Path) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("configs/claude/settings");
    let dst = work.join("configs/claude/settings");
    fs::create_dir_all(&dst).unwrap();
    for entry in fs::read_dir(&src).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
        }
    }
}

/// 既存 ~/.claude/settings.json を用意する（ローカル編集を模す）。
#[cfg(unix)]
fn write_existing_settings(home: &Path, body: &str) {
    let dst = home.join(".claude/settings.json");
    fs::create_dir_all(dst.parent().unwrap()).unwrap();
    fs::write(&dst, body).unwrap();
}

/// rtk 在: ローカルの非管理キー（model / effortLevel / 任意の localOnly）を全温存しつつ共有
/// （language）を base で上書きし、when.dep=rtk を満たして rtk hook が入る（旧 `$local + $forced`）。
#[cfg(unix)]
#[test]
fn apply_real_claude_settings_preserves_local_and_overrides_with_rtk() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    copy_real_claude_settings(work.path());
    write_stub(bin.path(), "rtk", "exit 0\n"); // 存在＋実行ビットだけで十分（実行はされない）。
    write_existing_settings(
        home.path(),
        "{\"model\":\"opus\",\"effortLevel\":\"high\",\"language\":\"English\",\"localOnly\":\"keepme\"}\n",
    );

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success();

    let out = fs::read_to_string(home.path().join(".claude/settings.json")).unwrap();
    assert!(
        out.contains("\"model\": \"opus\""),
        "ローカル model が温存されるべき:\n{out}"
    );
    assert!(
        out.contains("\"effortLevel\": \"high\""),
        "ローカル effortLevel が温存されるべき:\n{out}"
    );
    assert!(
        out.contains("\"language\": \"日本語\""),
        "共有 language が base で上書きされるべき:\n{out}"
    );
    assert!(
        out.contains("\"localOnly\": \"keepme\""),
        "preserve=true は dotfiles 非管理キー localOnly も土台のまま全温存すべき:\n{out}"
    );
    assert!(
        out.contains("rtk hook claude"),
        "rtk 在で when.dep=rtk 断片の rtk hook が入るべき:\n{out}"
    );
    assert!(
        out.contains("rm guard (trash)"),
        "rm→trash ガードは常に入るべき:\n{out}"
    );
}

/// rtk 不在: rtk hook は脱落するが、base＋preserve で settings.json は書かれる
/// （rm→trash ガード・共有設定・ローカル温存は残る ＝ 無条件化の回帰解消の核）。
#[cfg(unix)]
#[test]
fn apply_real_claude_settings_omits_rtk_hook_when_absent() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // rtk を置かない。

    copy_real_claude_settings(work.path());
    write_existing_settings(home.path(), "{\"model\":\"opus\"}\n");

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", empty_bin.path())
        .assert()
        .success();

    let out = fs::read_to_string(home.path().join(".claude/settings.json")).unwrap();
    assert!(
        !out.contains("rtk hook claude"),
        "rtk 不在なら when.dep 断片は脱落し rtk hook は入らないべき:\n{out}"
    );
    assert!(
        out.contains("rm guard (trash)"),
        "rtk 不在でも rm→trash ガードは残るべき:\n{out}"
    );
    assert!(
        out.contains("\"language\": \"日本語\""),
        "rtk 不在でも base の共有設定が書かれるべき:\n{out}"
    );
    assert!(
        out.contains("\"model\": \"opus\""),
        "rtk 不在でも preserve（ローカル温存）は効くべき:\n{out}"
    );
}
