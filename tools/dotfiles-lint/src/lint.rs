//! dotfiles の lint オーケストレータ。旧 `nix/lint`（Python）の移植。
//!
//! ファイルを分類し、種別ごとに外部フォーマッタ/リンタを呼ぶ。
//! ツール群（shfmt / shellcheck / taplo / stylua / rumdl / ruff / fish）は
//! mise で供給される前提で、PATH 上の名前で実行する。

mod classify;
mod collect;
mod exec;
mod report;
mod rules;

pub use collect::{collect_files, find_repo_root};

use std::path::PathBuf;

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
    verbose: bool,
    pub failures: Vec<Failure>,
}

impl Orchestrator {
    pub fn new(repo_root: PathBuf, verbose: bool) -> Self {
        Self {
            repo_root,
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
        if rules::match_shell(f) && self.fix_shell(f) != 0 {
            failed = true;
        }
        if classify::is_fish(&f.rel_path) && self.fix_fish(f) != 0 {
            failed = true;
        }
        if classify::is_lua(&f.rel_path) && self.fix_lua(f) != 0 {
            failed = true;
        }
        if rules::match_python(f) && self.fix_python(f) != 0 {
            failed = true;
        }
        if classify::is_toml(&f.rel_path) && self.fix_toml(f) != 0 {
            failed = true;
        }
        if rules::match_markdown(f) && self.fix_markdown(f) != 0 {
            failed = true;
        }
        failed
    }

    fn check_file(&mut self, f: &FileContext) -> bool {
        let mut failed = false;
        if rules::match_shell(f) && self.check_shell(f) != 0 {
            failed = true;
        }
        if classify::is_fish(&f.rel_path) && self.check_fish(f) != 0 {
            failed = true;
        }
        if classify::is_lua(&f.rel_path) && self.check_lua(f) != 0 {
            failed = true;
        }
        if rules::match_python(f) && self.check_python(f) != 0 {
            failed = true;
        }
        if classify::is_toml(&f.rel_path) && self.check_toml(f) != 0 {
            failed = true;
        }
        if rules::match_markdown(f) && self.check_markdown(f) != 0 {
            failed = true;
        }
        if rules::match_markdown(f) && self.check_textlint(f) != 0 {
            failed = true;
        }
        failed
    }
}
