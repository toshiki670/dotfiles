//! ファイルの分類判定。旧 `nix/lint/classify.py` の移植。
//!
//! 拡張子・パスベースの判定はファイル名（リポジトリ相対）に対して行い、
//! 内容ベースの判定（chezmoi マーカー・shebang）は実ファイルを読む。

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
    f.ends_with(".fish") || f.ends_with(".fish.tmpl")
}

pub fn is_shell_ext(f: &str) -> bool {
    f.ends_with(".sh") || f.ends_with(".sh.tmpl")
}

pub fn is_shell_path(f: &str) -> bool {
    f.starts_with("bin/") || f.starts_with("bash/")
}

/// ファイルに chezmoi テンプレートマーカー `{{` を含むか。
pub fn has_chezmoi_markers(path: &Path) -> bool {
    std::fs::read(path)
        .map(|bytes| String::from_utf8_lossy(&bytes).contains("{{"))
        .unwrap_or(false)
}

pub fn is_home_chezmoi_shell_template(f: &str, repo_root: &Path) -> bool {
    f.starts_with("home/") && f.ends_with(".sh.tmpl") && has_chezmoi_markers(&repo_root.join(f))
}

pub fn is_home_chezmoi_fish_template(f: &str, repo_root: &Path) -> bool {
    f.starts_with("home/") && f.ends_with(".fish.tmpl") && has_chezmoi_markers(&repo_root.join(f))
}

pub fn is_home_chezmoi_fish_completion_template(f: &str, repo_root: &Path) -> bool {
    is_home_chezmoi_fish_template(f, repo_root) && f.contains("dot_config/fish/completions/")
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
        assert!(is_lua("a.lua"));
        assert!(is_toml("Cargo.toml"));
        assert!(is_python("x.py"));
        assert!(is_fish("f.fish"));
        assert!(is_fish("f.fish.tmpl"));
        assert!(is_shell_ext("s.sh"));
        assert!(is_shell_ext("s.sh.tmpl"));
        assert!(!is_toml("starship.toml.tmpl"));
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
        let tmpl = dir.path().join("x.sh.tmpl");
        std::fs::write(&tmpl, "#!/bin/bash\necho {{ .chezmoi.os }}\n").unwrap();
        assert!(has_chezmoi_markers(&tmpl));
        assert_eq!(shell_flavor(&tmpl), "bash");

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
