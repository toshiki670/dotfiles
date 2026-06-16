//! chezmoi テンプレート展開。

use std::path::PathBuf;
use std::process::Command;

use super::Orchestrator;

impl Orchestrator {
    /// chezmoi テンプレートを展開して一時ファイルに書き出す。
    pub(super) fn render_template(&self, rel_path: &str, ext: &str) -> Option<PathBuf> {
        let source_dir = self.repo_root.join("home");
        let src = self.repo_root.join(rel_path);
        let out_name = format!("rendered_{}{}", rel_path.replace(['/', '.'], "_"), ext);
        let out = self.tmp_dir.join(out_name);

        let output = Command::new("chezmoi")
            .arg("-S")
            .arg(&source_dir)
            .arg("execute-template")
            .arg("-f")
            .arg(&src)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                if std::fs::write(&out, &o.stdout).is_err() {
                    return None;
                }
                Some(out)
            }
            Ok(o) => {
                eprintln!("lint: chezmoi execute-template failed: {rel_path}");
                let stderr = String::from_utf8_lossy(&o.stderr);
                if !stderr.trim().is_empty() {
                    eprintln!("{}", stderr.trim_end());
                }
                None
            }
            Err(_) => {
                eprintln!("lint: chezmoi execute-template failed: {rel_path}");
                None
            }
        }
    }
}
