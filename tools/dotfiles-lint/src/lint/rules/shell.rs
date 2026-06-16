use super::super::{FileContext, Orchestrator, classify};
use crate::lint::exec::abs;

impl Orchestrator {
    pub(crate) fn fix_shell(&mut self, f: &FileContext) -> i32 {
        if classify::is_home_chezmoi_shell_template(&f.rel_path, &self.repo_root) {
            return 0;
        }
        let abs = abs(f);
        self.run_rule_cmd(
            f,
            "shell",
            "fix",
            &["shfmt", "-w", "-i", "2", "-ci", &abs],
            None,
            false,
        )
    }

    pub(crate) fn check_shell(&mut self, f: &FileContext) -> i32 {
        if classify::is_home_chezmoi_shell_template(&f.rel_path, &self.repo_root) {
            let rendered = match self.render_template(&f.rel_path, ".sh") {
                Some(p) => p.to_string_lossy().into_owned(),
                None => return 1,
            };
            let mut failed = 0;
            if self.run_rule_cmd(
                f,
                "shell",
                "check",
                &["shfmt", "-d", "-i", "2", "-ci", &rendered],
                None,
                false,
            ) != 0
            {
                failed = 1;
            }
            if self.run_rule_cmd(f, "shell", "check", &["shellcheck", &rendered], None, true) != 0 {
                failed = 1;
            }
            return failed;
        }

        let abs = abs(f);
        let mut failed = 0;
        if self.run_rule_cmd(
            f,
            "shell",
            "check",
            &["shfmt", "-d", "-i", "2", "-ci", &abs],
            None,
            false,
        ) != 0
        {
            failed = 1;
        }
        if self.run_rule_cmd(f, "shell", "check", &["shellcheck", &abs], None, false) != 0 {
            failed = 1;
        }
        failed
    }
}
