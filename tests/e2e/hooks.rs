//! `dotfiles apply` の onchange フック（S5 / #459）の E2E。
//!
//! 架空のフックコマンド `faketool`（PATH 先頭スタブ）と temp HOME で、エンジンの契約を検証する:
//! ①onchange skip/run（ソースハッシュ・条件④）②when.os ユニット gate が hooks を覆う（条件③）
//! ③未インストール（PATH 不在）は中断せず skip ④実行して非ゼロ終了は apply エラー
//! ⑤空コマンドの load 時拒否 ⑥list の hooks 表示 ⑦区切り付き相対パス hook は manifest dir 基準で
//! 解決・実行（§13.3 / #498）。

use crate::{dotfiles, write_stub};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// `faketool` の PATH スタブを置く（呼ばれるたび `$HOME/hook-ran` へ 1 行追記＝実行回数の観測点）。
#[cfg(unix)]
fn write_faketool(bin: &Path) {
    write_stub(bin, "faketool", "printf 'x\\n' >> \"$HOME/hook-ran\"\n");
}

/// `faketool` フック実行マーカーの行数（＝実行回数）。未作成なら 0。
fn marker_lines(home: &Path) -> usize {
    fs::read_to_string(home.join("hook-ran"))
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

/// `faketool` を 1 つだけ宣言したユニット（dst＋ソースファイル）を `work` に書き出す。
/// `source_body` を変えるとユニットのソースハッシュが変わり、onchange が再実行を促す。
#[cfg(unix)]
fn write_hook_unit(work: &Path, source_body: &str) {
    let unit = work.join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo\"\nhooks = [[\"faketool\"]]\n",
    )
    .unwrap();
    fs::write(unit.join("data.txt"), source_body).unwrap();
}

/// 共通の apply 実行ヘルパ（HOME と PATH を temp に固定）。
#[cfg(unix)]
fn apply(work: &Path, home: &Path, path: &Path) -> assert_cmd::assert::Assert {
    dotfiles()
        .arg("apply")
        .current_dir(work)
        .env("HOME", home)
        .env("PATH", path)
        .assert()
}

/// onchange gate（条件④）: 初回は実行、ソース不変の再 apply は skip、ソース変化で再実行。
/// `faketool` スタブは呼ばれるたび `$HOME/hook-ran` へ追記するので、行数で実行回数を測る。
#[cfg(unix)]
#[test]
fn hook_runs_on_first_apply_skips_when_unchanged_reruns_on_change() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_faketool(bin.path());
    write_hook_unit(work.path(), "v1");

    // 初回 apply: ソース未記録 → フック実行（マーカー 1 行）。
    apply(work.path(), home.path(), bin.path())
        .success()
        .stdout(predicate::str::contains("hook: faketool"))
        .stdout(predicate::str::contains("ran"));
    assert_eq!(marker_lines(home.path()), 1, "初回はフックが実行されるべき");

    // 2 回目 apply（ソース不変）: onchange で skip（マーカー 1 行のまま）。
    apply(work.path(), home.path(), bin.path())
        .success()
        .stdout(predicate::str::contains("ソース不変"));
    assert_eq!(
        marker_lines(home.path()),
        1,
        "ソース不変ならフックは再実行されないべき",
    );

    // ソース変更 → ソースハッシュが変わり再実行（マーカー 2 行）。
    write_hook_unit(work.path(), "v2");
    apply(work.path(), home.path(), bin.path())
        .success()
        .stdout(predicate::str::contains("ran"));
    assert_eq!(
        marker_lines(home.path()),
        2,
        "ソース変化時のみフックが再実行されるべき",
    );
}

/// 条件③: when.os ユニット gate が false のユニットは配置ごと skip され、その hooks も走らない。
/// `faketool` は PATH にあるが、os 不一致でユニットが skip されるためマーカーは作られない。
#[cfg(unix)]
#[test]
fn os_gate_skips_unit_hooks() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_faketool(bin.path());
    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo\"\nwhen = { os = \"nonsuch-os\" }\nhooks = [[\"faketool\"]]\n",
    )
    .unwrap();
    fs::write(unit.join("data.txt"), "v1").unwrap();

    apply(work.path(), home.path(), bin.path())
        .success()
        .stdout(predicate::str::contains("skip"))
        .stdout(predicate::str::contains("demo"));

    assert_eq!(
        marker_lines(home.path()),
        0,
        "os gate=false のユニットでは hooks も走らないべき",
    );
}

