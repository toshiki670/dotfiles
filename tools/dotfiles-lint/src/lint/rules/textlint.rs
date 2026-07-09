//! prose 文書の lint 規則（textlint を呼ぶ）。
//!
//! 対象は preset-ja-technical-writing・preset-ai-writing・ja-no-weak-phrase
//! など、文章の形式的特徴（文長・助詞連続・AI 文体の特徴等）を見るルールに限る。
//! 語彙が特定の内部設計方針を参照しているか等、文脈・出自の判断が要るものは
//! 静的なパターンマッチでは原理的に代替できない（prh ベースの語彙禁止を試みたが、
//! 用語自体は他の文脈でも一般的に使われうるため誤検知が避けられず撤去した）。
//! そうした判断は prose-tidy skill 側（AI の文脈読解）に委ねる。
//!
//! prose の自動修正は意味を壊しうるため check のみで、fix は提供しない。
//!
//! package.json は持たない。`npx -p` の複数指定で textlint 本体とルールパッケージを
//! 同じ一時環境に揃え、その場だけで解決させる（実機検証済み）。mise の npm backend は
//! パッケージごとに隔離されたディレクトリへ入れるため、複数パッケージの相互解決ができない。

use super::super::{FileContext, Orchestrator};

/// textlint 本体とルールパッケージ（`npx -p` へ渡す一時環境の中身）。
/// 版を上げるときはここだけ直せばよい。
const TEXTLINT_PACKAGES: &[&str] = &[
    "textlint@15.7.1",
    "textlint-rule-preset-ja-technical-writing@12.0.2",
    "textlint-rule-preset-ai-writing@1.1.0",
    "textlint-rule-ja-no-weak-phrase@2.0.0",
];

impl Orchestrator {
    pub(crate) fn check_textlint(&mut self, f: &FileContext) -> i32 {
        let root = self.repo_root.clone();
        let rel = f.rel_path.clone();

        let mut cmd: Vec<&str> = vec!["npx", "-y"];
        for pkg in TEXTLINT_PACKAGES {
            cmd.push("-p");
            cmd.push(pkg);
        }
        cmd.push("textlint");
        cmd.push(&rel);

        self.run_rule_cmd(f, "textlint", "check", &cmd, Some(&root))
    }
}
