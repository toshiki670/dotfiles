use super::super::{FileContext, Orchestrator};
use crate::lint::exec::{abs, capture, format_cmd};

impl Orchestrator {
    pub(crate) fn fix_fish(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        let cmd = ["fish_indent", abs.as_str()];
        if self.verbose {
            println!("[fix:fish] {}: {}", f.rel_path, format_cmd(&cmd));
        }
        match capture(&cmd, None) {
            Some((0, formatted, _)) => {
                let current = std::fs::read(&f.abs_path)
                    .map(|b| String::from_utf8_lossy(&b).into_owned())
                    .unwrap_or_default();
                if formatted != current {
                    let _ = std::fs::write(&f.abs_path, formatted.as_bytes());
                }
                0
            }
            Some((code, _, _)) => {
                self.record(f, "fish", "fix", &cmd, code);
                1
            }
            None => {
                self.record(f, "fish", "fix", &cmd, 1);
                1
            }
        }
    }

    pub(crate) fn check_fish(&mut self, f: &FileContext) -> i32 {
        let target = abs(f);
        let mut failed = 0;
        if self.run_rule_cmd(
            f,
            "fish",
            "check",
            &["fish_indent", "--check", &target],
            None,
        ) != 0
        {
            failed = 1;
        }
        if self.run_rule_cmd(f, "fish", "check", &["fish", "--no-execute", &target], None) != 0 {
            failed = 1;
        }
        failed
    }
}
