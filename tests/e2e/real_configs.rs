//! 出荷する実 configs（`configs/`）の data-driven E2E（#488）。
//!
//! 個々のツール（bat / claude / git / zellij …）は一時データであり、いつか入れ替わる。
//! エンジンとそのテストはツール群より長生きするので、ここでは **特定ツールを名指しせず**、
//! `configs/` 配下の全ユニットを実行時に走査して次を検証する:
//!
//! - 全 manifest が load/validate を通る（apply はユニット gate より先に load するため、
//!   gate で skip されるユニットも含め全件が報告される ＝ §5.5・[`apply`](crate) 評価順）
//! - gate を通って配置されたユニットは dst を実体化する
//! - `dotfiles list` が全ユニットを名前で集約する
//!
//! ツールが増減・改名しても本ファイルは無変更で追従する。ツール固有の合成・注入・再帰の
//! 規則は架空 fixture の hermetic 群（[`crate::overlay`] / [`crate::secrets`] /
//! [`crate::apply_copy`]）が網羅する。ここは「実 configs がエンジンの契約を満たす実体である」
//! ことだけを確かめる結線テスト。

use crate::dotfiles;
use std::fs;
use std::path::{Path, PathBuf};

/// repo の `configs/` ルート（`CARGO_MANIFEST_DIR` 基準）。
fn configs_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("configs")
}

/// `configs/` 配下を走査し、設定単位（`manifest.toml` を持つディレクトリ）の source 相対名
/// （`/` 区切り。`dotfiles list` / `apply` の表示名と一致）を集める。バイナリ側 discover とは
/// 独立に歩く（テストはエンジン内部に依存せず、ソースツリーだけを真理とする）。走査が必ず
/// 1 件以上返る不変条件はここで一度だけ担保する（空＝走査の不具合）。
fn discover_unit_names() -> Vec<String> {
    let root = configs_root();
    let mut names = Vec::new();
    walk(&root, &root, &mut names);
    names.sort();
    assert!(
        !names.is_empty(),
        "configs/ にユニットが無い（走査の不具合）"
    );
    names
}

/// `dir` 以下を再帰的にたどり、`manifest.toml` を持つディレクトリの相対名を `out` へ積む。
fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
    if dir.join("manifest.toml").is_file() {
        let rel = dir.strip_prefix(root).unwrap();
        out.push(rel.to_string_lossy().into_owned());
    }
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, root, out);
        }
    }
}

/// apply が 1 行（`apply: {name} → {dst} ({label})`）で報告した配置先を、test の HOME 上の
/// 実パスへ写す。実 configs の dst は全て `~/…`（HOME 配下）なので先頭 `~/` を temp HOME へ
/// 読み替える。形式・前提を外れたら expect で fail loud（エンジンの展開規則は複製しない）。
fn placed_path(line: &str, home: &Path) -> PathBuf {
    let dst = line
        .split(" → ")
        .nth(1)
        .and_then(|after| after.rsplit_once(" (").map(|(dst, _label)| dst))
        .expect("apply 行が `→ dst (label)` 形式でない");
    let rel = dst
        .strip_prefix("~/")
        .expect("実 configs の dst は ~/ 始まり（HOME 配下）前提");
    home.join(rel)
}

/// 全 manifest が load でき、apply が全ユニットを報告し、gate を通ったユニットは dst を生成する。
///
/// dep gate（`when.deps`）は空 PATH で決定的に外す。load は gate 評価より先なので、空 PATH で
/// skip されるユニットも apply 行で 1 件報告される（= manifest が load/validate を通った証跡）。
#[test]
fn real_configs_all_load_and_placed_units_create_dst() {
    let names = discover_unit_names();

    let home = tempfile::tempdir().unwrap();
    let empty_path = tempfile::tempdir().unwrap(); // PATH を空にし dep gate を決定的に外す。

    let out = dotfiles()
        .arg("apply")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("HOME", home.path())
        .env("PATH", empty_path.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(out).unwrap();

    let line_for = |name: &str| {
        stdout
            .lines()
            .find(|l| l.starts_with(&format!("apply: {name} →")))
    };

    // ① load 網羅: 走査した全ユニットが apply 行で報告される（全 manifest が load/validate を通った）。
    for name in &names {
        assert!(
            line_for(name).is_some(),
            "ユニット {name} が apply で報告されていない（manifest が load されていない）:\n{stdout}",
        );
    }

    // ② dst 生成: skip でない（gate を通った）ユニットは dst を実体化する。
    let mut placed = 0;
    for name in &names {
        let line = line_for(name).unwrap();
        if line.contains("→ skip") {
            continue; // dep/os gate で skip されたユニット（dst を触らないのが正しい挙動）。
        }
        let path = placed_path(line, home.path());
        assert!(
            path.exists(),
            "配置されたユニット {name} の dst が生成されていない: {path:?}\n行: {line}",
        );
        placed += 1;
    }
    assert!(
        placed > 0,
        "空 PATH でも gate の無いユニットは配置されるはず（1 つも配置されていない）:\n{stdout}",
    );
}

/// `dotfiles list` が走査した全ユニットを名前で集約表示する。
#[test]
fn real_configs_listed_by_unit_name() {
    let names = discover_unit_names();

    let out = dotfiles()
        .arg("list")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(out).unwrap();

    // 先頭列（単位名）で照合する。dst や属性に名前の部分文字列が現れても引っ張られない。
    for name in &names {
        assert!(
            stdout
                .lines()
                .any(|l| l.split_whitespace().next() == Some(name.as_str())),
            "list にユニット {name} が出ていない:\n{stdout}",
        );
    }
}
