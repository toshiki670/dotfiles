//! 不要になった配置の追跡・退避（#521）。
//!
//! 除去条件は「前回 apply 時点の期待配置集合 ∖ 今回の期待配置集合」（[`crate::placements::expected_gated`]、
//! 両辺とも gate 評価後）。3 つの取り残し経路（ユニット gate flip・ユニット削除・ツリーファイル削除）は
//! この 1 式に収束するため、経路ごとの特例は無い。
//!
//! # snapshot（`~/.config/dotfiles/placements`）
//!
//! 「前回」を持つのは永続台帳（path → owner の登録簿）ではなく、単純な相対パス一覧のスナップショット。
//! owner は毎回宣言から再導出できるため、これで足りる。**書き込みは 2 種類だけ**:
//! - [`union`]（[`crate::apply`] が毎回の apply 成功後に呼ぶ）: `snapshot ∪ 今回`。**縮めない**。
//!   apply 自体は上書きしない ― 上書きすると「apply の直後に確認したら diff が空」になり機能が
//!   死ぬ（apply が current を snapshot へ即座に反映してしまうため）。union なら、2 回の
//!   [`commit`] の間に複数回 apply が挟まっても、その間に一瞬でも「期待」だった配置は snapshot に
//!   残り続けるので取りこぼさない。
//! - [`commit`]（`dotfiles apply --force` が呼ぶ opt-in の実削除）: stale 候補を退避したら
//!   `snapshot := 今回`（縮める）。候補が 0 件でも常にリセットする（初回 `--force` で baseline を
//!   作る・[`stale`] が「実在しない」として除外した古い行を snapshot から掃除する）。
//!
//! 既定（`--force` 無し）は報告のみ（[`crate::doctor`] が [`stale`] を読んで表示）で、snapshot・
//! ファイルシステムのいずれも変更しない。
//!
//! snapshot が無い（初回）・読み込み自体に失敗した場合は空集合として扱う（warn 付き）。行ごとの
//! 検証にも同じ安全側フォールバックを適用する: 絶対パス・`..` を含む行（本来書かれないはずだが、
//! 壊れる／改ざんされた場合に home の外を指しうる）はその行だけ無視する（warn 付き）。diff の向き
//! （snapshot ∖ 今回）上、集合が減る方向の判定ミスは候補が減るだけ＝安全側に倒れる。
//!
//! # 退避（trash）
//!
//! 削除は unlink ではなく `~/.config/dotfiles/trash/<unix 秒>/<home 相対パス>` への move（[`quarantine`]）。
//! 対象はレギュラーファイルのみ（`symlink_metadata` で検証）― output パスの表記は `~` 単体
//! （home 直下）も許すため、将来もし非ツリーユニットが `output = "~"` を宣言して削除された場合、
//! この検証が無いと候補パスが `$HOME` そのものになり得る。ディレクトリ・symlink・その他は
//! 退避せず失敗として扱う（黙って skip しない ― 宣言と実体が食い違っている兆候のため）。

use crate::placements::Placement;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// snapshot ファイルのパス（`<home>/.config/dotfiles/placements`）。
fn snapshot_path(home: &Path) -> PathBuf {
    home.join(".config/dotfiles/placements")
}

/// 退避先ルート（`<home>/.config/dotfiles/trash`）。
fn trash_root(home: &Path) -> PathBuf {
    home.join(".config/dotfiles/trash")
}

/// `home` を基点にした相対パスへ変換する（snapshot は home 相対で持つ ― home の違うマシン間で
/// コピーしても意味が壊れず、絶対パス表記より読みやすい）。
fn to_relative(home: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(home).unwrap_or(path).to_path_buf()
}

