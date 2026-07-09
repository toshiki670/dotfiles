//! prose 文書の lint 規則（textlint を呼ぶ）。
//!
//! prose の自動修正は意味を壊しうるため check のみで、fix は提供しない
//! （rumdl 等の構文フォーマッタとは異なり、prh の `expected` は人間向けの指示文であって
//! そのまま本文へ代入してよい修正案ではない）。

use super::super::{FileContext, Orchestrator};

impl Orchestrator {
    pub(crate) fn check_textlint(&mut self, f: &FileContext) -> i32 {
        let root = self.repo_root.clone();
        let rel = f.rel_path.clone();
        self.run_rule_cmd(
            f,
            "textlint",
            "check",
            &["npx", "--no-install", "textlint", &rel],
            Some(&root),
        )
    }
}
