//! ファイルの分類判定。旧 `nix/lint/classify.py` の移植。
//!
//! 拡張子・パスベースの判定はファイル名（リポジトリ相対）に対して行い、
//! 内容ベースの判定（shebang）は実ファイルを読む。

use std::path::Path;

/// ファイルの先頭行を返す（読めなければ空文字）。
fn first_line(path: &Path) -> String {
    std::fs::read(path)
        .ok()
        .map(|bytes| {
            let text = String::from_utf8_lossy(&bytes);
            text.lines().next().unwrap_or("").to_string()
        })
        .unwrap_or_default()
}

pub fn is_markdown(f: &str) -> bool {
    f.ends_with(".md")
}

/// 自動生成の CHANGELOG.md か（root・各クレート直下の per-package とも対象）。
pub fn is_changelog(f: &str) -> bool {
    f == "CHANGELOG.md" || f.ends_with("/CHANGELOG.md")
}

pub fn is_lua(f: &str) -> bool {
    f.ends_with(".lua")
}

pub fn is_toml(f: &str) -> bool {
    f.ends_with(".toml")
}

pub fn is_python(f: &str) -> bool {
    f.ends_with(".py")
}

pub fn is_fish(f: &str) -> bool {
    f.ends_with(".fish")
}

pub fn is_shell_ext(f: &str) -> bool {
    f.ends_with(".sh")
}

pub fn is_shell_path(f: &str) -> bool {
    f.starts_with("bin/") || f.starts_with("bash/")
}

/// 先頭行が python の shebang か。
pub fn has_python_shebang(path: &Path) -> bool {
    let first = first_line(path);
    first.starts_with("#!") && first.contains("python")
}

/// shebang から shell 種別を判定する（"bash" / "sh" / ""）。
pub fn shell_flavor(path: &Path) -> String {
    let first = first_line(path);
    if first.contains("bash") {
        "bash".to_string()
    } else if first.starts_with("#!") && first.ends_with("sh") {
        "sh".to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_predicates() {
        assert!(is_markdown("README.md"));
        assert!(is_changelog("CHANGELOG.md"));
        assert!(is_changelog("crates/gh-clone/CHANGELOG.md"));
        assert!(!is_changelog("README.md"));
        assert!(!is_changelog("docs/CHANGELOG.md.bak"));
        assert!(is_lua("a.lua"));
        assert!(is_toml("Cargo.toml"));
        assert!(is_python("x.py"));
        assert!(is_fish("f.fish"));
        assert!(is_shell_ext("s.sh"));
    }

    #[test]
    fn shell_path_predicate() {
        assert!(is_shell_path("bin/foo"));
        assert!(is_shell_path("bash/bar"));
        assert!(!is_shell_path("home/bin/foo"));
    }

    #[test]
    fn content_predicates() {
        let dir = tempfile::tempdir().unwrap();
        let sh_bash = dir.path().join("x.sh");
        std::fs::write(&sh_bash, "#!/bin/bash\necho hi\n").unwrap();
        assert_eq!(shell_flavor(&sh_bash), "bash");

        let py = dir.path().join("p");
        std::fs::write(&py, "#!/usr/bin/env python3\nprint(1)\n").unwrap();
        assert!(has_python_shebang(&py));
        assert_eq!(shell_flavor(&py), String::new());

        let sh = dir.path().join("s");
        std::fs::write(&sh, "#!/bin/sh\necho hi\n").unwrap();
        assert_eq!(shell_flavor(&sh), "sh");
        assert!(!has_python_shebang(&sh));
    }
}
