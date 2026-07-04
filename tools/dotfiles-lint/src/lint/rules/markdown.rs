use super::super::{FileContext, Orchestrator};

impl Orchestrator {
    pub(crate) fn fix_markdown(&mut self, f: &FileContext) -> i32 {
        let root = self.repo_root.clone();
        let rel = f.rel_path.clone();
        self.run_rule_cmd(
            f,
            "markdown",
            "fix",
            &["rumdl", "check", "--fix", "--config", ".rumdl.toml", &rel],
            Some(&root),
        )
    }

    pub(crate) fn check_markdown(&mut self, f: &FileContext) -> i32 {
        let root = self.repo_root.clone();
        let rel = f.rel_path.clone();
        self.run_rule_cmd(
            f,
            "markdown",
            "check",
            &["rumdl", "check", "--config", ".rumdl.toml", &rel],
            Some(&root),
        )
    }
}
