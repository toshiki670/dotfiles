//! シェルファイルの lint 規則（shfmt でフォーマット、shellcheck で静的解析）。

use super::super::{FileContext, Orchestrator};
use crate::lint::exec::abs;

impl Orchestrator {
    pub(crate) fn fix_shell(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(
            f,
            "shell",
            "fix",
            &["shfmt", "-w", "-i", "2", "-ci", &abs],
            None,
        )
    }

    pub(crate) fn check_shell(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        let mut failed = 0;
        if self.run_rule_cmd(
            f,
            "shell",
            "check",
            &["shfmt", "-d", "-i", "2", "-ci", &abs],
            None,
        ) != 0
        {
            failed = 1;
        }
        if self.run_rule_cmd(f, "shell", "check", &["shellcheck", &abs], None) != 0 {
            failed = 1;
        }
        failed
    }
}
