//! 期待配置集合（全 manifest が宣言する path output の総和）の導出。
//!
//! 2 つのビューを同じ walk から出す:
//! - [`expected`]（gate 評価なし）: [`crate::doctor`] のユニット間 output 衝突検出（#593）が使う。
//!   gate は環境（`deps` / `os`）や user 状態（`profile`）で変わるため、評価してから集合を作ると
//!   「いまの環境では起きない衝突」を見落とす。衝突は配線の誤りであって環境依存の事象ではないため、
//!   宣言ベースが正しい。
//! - [`expected_gated`]（gate 評価あり）: #521（prune）の「今回の期待配置集合」。ユニット全体の
//!   `when` に加え、output step 自身の `when` も見る ― [`crate::apply::pipeline`] は output step の
//!   `when` が不成立ならその書き込みを飛ばすため、ユニット粒度だけで判定すると「apply が実際に
//!   書くもの」と食い違う。両者は同じ評価規則（[`crate::manifest::When`]）を共有するので、
//!   `expected_gated` は判定そのものを持たず、呼び出し側（[`crate::apply::gate::when_satisfied`]）
//!   から 1 つの述語として受け取る ― 本モジュールを「共有核」（[`crate`] 冒頭の依存方向）に保ち、
//!   配置エンジン（`apply` 以下）へ依存させないため。
//!
//! prune の除去候補は「前回 apply 時点の [`expected_gated`] ∖ 今回の [`expected_gated`]」（[`crate::prune`]
//! が計算する diff）で、`when` のどのキー（`deps` / `os` / `profile`）が不成立の原因かは区別しない ―
//! profile の再分類もツール未インストールによる `deps` 不成立も、「以前は置いていたが今回は置かない」
//! という点で同じ取り残し経路であり、扱いを分ける理由が無いため（安全性は削除の既定を dry-run・
//! 実削除を opt-in にすることで担保する側に置き、導出側では分岐させない）。前回分の保持は永続台帳
//! （path → owner の登録簿）ではなく、単純なパス一覧のスナップショットで足りる ― 両辺とも本モジュール
//! の導出結果（宣言由来）なので、user が独自に置いた drop-in ファイルはどちらの集合にも決して現れない。
//!
//! ツリー配置（`input = "."`）は [`crate::discover::unit_files`] で配下ファイルへ展開してから
//! output パスと結合する ― `~/.config/fish/conf.d` や `~/.config/fish/functions` は複数ユニットが
//! 共有する合流点であり、ディレクトリ単位で比較すると別ファイルを置き合う正当な合流まで衝突として
//! 報告してしまう。cmd output はファイルシステムに痕跡を残さないため対象外（∅）。比較は解決済み
//! パスの完全一致だけなので、ツリーが `~/.config/foo/` 配下へ展開する一方で別ユニットが
//! `~/.config/foo` というファイルへ output する「ディレクトリ×ファイルの前置衝突」も対象外
//! （#593 が定義する衝突の範囲外として意図的に外す）。ツリー配置（[`crate::manifest::Steps::Tree`]）
//! は step を持たず `when` も書けないため、ツリー展開そのものに step gate の判定は要らない。

use crate::discover::{self, MANIFEST};
use crate::manifest::{Manifest, Step, StepSource, Steps, When, resolve_output_path};
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
    collect(source, home, None)
}

/// gate 述語（[`When`] を満たすか）。`expected_gated` / `collect` が呼び出し側から受け取る型。
type WhenSatisfied<'a> = &'a dyn Fn(&Option<When>) -> bool;

/// `source` 配下の全 manifest から、`when_satisfied`（[`crate::apply::gate::when_satisfied`] を
/// 渡す想定）を満たす path output だけを集める。ユニット全体の `when` と、output step 自身の
/// `when` の両方に同じ述語を適用する（両者は同じ評価規則を共有するため）。
pub fn expected_gated(
    source: &Path,
    home: &Path,
    when_satisfied: WhenSatisfied,
) -> Result<Vec<Placement>, String> {
    collect(source, home, Some(when_satisfied))
}

/// [`expected`] / [`expected_gated`] が共有する walk 本体。`gate` が `None` なら宣言のみ
/// （gate 評価なし）、`Some` ならユニット gate ・ output step の step gate の両方を通す。
fn collect(
    source: &Path,
    home: &Path,
    gate: Option<WhenSatisfied>,
) -> Result<Vec<Placement>, String> {
    let units = discover::collect(source)?;
    let mut placements = Vec::new();
    for unit in &units {
        let manifest = Manifest::load(&unit.dir.join(MANIFEST))?;
        if let Some(satisfied) = gate
            && !satisfied(&manifest.when)
        {
            continue; // ユニット gate 不成立 ＝ このユニットは何も置かない。
        }
        let name = unit.rel.to_string_lossy().into_owned();
        match &manifest.steps {
            Steps::Tree { output } => {
                push_tree_placements(&unit.dir, &name, output, home, &mut placements)?;
            }
            Steps::Pipeline { steps, .. } => {
                for step in steps {
                    let Step::Output(out) = step else { continue };
                    let StepSource::Path(p) = &out.dest else {
                        continue;
                    };
                    if let Some(satisfied) = gate
                        && !satisfied(&out.when)
                    {
                        continue; // この output step 自身の gate 不成立 ＝ 今回は書かれない。
                    }
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

/// ツリーユニットを配下ファイルへ展開し、それぞれの配置先を積む。`output` はツリーの配置先
/// （`~` 起点の生表記）。
fn push_tree_placements(
    unit_dir: &Path,
    name: &str,
    output: &str,
    home: &Path,
    out: &mut Vec<Placement>,
) -> Result<(), String> {
    let dst_root = resolve_output_path(home, output);
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
