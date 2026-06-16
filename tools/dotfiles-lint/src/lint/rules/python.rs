use super::super::{FileContext, Orchestrator};
use crate::lint::exec::abs;

impl Orchestrator {
    pub(crate) fn fix_python(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(f, "python", "fix", &["ruff", "format", &abs], None, false)
    }

    pub(crate) fn check_python(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(
            f,
            "python",
            "check",
            &["ruff", "format", "--check", &abs],
            None,
            false,
        )
    }
}
