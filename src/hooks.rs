//! onchange フック（§13, S5）: ユニット配置後（after フェーズ）に走る組み込み処理。
//!
//! chezmoi の `run_*` スクリプトのうち本スライス対象の 2 つを native Rust へ移植する:
//! - `bat-cache` … `bat cache --build`（カスタムテーマ ayu-dark を bat/delta に登録）。
//! - `ghostty-macos-symlink` … `~/Library/Application Support/.../config` → `~/.config/ghostty/config`
//!   の symlink を macOS のアプリ参照先に張る（ghostty ユニットの `os = "darwin"` で gate）。
//!
//! フックは manifest の `hooks` 属性で宣言され、フック名は [`KNOWN`] レジストリと突き合わせて
//! load 時に検証される（[`crate::manifest`]）。実行は **onchange gate**（[`crate::onchange`]）を
//! 通す: ユニットのソースハッシュが前回適用時と同じならスキップ、変化（または初回）なら実行する。
//! ユニット gate（`deps` / `os`）が false のユニットは配置ごと skip されるため、その hooks も
//! [`crate::apply`] に到達せず走らない（＝ os 属性でフックを分岐できる）。
//!
//! cargo-install フックは本スライス対象外（upgrade-env が別途担うため、`dotfiles apply` で
//! 走らせるかは別 issue で検討）。

use crate::onchange::{self, State};
use crate::{gate, generate};
use std::path::Path;

/// 組み込みフック名のレジストリ（manifest の `hooks` 検証に使う）。
pub const KNOWN: &[&str] = &["bat-cache", "ghostty-macos-symlink"];

/// フック名が組み込みレジストリにあるか（[`crate::manifest`] の load 時検証が参照）。
pub fn is_known(name: &str) -> bool {
    KNOWN.contains(&name)
}

/// フック実行の結果。`Ran` のときだけ onchange ハッシュを保存する。
enum Outcome {
    /// 効果が起きた（または既に望ましい状態）。ハッシュを保存し、次回は onchange で skip する。
    Ran,
    /// 前提（依存ツール等）が揃わず未実行。**ハッシュは保存しない**ので、前提が整えば次回再実行される。
    Skipped(String),
}

/// 1 フックを onchange gate を通して実行する（[`crate::apply`] が配置成功後に呼ぶ）。
///
/// `unit_dir` はソースハッシュの対象（ユニットのソースツリー）、`unit_rel` は状態キーと表示名、
/// `home` はアクションのパス基点。ハッシュが前回と同じなら実行せず skip を表示して返す。
pub fn run_if_changed(
    hook: &str,
    unit_dir: &Path,
    unit_rel: &str,
    home: &Path,
    state: &mut State,
) -> Result<(), String> {
    let hash = onchange::hash_dir(unit_dir)?;
    let key = format!("{unit_rel}::{hook}");

    if state.get(&key) == Some(hash.as_str()) {
        println!("hook: {hook} ({unit_rel}) → skip (ソース不変)");
        return Ok(());
    }

    match dispatch(hook, home)? {
        Outcome::Ran => {
            // 実行できたときだけ前回ハッシュを更新する（途中失敗時の取りこぼし防止に逐次保存）。
            state.set(&key, &hash);
            state.save()?;
            println!("hook: {hook} ({unit_rel}) → ran");
        }
        Outcome::Skipped(reason) => {
            // ハッシュを保存しない＝前提が整えば次回 apply で再実行される。
            println!("hook: {hook} ({unit_rel}) → skip ({reason})");
        }
    }
    Ok(())
}

/// フック名から組み込みアクションへ振り分ける。未知名は [`crate::manifest`] が load 時に弾くので
/// 通常ここには来ないが、防御的にエラーを返す。
fn dispatch(hook: &str, home: &Path) -> Result<Outcome, String> {
    match hook {
        "bat-cache" => bat_cache(),
        "ghostty-macos-symlink" => ghostty_macos_symlink(home),
        other => Err(format!("未知の hook `{other}`")),
    }
}

/// `bat cache --build`: カスタムテーマを bat/delta に登録する。`bat` が PATH に無ければ
/// `Skipped`（ハッシュ未保存）にして、後で bat を入れたときに再実行されるようにする。
fn bat_cache() -> Result<Outcome, String> {
    if gate::which("bat").is_none() {
        return Ok(Outcome::Skipped("bat が PATH にない".to_string()));
    }
    // 進捗は stdout/stderr に出るが捨てる（失敗時のみ run_cmd が stderr を添えてエラーにする）。
    generate::run_cmd(&[
        "bat".to_string(),
        "cache".to_string(),
        "--build".to_string(),
    ])?;
    Ok(Outcome::Ran)
}

/// ghostty の macOS 参照先（`~/Library/Application Support/com.mitchellh.ghostty/config`）から
/// `~/.config/ghostty/config` への symlink を冪等に張る（chezmoi `rm -f` ＋ `ln -sf` の移植）。
///
/// 既に正しい symlink なら何もしない。異なる実体（誤ったリンク・実ファイル）があれば
/// [`std::fs::remove_file`] で除去してから張り直す ― これはツールが所有する参照リンクの更新で
/// あり、ユーザーデータの削除ではない。ghostty ユニットは `os = "darwin"` で gate されるため
/// macOS でのみ到達する。
#[cfg(unix)]
fn ghostty_macos_symlink(home: &Path) -> Result<Outcome, String> {
    let support_dir = home.join("Library/Application Support/com.mitchellh.ghostty");
    let link = support_dir.join("config");
    let source = home.join(".config/ghostty/config");

    std::fs::create_dir_all(&support_dir)
        .map_err(|e| format!("{}: ディレクトリ作成失敗: {e}", support_dir.display()))?;

    // 既に正しい symlink なら no-op（chezmoi の早期 return 相当）。
    if std::fs::read_link(&link).is_ok_and(|t| t == source) {
        return Ok(Outcome::Ran);
    }
    // 誤ったリンク・実ファイル（dangling symlink 含む）があれば除去してから張り直す。
    if std::fs::symlink_metadata(&link).is_ok() {
        std::fs::remove_file(&link)
            .map_err(|e| format!("{}: 既存エントリ除去失敗: {e}", link.display()))?;
    }
    std::os::unix::fs::symlink(&source, &link)
        .map_err(|e| format!("{}: symlink 作成失敗: {e}", link.display()))?;
    Ok(Outcome::Ran)
}

/// 非 unix では symlink 機構が異なるため未対応（ghostty ユニットは darwin gate なので実際は来ない）。
#[cfg(not(unix))]
fn ghostty_macos_symlink(_home: &Path) -> Result<Outcome, String> {
    Ok(Outcome::Skipped("unix 以外は未対応".to_string()))
}