/// snapshot を読む。無い・壊れていれば空（[`crate::hooks::onchange::State`] と同じ安全側フォール
/// バック）。行ごとに 1 相対パス。空行・前後空白は無視する。絶対パス・`..` を含む行は
/// [`is_safe_relative`] が弾く（本来 [`to_relative`] が書いた行しか無いはずだが、snapshot が
/// 壊れる／改ざんされた場合に home の外を指す行を通すと [`stale`] 経由で home 外のファイルを
/// 退避しかねないため、読み込み時点で一行ずつ検証する）。
fn load(home: &Path) -> Vec<PathBuf> {
    let path = snapshot_path(home);
    match std::fs::read_to_string(&path) {
        Ok(text) => text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let rel = PathBuf::from(line);
                if is_safe_relative(&rel) {
                    Some(rel)
                } else {
                    eprintln!("apply: {} の不正な行を無視します: {line}", path.display());
                    None
                }
            })
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => {
            eprintln!(
                "apply: {} の読み込みに失敗（空として続行）: {e}",
                path.display()
            );
            Vec::new()
        }
    }
}

/// `rel` が安全な相対パスか（絶対パス・`..`（親ディレクトリ参照）を含まない）。`home.join(rel)` は
/// `rel` が絶対パスならそれで丸ごと置き換わり、`..` を含めば home の外を指しうるため、snapshot の
/// 行を home へ結合する前にここで弾く。
fn is_safe_relative(rel: &Path) -> bool {
    !rel.is_absolute()
        && !rel
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
}

/// snapshot を書く（相対パスをソートして決定的に）。
fn write(home: &Path, relative_paths: &std::collections::BTreeSet<PathBuf>) -> Result<(), String> {
    let path = snapshot_path(home);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    let mut text = String::new();
    for p in relative_paths {
        text.push_str(&p.to_string_lossy());
        text.push('\n');
    }
    std::fs::write(&path, text).map_err(|e| format!("{}: 書き込み失敗: {e}", path.display()))
}

/// snapshot を `snapshot ∪ current` に更新する（縮めない）。`dotfiles apply` が毎回の成功時に呼ぶ。
pub fn union(home: &Path, current: &[Placement]) -> Result<(), String> {
    let mut merged: std::collections::BTreeSet<PathBuf> = load(home).into_iter().collect();
    merged.extend(current.iter().map(|p| to_relative(home, &p.path)));
    write(home, &merged)
}

/// snapshot を `current` に置き換える（縮める）。[`commit`] が退避成功後にだけ呼ぶ。
fn reset(home: &Path, current: &[Placement]) -> Result<(), String> {
    let set: std::collections::BTreeSet<PathBuf> =
        current.iter().map(|p| to_relative(home, &p.path)).collect();
    write(home, &set)
}

/// 除去候補（`snapshot ∖ current`）を返す。実在しない（既に手で消された等）パスは対象外にする ―
/// 何も無い所を退避しようとするのはエラーではなく無害なので、報告からも静かに落とす。
///
/// [`crate::doctor`]（報告のみ）と [`commit`]（実退避）が共有する。
pub fn stale(home: &Path, current: &[Placement]) -> Vec<PathBuf> {
    let current_rel: std::collections::BTreeSet<PathBuf> =
        current.iter().map(|p| to_relative(home, &p.path)).collect();
    load(home)
        .into_iter()
        .filter(|rel| !current_rel.contains(rel))
        .map(|rel| home.join(rel))
        .filter(|abs| is_regular_file(abs))
        .collect()
}

/// レギュラーファイルか（symlink・ディレクトリ・不在は false）。symlink を辿らない
/// （dotfiles は symlink を配置しない契約なので、symlink が現れているのは想定外の状態）。
fn is_regular_file(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_file())
        .unwrap_or(false)
}

/// stale 候補を退避し、snapshot を `current` へリセットする。`dotfiles apply --force` が呼ぶ
/// opt-in の実削除。戻り値は実際に退避した元パスの一覧（表示用）。
///
/// `reset` は候補の有無に関わらず**常に**呼ぶ。候補が無いからと省略すると、(a) snapshot がまだ
/// 無い初回 `--force` で baseline が作られないままになり、(b) [`stale`] が「実在しない」として
/// 除外した古い行（手動削除済みのパス）が snapshot に残り続け、後日たまたま同じパスへ別のファイル
/// が現れた時にそれを stale 候補と誤認する、という 2 つの取りこぼしを生む。
pub fn commit(home: &Path, current: &[Placement]) -> Result<Vec<PathBuf>, String> {
    let candidates = stale(home, current);
    let mut moved = Vec::new();
    if !candidates.is_empty() {
        let run_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());
        let run_dir = trash_root(home).join(&run_id);
        std::fs::create_dir_all(&run_dir)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", run_dir.display()))?;
        secure_dir(&run_dir)?;

        for path in &candidates {
            quarantine(home, &run_dir, path)?;
            moved.push(path.clone());
        }
    }

    reset(home, current)?;
    Ok(moved)
}

