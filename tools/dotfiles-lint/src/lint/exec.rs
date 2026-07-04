//! コマンド実行と失敗記録。

use std::path::Path;
use std::process::Command;

use super::{Failure, FileContext, Orchestrator};

impl Orchestrator {
    /// ルールのコマンドを実行し、非ゼロなら失敗を記録する。
    pub(super) fn run_rule_cmd(
        &mut self,
        f: &FileContext,
        rule: &str,
        phase: &str,
        cmd: &[&str],
        cwd: Option<&Path>,
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
            self.record(f, rule, phase, cmd, code);
        }
        code
    }

    pub(super) fn record(
        &mut self,
        f: &FileContext,
        rule: &str,
        phase: &str,
        cmd: &[&str],
        code: i32,
    ) {
        self.failures.push(Failure {
            file: f.rel_path.clone(),
            rule: rule.to_string(),
            phase: phase.to_string(),
            command: format_cmd(cmd),
            exit_code: code,
        });
    }
}

/// FileContext の絶対パスを String にする。
pub(super) fn abs(f: &FileContext) -> String {
    f.abs_path.to_string_lossy().into_owned()
}

/// コマンドを表示用にシェルクオートして連結する。
pub(super) fn format_cmd(cmd: &[&str]) -> String {
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

/// コマンドを実行し (exit code, stdout, stderr) を返す。
pub(super) fn capture(cmd: &[&str], cwd: Option<&Path>) -> Option<(i32, String, String)> {
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
}
