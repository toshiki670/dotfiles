use super::super::{FileContext, Orchestrator};
use crate::lint::exec::abs;

impl Orchestrator {
    pub(crate) fn fix_lua(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(f, "lua", "fix", &["stylua", &abs], None)
    }

    pub(crate) fn check_lua(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(f, "lua", "check", &["stylua", "--check", &abs], None)
    }
}