/// 所有者のみアクセス可（0700）にする。退避先は元が `private`（0600）だったファイルも受けるため。
#[cfg(unix)]
fn secure_dir(dir: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| format!("{}: パーミッション設定失敗: {e}", dir.display()))
}

#[cfg(not(unix))]
fn secure_dir(_dir: &Path) -> Result<(), String> {
    Ok(())
}

/// `path`（`home` 配下の絶対パス）を `run_dir` 配下の同一相対構造へ move する。レギュラーファイル
/// であることを再検証してから動かす（[`stale`] は退避直前の状態を再確認しないため、その間に
/// 差し替えられた場合の防壁として二重に見る）。同一ファイルシステム内は `rename`（属性を保った
/// まま原子的に移動）、`EXDEV`（マウント境界越え）等で失敗すれば copy + remove にフォールバックする。
fn quarantine(home: &Path, run_dir: &Path, path: &Path) -> Result<(), String> {
    if !is_regular_file(path) {
        // stale() 通過後に消えた／置き換わった。消えていれば何もせず正常終了、それ以外
        // （ディレクトリ・symlink 等へ差し替わっていた）は宣言と実体の食い違いとしてエラーにする。
        return match std::fs::symlink_metadata(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Ok(_) => Err(format!(
                "{}: レギュラーファイルではないため退避を中止しました",
                path.display()
            )),
            Err(e) => Err(format!("{}: 状態確認に失敗: {e}", path.display())),
        };
    }

    let rel = to_relative(home, path);
    let dst = run_dir.join(&rel);
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", parent.display()))?;
    }
    if std::fs::rename(path, &dst).is_ok() {
        return Ok(());
    }
    // マウント境界越え等で rename が失敗した場合は copy + remove にフォールバックする。
    std::fs::copy(path, &dst).map_err(|e| format!("{}: 退避コピー失敗: {e}", path.display()))?;
    std::fs::remove_file(path).map_err(|e| format!("{}: 退避元の削除に失敗: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placement(unit: &str, path: PathBuf) -> Placement {
        Placement {
            unit: unit.to_string(),
            path,
        }
    }

    #[test]
    fn missing_snapshot_loads_empty() {
        let home = tempfile::tempdir().unwrap();
        assert_eq!(load(home.path()), Vec::<PathBuf>::new());
    }

    #[test]
    fn union_accumulates_across_calls_without_shrinking() {
        let home = tempfile::tempdir().unwrap();
        let a = placement("a", home.path().join("a.txt"));
        let b = placement("b", home.path().join("b.txt"));

        union(home.path(), std::slice::from_ref(&a)).unwrap();
        // 2 回目は a を含まない current だが、union は縮めない。
        union(home.path(), std::slice::from_ref(&b)).unwrap();

        let snapshot = load(home.path());
        assert!(snapshot.contains(&PathBuf::from("a.txt")));
        assert!(snapshot.contains(&PathBuf::from("b.txt")));
    }

    #[test]
    fn stale_is_snapshot_minus_current_and_requires_existing_regular_file() {
        let home = tempfile::tempdir().unwrap();
        let gone = home.path().join("gone.txt");
        let still_there = home.path().join("still-there.txt");
        std::fs::write(&still_there, "x").unwrap();
        // gone.txt は snapshot にだけ存在し、実ファイルは無い ― 対象外になる。
        union(
            home.path(),
            &[
                placement("a", gone.clone()),
                placement("b", still_there.clone()),
            ],
        )
        .unwrap();

        // 今回の期待集合は空（両方とも消えた体）。
        let current: Vec<Placement> = Vec::new();
        let stale_paths = stale(home.path(), &current);

        assert_eq!(stale_paths, vec![still_there]);
    }

    #[test]
    fn commit_moves_stale_files_and_resets_snapshot() {
        let home = tempfile::tempdir().unwrap();
        let removed = home.path().join(".config/foo/bar.conf");
        std::fs::create_dir_all(removed.parent().unwrap()).unwrap();
        std::fs::write(&removed, "old").unwrap();
        union(home.path(), &[placement("foo", removed.clone())]).unwrap();

        let current: Vec<Placement> = Vec::new(); // foo はもう期待されない。
        let moved = commit(home.path(), &current).unwrap();

        assert_eq!(moved, vec![removed.clone()]);
        assert!(!removed.exists(), "元の場所からは消える");
        assert!(
            load(home.path()).is_empty(),
            "snapshot は現在集合へリセットされる"
        );

        // 退避先に同一相対構造で実体がある。
        let trash = trash_root(home.path());
        let run_dirs: Vec<_> = std::fs::read_dir(&trash).unwrap().collect();
        assert_eq!(run_dirs.len(), 1);
        let quarantined = run_dirs[0]
            .as_ref()
            .unwrap()
            .path()
            .join(".config/foo/bar.conf");
        assert_eq!(std::fs::read_to_string(quarantined).unwrap(), "old");
    }

    #[test]
    fn commit_with_nothing_stale_is_a_noop() {
        let home = tempfile::tempdir().unwrap();
        let moved = commit(home.path(), &[]).unwrap();
        assert!(moved.is_empty());
        assert!(!trash_root(home.path()).exists(), "退避先は作られない");
    }

    #[test]
    fn first_ever_force_run_establishes_baseline_even_with_nothing_to_move() {
        // snapshot が無い状態でいきなり --force しても baseline は作られる（union を挟まずに
        // --force から始めても機能が死なない）。
        let home = tempfile::tempdir().unwrap();
        let a = placement("a", home.path().join("a.txt"));

        let moved = commit(home.path(), std::slice::from_ref(&a)).unwrap();

        assert!(moved.is_empty());
        assert_eq!(load(home.path()), vec![PathBuf::from("a.txt")]);
    }

    #[test]
    fn commit_clears_phantom_entries_even_without_other_candidates() {
        // gone.txt は snapshot にだけ存在し実ファイルは無いため stale() の候補には出ない。
        // それでも commit は snapshot をリセットし、この幽霊エントリを消す（消さないと、後日
        // 同じパスに無関係なファイルが現れた時に誤って退避対象になりうる）。
        let home = tempfile::tempdir().unwrap();
        let gone = home.path().join("gone.txt");
        union(home.path(), &[placement("a", gone)]).unwrap();

        let moved = commit(home.path(), &[]).unwrap();

        assert!(moved.is_empty(), "実在しない候補は退避対象にならない");
        assert!(
            load(home.path()).is_empty(),
            "候補が無くても snapshot はリセットされ、幽霊エントリは消える"
        );
    }

    #[test]
    fn load_ignores_absolute_and_parent_dir_lines() {
        // snapshot が壊れる／改ざんされた場合の防壁: home の外を指しうる行はその行だけ無視する。
        let home = tempfile::tempdir().unwrap();
        let path = snapshot_path(home.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "safe.txt\n/etc/passwd\n../../etc/passwd\nnested/../../escape\n",
        )
        .unwrap();

        assert_eq!(load(home.path()), vec![PathBuf::from("safe.txt")]);
    }

    #[cfg(unix)]
    #[test]
    fn quarantine_refuses_directories() {
        let home = tempfile::tempdir().unwrap();
        let dir_path = home.path().join("a-directory");
        std::fs::create_dir_all(&dir_path).unwrap();
        let run_dir = tempfile::tempdir().unwrap();

        let err = quarantine(home.path(), run_dir.path(), &dir_path).unwrap_err();
        assert!(err.contains("レギュラーファイルではない"));
        assert!(dir_path.exists(), "拒否した場合は元の場所に残る");
    }
}
