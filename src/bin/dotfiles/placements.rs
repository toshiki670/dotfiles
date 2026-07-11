//! 期待配置集合（全 manifest が宣言する path output の総和）の導出。
//!
//! [`crate::doctor`] のユニット間 output 衝突検出（#593）が使う。#521（prune）が必要とする
//! 「dotfiles がどのパスへ書くと宣言しているか」の集合も、本モジュールの [`expected`] をそのまま
//! 再利用できる（gate 評価後の絞り込みは呼び出し側の #521 が担う）。
//!
//! 導出は `when`（gate）を一切評価しない、宣言だけを見た計算にする。gate は環境（`deps` / `os`）や
//! user 状態（`profile`）で変わるため、評価してから集合を作ると「いまの環境では起きない衝突」を
//! 見落とす。衝突は配線の誤りであって環境依存の事象ではないため、宣言ベースが正しい。
//!
//! ツリー配置（`input = "."`）は [`crate::discover::unit_files`] で配下ファイルへ展開してから
//! output パスと結合する ― `~/.config/fish/conf.d` や `~/.config/fish/functions` は複数ユニットが
//! 共有する合流点であり、ディレクトリ単位で比較すると別ファイルを置き合う正当な合流まで衝突として
//! 報告してしまう。cmd output はファイルシステムに痕跡を残さないため対象外（∅）。比較は解決済み
//! パスの完全一致だけなので、ツリーが `~/.config/foo/` 配下へ展開する一方で別ユニットが
//! `~/.config/foo` というファイルへ output する「ディレクトリ×ファイルの前置衝突」も対象外
//! （#593 が定義する衝突の範囲外として意図的に外す）。

use crate::discover::{self, MANIFEST};
use crate::manifest::{Manifest, StepSource, resolve_output_path};
use std::path::{Path, PathBuf};

/// 1 つの宣言された配置先。
pub struct Placement {
    /// 宣言したユニット名（`source` 相対、表示用）。
    pub unit: String,
    /// 解決済みの配置先パス（`home` を基点に `~` 展開済み）。
    pub path: PathBuf,
}

/// `source` 配下の全 manifest から、宣言された path output を集める（gate 評価なし）。
pub fn expected(source: &Path, home: &Path) -> Result<Vec<Placement>, String> {
    let units = discover::collect(source)?;
    let mut placements = Vec::new();
    for unit in &units {
        let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
        let name = unit.rel.to_string_lossy().into_owned();
        if manifest.is_tree() {
            push_tree_placements(&unit.dir, &name, &manifest, home, &mut placements)?;
        } else {
            for step in &manifest.steps {
                if let Some(StepSource::Path(p)) = &step.output {
                    placements.push(Placement {
                        unit: name.clone(),
                        path: resolve_output_path(home, p),
                    });
                }
            }
        }
    }
    Ok(placements)
}

/// ツリーユニット（`input = "."`）を配下ファイルへ展開し、それぞれの配置先を積む。
/// ツリーの output パスは [`Manifest::display_dst`] が返す唯一のパス表記（`validate` 済みなら
/// 常にパス output 1 つ）をそのまま使う。
fn push_tree_placements(
    unit_dir: &Path,
    name: &str,
    manifest: &Manifest,
    home: &Path,
    out: &mut Vec<Placement>,
) -> Result<(), String> {
    let dst_root = resolve_output_path(home, &manifest.display_dst());
    for file in discover::unit_files(unit_dir)? {
        let rel = file
            .strip_prefix(unit_dir)
            .map_err(|e| format!("{}: 相対パス算出失敗: {e}", file.display()))?;
        out.push(Placement {
            unit: name.to_string(),
            path: dst_root.join(rel),
        });
    }
    Ok(())
}
