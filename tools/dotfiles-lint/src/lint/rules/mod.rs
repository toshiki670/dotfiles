//! 言語別ルール。

mod fish;
mod lua;
mod markdown;
mod python;
mod shell;
mod textlint;
mod toml;

use super::{FileContext, classify};

pub(super) fn match_shell(f: &FileContext) -> bool {
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

pub(super) fn match_python(f: &FileContext) -> bool {
    classify::is_python(&f.rel_path) || classify::has_python_shebang(&f.abs_path)
}

/// CHANGELOG.md は自動生成のため除外する（root・各クレート直下の per-package とも）。
pub(super) fn match_markdown(f: &FileContext) -> bool {
    classify::is_markdown(&f.rel_path) && !classify::is_changelog(&f.rel_path)
}
