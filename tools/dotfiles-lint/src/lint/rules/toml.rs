use super::super::{FileContext, Orchestrator};
use crate::lint::exec::abs;

impl Orchestrator {
    pub(crate) fn fix_toml(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(f, "toml", "fix", &["taplo", "fmt", &abs], None)
    }

    pub(crate) fn check_toml(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        let mut failed = 0;
        if self.run_rule_cmd(f, "toml", "check", &["taplo", "fmt", "--check", &abs], None) != 0 {
            failed = 1;
        }
        if self.run_rule_cmd(f, "toml", "check", &["taplo", "lint", &abs], None) != 0 {
            failed = 1;
        }
        failed
    }
}
