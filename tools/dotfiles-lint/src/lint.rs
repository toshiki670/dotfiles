//! dotfiles の lint オーケストレータ。旧 `nix/lint`（Python）の移植。
//!
//! ファイルを分類し、種別ごとに外部フォーマッタ/リンタを呼ぶ。chezmoi の
//! `*.sh.tmpl` / `*.fish.tmpl` は `chezmoi execute-template` で展開してから検査する。
//! ツール群（shfmt / shellcheck / taplo / stylua / rumdl / ruff / chezmoi / fish）は
//! mise で供給される前提で、PATH 上の名前で実行する。

mod classify;
mod collect;

pub use collect::{collect_files, find_repo_root};

use std::path::{Path, PathBuf};
use std::process::Command;

/// 実行モード。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Fix,
    Check,
}

/// 収集された 1 ファイル。
pub struct FileContext {
    pub rel_path: String,
    pub abs_path: PathBuf,
}

/// 失敗 1 件の記録。
pub struct Failure {
    pub file: String,
    pub rule: String,
    pub phase: String,
    pub command: String,
    pub exit_code: i32,
}

/// lint 実行の文脈と失敗記録を保持する。
pub struct Orchestrator {
    repo_root: PathBuf,
    tmp_dir: PathBuf,
    verbose: bool,
    pub failures: Vec<Failure>,
}

impl Orchestrator {
    pub fn new(repo_root: PathBuf, tmp_dir: PathBuf, verbose: bool) -> Self {
        Self {
            repo_root,
            tmp_dir,
            verbose,
            failures: Vec::new(),
        }
    }

    /// fix / check を実行する。失敗があれば true を返す。
    pub fn run(&mut self, mode: Mode, check_after_fix: bool, files: &[FileContext]) -> bool {
        let mut failed = false;

        if mode == Mode::Fix {
            for f in files {
                if self.fix_file(f) {
                    failed = true;
                }
            }
            println!("lint(fix): completed");
        }

        if mode == Mode::Check || check_after_fix {
            for f in files {
                if self.check_file(f) {
                    failed = true;
                }
            }
        }

        failed
    }

    fn fix_file(&mut self, f: &FileContext) -> bool {
        let mut failed = false;
        if self.match_shell(f) && self.fix_shell(f) != 0 {
            failed = true;
        }
        if classify::is_fish(&f.rel_path) && self.fix_fish(f) != 0 {
            failed = true;
        }
        if classify::is_lua(&f.rel_path) && self.fix_lua(f) != 0 {
            failed = true;
        }
        if self.match_python(f) && self.fix_python(f) != 0 {
            failed = true;
        }
        if classify::is_toml(&f.rel_path) && self.fix_toml(f) != 0 {
            failed = true;
        }
        if self.match_markdown(f) && self.fix_markdown(f) != 0 {
            failed = true;
        }
        failed
    }

    fn check_file(&mut self, f: &FileContext) -> bool {
        let mut failed = false;
        if self.match_shell(f) && self.check_shell(f) != 0 {
            failed = true;
        }
        if classify::is_fish(&f.rel_path) && self.check_fish(f) != 0 {
            failed = true;
        }
        if classify::is_lua(&f.rel_path) && self.check_lua(f) != 0 {
            failed = true;
        }
        if self.match_python(f) && self.check_python(f) != 0 {
            failed = true;
        }
        if classify::is_toml(&f.rel_path) && self.check_toml(f) != 0 {
            failed = true;
        }
        if self.match_markdown(f) && self.check_markdown(f) != 0 {
            failed = true;
        }
        failed
    }

    // --- shell -------------------------------------------------------------

    fn match_shell(&self, f: &FileContext) -> bool {
        if !(classify::is_shell_ext(&f.rel_path) || classify::is_shell_path(&f.rel_path)) {
            return false;
        }
        if classify::has_python_shebang(&f.abs_path) {
            return false;
        }
        matches!(
            classify::shell_flavor(&f.abs_path).as_str(),
            "bash" | "sh" | ""
        )
    }