/// プログラム未インストール（PATH 不在）は apply を中断せず skip し、ハッシュを保存しない
/// （chezmoi の `command -v` ガード相当）。後で入れると同じソースでも再実行されることで未保存を示す。
#[cfg(unix)]
#[test]
fn missing_program_skips_without_storing_hash() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // faketool を置かない。
    let bin = tempfile::tempdir().unwrap(); // faketool を置く。

    write_faketool(bin.path());
    write_hook_unit(work.path(), "v1");

    // faketool 不在: 中断せず成功・skip 表示・マーカー無し。
    apply(work.path(), home.path(), empty_bin.path())
        .success()
        .stdout(predicate::str::contains("skip"))
        .stdout(predicate::str::contains("PATH にない"));
    assert_eq!(
        marker_lines(home.path()),
        0,
        "未インストールなら実行されない"
    );

    // faketool を入れて再 apply（ソース不変）: ハッシュ未保存なので実行される（マーカー 1 行）。
    apply(work.path(), home.path(), bin.path()).success();
    assert_eq!(
        marker_lines(home.path()),
        1,
        "未インストール時にハッシュを保存していないので、導入後は再実行されるべき",
    );
}

/// 実行して非ゼロ終了したフックは apply をエラーで止める（未インストールの skip とは区別する）。
#[cfg(unix)]
#[test]
fn nonzero_exit_fails_apply() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "faketool", "exit 1\n");
    write_hook_unit(work.path(), "v1");

    apply(work.path(), home.path(), bin.path())
        .failure()
        .stderr(predicate::str::contains("faketool"))
        .stderr(predicate::str::contains("異常終了"));
}

/// §13.3 / #498: 区切り付き相対パスの hook（`["./hook.sh"]`）はユニットの `manifest.toml`
/// ディレクトリ基準で解決・実行される。apply のプロセス CWD は `work` ルート（unit dir ではない）に
/// 固定するので、もし旧挙動（CWD 継承）なら `work/./hook.sh` は不在で skip されマーカーが残らない。
/// マーカーが作られ、かつ hook の実行時 CWD がユニットディレクトリであることが manifest dir 基準解決の証拠。
#[cfg(unix)]
#[test]
fn relative_path_hook_resolves_against_manifest_dir() {
    use std::os::unix::fs::PermissionsExt;
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo\"\nhooks = [[\"./hook.sh\"]]\n",
    )
    .unwrap();
    fs::write(unit.join("data.txt"), "v1").unwrap();
    // hook 自身の実行時 CWD を $HOME/hook-cwd に書き出す（解決基準と runtime CWD の観測点）。
    let script = unit.join("hook.sh");
    fs::write(&script, "#!/bin/sh\npwd -P > \"$HOME/hook-cwd\"\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();

    // apply のプロセス CWD = work ルート（≠ unit dir）。
    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hook: ./hook.sh"))
        .stdout(predicate::str::contains("ran"));

    // マーカーがある＝相対 hook が解決・実行された。記録された実行時 CWD は manifest ディレクトリ。
    // tempdir は /var → /private/var の symlink 差があるので canonicalize して比較する。
    let cwd = fs::read_to_string(home.path().join("hook-cwd"))
        .expect("相対 hook が実行されマーカーを残すべき（CWD 継承なら不在で skip される）");
    assert_eq!(
        fs::canonicalize(cwd.trim()).unwrap(),
        fs::canonicalize(&unit).unwrap(),
        "相対 hook の実行時 CWD は manifest ディレクトリであるべき",
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

/// `dotfiles list` が hooks 属性（件数）を表示する。
#[test]
fn list_shows_hooks_attr() {
    let work = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/demo");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "dst = \"~/.config/demo\"\nhooks = [[\"faketool\"]]\n",
    )
    .unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hooks=1"));
}
