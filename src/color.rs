//! `dotfiles color`（§10 / §11）: テーマの手動固定／追従切替（[`set`]）と ANSI カラー確認表
//! （[`sample`]）。
//!
//! [`set`] は `dark` / `light` / `auto` を状態ファイル（[`crate::theme`]）へ書き、新しい状態で
//! apply を再実行して `when.theme` overlay を反映し（Ghostty config の theme 行を build-time で
//! 固定）、最後に `theme = "source"` ユニットの hooks を強制発火して端末へ反映させる（§10.2.1）。
//! 起点（Ghostty）の背景＋ANSI が固定されれば、follower（eza/delta）も self（bat/nvim/fzf）も
//! 既存の追従機構のまま自然に反映される ― 個別の override 変数は新設しない。
//!
//! [`sample`] は旧 `crates/color`（さらに遡れば `bin/color`（Python））の責務を吸収したもので、
//! 端末で色を確認するための表を出力する（入力は取らない）。
//!
//! 参考:
//! - <http://ascii-table.com/ansi-escape-sequences.php>
//! - <http://archive.linux.or.jp/JF/JFdocs/Bash-Prompt-HOWTO-5.html>

use crate::discover::{self, MANIFEST};
use crate::manifest::{Manifest, Theme, ThemeRole};
use crate::{apply, gate, hooks, theme};
use std::path::Path;

/// テーマを `requested`（dark / light / auto）へ切り替える（§10.2）。
///
/// 手順:
/// 1. 状態ファイル（[`crate::theme::set`]）へ書く。以後の apply / gate はこの状態を読む。
/// 2. 新しい状態で apply を再実行（[`crate::apply::run`]）し、`when.theme` overlay に該当 fragment を
///    選ばせる（Ghostty config の theme 行が固定／追従へ書き換わる）。
/// 3. `theme = "source"` ユニットの hooks を強制発火（[`reload_sources`]）して端末へ反映する。
///    状態だけ変えソースは不変なので、通常の onchange gate では reload hook が skip されるため。
///
/// `source` は apply と同じ固定ソース（cwd 相対 `configs/`）。汎用化（embedded / `--source`）は S8。
pub fn set(source: &Path, home: &Path, requested: Theme) -> Result<(), String> {
    theme::set(home, requested)?;
    println!("color: テーマを {requested} に設定しました");
    apply::run(source, home)?;
    reload_sources(source, requested)?;
    Ok(())
}

/// `theme = "source"`（端末背景＋ANSI の起点）かつ unit gate を通ったユニットの hooks を強制実行し、
/// テーマ切替を端末へ反映する（Ghostty の SIGUSR2 reload 等）。gate off のユニットは config を
/// 書いていないので reload しても無意味なため除外する。reload コマンド自体は manifest の `hooks`
/// にデータとして宣言され、エンジンはツール名（ghostty 等）を一切持たない（§10.2.1・D9）。
fn reload_sources(source: &Path, theme: Theme) -> Result<(), String> {
    for unit in discover::collect(source)? {
        let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
        if manifest.theme == Some(ThemeRole::Source)
            && gate::unit_skip_reason(&manifest, theme).is_none()
        {
            let name = unit.rel.to_string_lossy();
            hooks::run_unit_hooks_forced(&unit.dir, &name, &manifest.hooks)?;
        }
    }
    Ok(())
}

/// 全シーケンスを既定へ戻す ANSI リセット。
const RESET: &str = "\x1b[0m";

/// (エスケープコード, 表示名) の 16 色定義。
const COLORS: &[(&str, &str)] = &[
    ("1;37", "White       "),
    ("37", "Light Gray  "),
    ("1;30", "Gray        "),
    ("30", "Black       "),
    ("31", "Red         "),
    ("1;31", "Light Red   "),
    ("32", "Green       "),
    ("1;32", "Light Green "),
    ("33", "Brown       "),
    ("1;33", "Yellow      "),
    ("34", "Blue        "),
    ("1;34", "Light Blue  "),
    ("35", "Purple      "),
    ("1;35", "Pink        "),
    ("36", "Cyan        "),
    ("1;36", "Light Cyan  "),
];

/// 16 色（前景 × 背景）と 256 色の確認表を stdout へ出力する。
pub fn sample() {
    println!("=== 16 Colors ===");
    println!(" On White(47)     On Black(40)     On Default     Color Code");

    for (code, name) in COLORS {
        let on_white = format!("\x1b[47m\x1b[{code}m  {name}{RESET}");
        let on_black = format!("\x1b[40m\x1b[{code}m  {name}{RESET}");
        let on_default = format!("\x1b[{code}m  {name}{RESET}");
        println!("{on_white}  {on_black}  {on_default}  {code}");
    }

    println!();
    println!("=== 256 Colors ===");

    let mut line = String::new();
    for code in 0..256 {
        line.push_str(&format!("\x1b[48;5;{code}m\x1b[38;5;0m {code:03} \x1b[0m"));
        if (code + 1) % 16 == 0 {
            println!("{line}{RESET}");
            line.clear();
        }
    }
    print!("{RESET}");
}