    fn fix_shell(&mut self, f: &FileContext) -> i32 {
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

    fn check_shell(&mut self, f: &FileContext) -> i32 {
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

    // --- fish --------------------------------------------------------------

    fn fix_fish(&mut self, f: &FileContext) -> i32 {
        if classify::is_home_chezmoi_fish_template(&f.rel_path, &self.repo_root) {
            return 0;
        }
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

    fn check_fish(&mut self, f: &FileContext) -> i32 {
        let is_tmpl = classify::is_home_chezmoi_fish_template(&f.rel_path, &self.repo_root);
        let target = if is_tmpl {
            match self.render_template(&f.rel_path, ".fish") {
                Some(p) => p.to_string_lossy().into_owned(),
                None => return 1,
            }
        } else {
            abs(f)
        };

        let mut failed = 0;
        if !is_tmpl
            && !classify::is_home_chezmoi_fish_completion_template(&f.rel_path, &self.repo_root)
            && self.run_rule_cmd(
                f,
                "fish",
                "check",
                &["fish_indent", "--check", &target],
                None,
                false,
            ) != 0
        {
            failed = 1;
        }
        if self.run_rule_cmd(
            f,
            "fish",
            "check",
            &["fish", "--no-execute", &target],
            None,
            false,
        ) != 0
        {
            failed = 1;
        }
        failed
    }

    // --- lua ---------------------------------------------------------------

    fn fix_lua(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(f, "lua", "fix", &["stylua", &abs], None, false)
    }

    fn check_lua(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(f, "lua", "check", &["stylua", "--check", &abs], None, false)
    }

    // --- python (ruff: Rust 製、Python ランタイム不要) ----------------------

    fn match_python(&self, f: &FileContext) -> bool {
        classify::is_python(&f.rel_path) || classify::has_python_shebang(&f.abs_path)
    }

    fn fix_python(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(f, "python", "fix", &["ruff", "format", &abs], None, false)
    }

    fn check_python(&mut self, f: &FileContext) -> i32 {
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

    // --- toml --------------------------------------------------------------

    fn fix_toml(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        self.run_rule_cmd(f, "toml", "fix", &["taplo", "fmt", &abs], None, false)
    }

    fn check_toml(&mut self, f: &FileContext) -> i32 {
        let abs = abs(f);
        let mut failed = 0;
        if self.run_rule_cmd(
            f,
            "toml",
            "check",
            &["taplo", "fmt", "--check", &abs],
            None,
            false,
        ) != 0
        {
            failed = 1;
        }
        if self.run_rule_cmd(f, "toml", "check", &["taplo", "lint", &abs], None, false) != 0 {
            failed = 1;
        }
        failed
    }

    // --- markdown (rumdl) --------------------------------------------------

    /// CHANGELOG.md は自動生成のため除外する（root・各クレート直下の per-package
    /// とも。.rumdl.toml の exclude と一致）。
    fn match_markdown(&self, f: &FileContext) -> bool {
        classify::is_markdown(&f.rel_path) && !classify::is_changelog(&f.rel_path)
    }

    fn fix_markdown(&mut self, f: &FileContext) -> i32 {
        let root = self.repo_root.clone();
        let rel = f.rel_path.clone();
        self.run_rule_cmd(
            f,
            "markdown",
            "fix",
            &["rumdl", "check", "--fix", "--config", ".rumdl.toml", &rel],
            Some(&root),
            false,
        )
    }

    fn check_markdown(&mut self, f: &FileContext) -> i32 {
        let root = self.repo_root.clone();
        let rel = f.rel_path.clone();
        self.run_rule_cmd(
            f,
            "markdown",
            "check",
            &["rumdl", "check", "--config", ".rumdl.toml", &rel],
            Some(&root),
            false,
        )
    }

    // --- helpers -----------------------------------------------------------

    /// chezmoi テンプレートを展開して一時ファイルに書き出す。
    fn render_template(&self, rel_path: &str, ext: &str) -> Option<PathBuf> {
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

    /// ルールのコマンドを実行し、非ゼロなら失敗を記録する。
    fn run_rule_cmd(
        &mut self,
        f: &FileContext,
        rule: &str,
        phase: &str,
        cmd: &[&str],
        cwd: Option<&Path>,
        shellcheck_hint: bool,
    ) -> i32 {
        if self.verbose {
            let where_ = cwd
                .map(|c| format!(" (cwd={})", c.display()))
                .unwrap_or_default();
            println!(
                "[{phase}:{rule}] {}: {}{where_}",
                f.rel_path,
                format_cmd(cmd)
            );
        }

        let mut command = Command::new(cmd[0]);
        command.args(&cmd[1..]);
        if let Some(c) = cwd {
            command.current_dir(c);
        }
        let code = match command.status() {
            Ok(status) => status.code().unwrap_or(1),
            Err(_) => 1,
        };

        if code != 0 {
            if shellcheck_hint {
                eprintln!(
                    "lint: shellcheck failed on expanded template (source: {})",
                    f.rel_path
                );
            }
            self.record(f, rule, phase, cmd, code);
        }
        code
    }

    fn record(&mut self, f: &FileContext, rule: &str, phase: &str, cmd: &[&str], code: i32) {
        self.failures.push(Failure {
            file: f.rel_path.clone(),
            rule: rule.to_string(),
            phase: phase.to_string(),
            command: format_cmd(cmd),
            exit_code: code,
        });
    }

    pub fn print_summary(&self) {
        println!("lint: failures={}", self.failures.len());
        for r in &self.failures {
            println!(
                "- {}:{} {} -> ({}) {}",
                r.phase, r.rule, r.file, r.exit_code, r.command
            );
        }
    }

    pub fn print_json(&self, failed: bool) {
        let mut items = Vec::with_capacity(self.failures.len());
        for r in &self.failures {
            items.push(format!(
                "{{\"file\":{},\"rule\":{},\"phase\":{},\"command\":{},\"exitCode\":{}}}",
                json_str(&r.file),
                json_str(&r.rule),
                json_str(&r.phase),
                json_str(&r.command),
                r.exit_code,
            ));
        }
        println!(
            "{{\"failed\":{},\"failureCount\":{},\"failures\":[{}]}}",
            i32::from(failed),
            self.failures.len(),
            items.join(",")
        );
    }
}

/// FileContext の絶対パスを String にする。
fn abs(f: &FileContext) -> String {
    f.abs_path.to_string_lossy().into_owned()
}

/// コマンドを表示用にシェルクオートして連結する。
fn format_cmd(cmd: &[&str]) -> String {
    cmd.iter()
        .map(|part| shell_quote(part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '=' | ':'))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// コマンドを実行し (exit code, stdout, stderr) を返す。
fn capture(cmd: &[&str], cwd: Option<&Path>) -> Option<(i32, String, String)> {
    let mut command = Command::new(cmd[0]);
    command.args(&cmd[1..]);
    if let Some(c) = cwd {
        command.current_dir(c);
    }
    let output = command.output().ok()?;
    Some((
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_leaves_simple_words() {
        assert_eq!(shell_quote("shfmt"), "shfmt");
        assert_eq!(shell_quote("-i"), "-i");
        assert_eq!(shell_quote("/a/b.sh"), "/a/b.sh");
    }

    #[test]
    fn shell_quote_wraps_spaces() {
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn format_cmd_joins() {
        assert_eq!(format_cmd(&["shfmt", "-w", "x y"]), "shfmt -w 'x y'");
    }

    #[test]
    fn json_str_escapes() {
        assert_eq!(json_str("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }
}
