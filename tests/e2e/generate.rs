//! `dotfiles apply` の cmd input/output（旧 generate 層）の E2E。
//!
//! 実バイナリに依存せず、PATH 先頭に置いたスタブ（架空 `foo` / [`crate::write_stub`]）で
//! `input.cmd` 実行と when.deps gate、明示 append step、list 表示を検証する。スタブは sh
//! スクリプトなので unix 限定。

use crate::{dotfiles, write_stub};
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// cmd input 単位 `configs/foo/completion`（input.cmd=foo / output=ファイル / when.deps=["foo"]）を
/// 書き出す。
#[cfg(unix)]
fn write_generate_unit(work: &Path) -> std::path::PathBuf {
    let unit = work.join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "when = { deps = [\"foo\"] }\n\
         [[steps]]\n\
         input.cmd = [\"foo\"]\n\
         [[steps]]\n\
         output = \"~/.config/fish/completions/foo.fish\"\n",
    )
    .unwrap();
    unit
}

/// input.cmd が cmd を実行し、その標準出力を output のファイルへ書き出すことを検証する。
#[cfg(unix)]
#[test]
fn apply_input_cmd_runs_cmd_and_writes_output() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "foo", "printf 'complete -c foo -f\\n'\n");
    write_generate_unit(work.path());

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path()) // スタブのみ。foo は PATH 上で解決される。
        .assert()
        .success();

    let placed = home.path().join(".config/fish/completions/foo.fish");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "complete -c foo -f\n",
        "cmd の stdout がそのまま output に書かれていない",
    );
}

/// when.deps gate: 依存バイナリが PATH に無ければユニットごとスキップし、ファイルを作らない（成功終了）。
#[cfg(unix)]
#[test]
fn apply_input_cmd_gate_skips_when_dep_missing() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let empty_bin = tempfile::tempdir().unwrap(); // foo を置かない＝依存欠落。

    write_generate_unit(work.path());

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", empty_bin.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("skip"))
        .stdout(predicate::str::contains("foo"));

    assert!(
        !home
            .path()
            .join(".config/fish/completions/foo.fish")
            .exists(),
        "gate が効かず依存欠落でも生成された",
    );
}

/// 明示 append step: cmd 出力の後ろへ、同梱の静的ファイル（独自補完ブロック相当）を append で
/// 連結する（gh completion の custom.fish と同じ形。旧 generate の暗黙 sibling 連結は廃止された
/// ため、append は明示 step として書く）。
#[cfg(unix)]
#[test]
fn apply_appends_static_step_after_cmd_output() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let bin = tempfile::tempdir().unwrap();

    write_stub(bin.path(), "foo", "printf 'GENERATED\\n'\n");
    let unit = work.path().join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "format = \"text\"\n\
         when   = { deps = [\"foo\"] }\n\
         [[steps]]\n\
         input.cmd = [\"foo\"]\n\
         [[steps]]\n\
         input = \"custom.fish\"\n\
         merge = \"append\"\n\
         [[steps]]\n\
         output = \"~/.config/fish/completions/foo.fish\"\n",
    )
    .unwrap();
    fs::write(unit.join("custom.fish"), "# CUSTOM\n").unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .env("PATH", bin.path())
        .assert()
        .success();

    let placed = home.path().join(".config/fish/completions/foo.fish");
    assert_eq!(
        fs::read_to_string(&placed).unwrap(),
        "GENERATED\n# CUSTOM\n",
        "cmd 出力の後ろへ append step の内容が連結されていない",
    );
}

/// steps に input が 1 つも無い manifest は load 時にエラー（旧「generate で cmd 無し」の直接の
/// 後継: 実体化できない/空のパイプラインを弾く）。
#[test]
fn apply_errors_when_no_input_step() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\noutput = \"~/.config/fish/completions/foo.fish\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("input"));
}

/// `{ cmd = [...] }` の `cmd` キーが無い input（例えば typo で空テーブルだけ書いた）は
/// StepSource のいずれの形（bare string / `{ cmd = [...] }`）にも一致せずパース時にエラーになる。
#[test]
fn apply_errors_when_cmd_table_missing_cmd_key() {
    let work = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "[[steps]]\ninput = {}\n[[steps]]\noutput = \"~/.config/fish/completions/foo.fish\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("apply")
        .current_dir(work.path())
        .env("HOME", home.path())
        .assert()
        .failure();
}

/// `dotfiles list` が cmd input 単位を steps サマリ ＋ deps 付きで表示することを検証する。
#[test]
fn list_shows_steps_summary_with_deps() {
    let work = tempfile::tempdir().unwrap();

    let unit = work.path().join("configs/foo/completion");
    fs::create_dir_all(&unit).unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "when = { deps = [\"foo\"] }\n\
         [[steps]]\n\
         input.cmd = [\"foo\"]\n\
         [[steps]]\n\
         output = \"~/.config/fish/completions/foo.fish\"\n",
    )
    .unwrap();

    dotfiles()
        .arg("list")
        .current_dir(work.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("steps=1in/1out"))
        .stdout(predicate::str::contains("when.deps=foo"));
}
