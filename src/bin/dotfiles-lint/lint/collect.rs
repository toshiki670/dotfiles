//! リポジトリのファイル収集。旧 `nix/lint/collect.py` の移植。
//!
//! `.gitignore` を尊重し、`.git` / `.cursor` を除外する。収集は `ignore`
//! クレート（ripgrep と同じ実装）に任せる。

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use super::FileContext;

/// `Cargo.toml` を上方向に探してリポジトリルートを決める。
///
/// 旧実装は `flake.nix` を目印にしていたが、Nix 撤去後も安定する
/// `Cargo.toml`（version の単一 SoT）に変更した。
pub fn find_repo_root(start: &Path) -> PathBuf {
    let canonical = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let mut cur = canonical.as_path();
    loop {
        if cur.join("Cargo.toml").is_file() {
            return cur.to_path_buf();
        }
        match cur.parent() {
            Some(parent) => cur = parent,
            None => return canonical.clone(),
        }
    }
}

/// リポジトリ配下のファイルを `.gitignore` 準拠で収集する。
pub fn collect_files(repo_root: &Path) -> Vec<FileContext> {
    let mut out = Vec::new();
    let walker = WalkBuilder::new(repo_root)
        .hidden(false) // .github/ や .markdownlint 系の dotfile も対象
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .parents(false)
        .filter_entry(|entry| {
            let name = entry.file_name();
            name != ".git" && name != ".cursor"
        })
        .build();

    for result in walker {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let abs = entry.path();
        let rel = match abs.strip_prefix(repo_root) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        out.push(FileContext {
            rel_path: rel,
            abs_path: abs.to_path_buf(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn collect_respects_gitignore_and_skips_special_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // .gitignore は git リポジトリ内でのみ効くため init する。
        Command::new("git")
            .arg("init")
            .arg("-q")
            .current_dir(root)
            .status()
            .unwrap();

        fs::write(root.join(".gitignore"), "build/\n*.log\n").unwrap();
        fs::write(root.join("a.md"), "# a\n").unwrap();
        fs::write(root.join("y.log"), "noise\n").unwrap();
        fs::create_dir_all(root.join("build")).unwrap();
        fs::write(root.join("build/x.toml"), "k = 1\n").unwrap();
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("sub/b.fish"), "echo hi\n").unwrap();
        fs::create_dir_all(root.join(".cursor")).unwrap();
        fs::write(root.join(".cursor/z.md"), "# z\n").unwrap();

        let mut got: Vec<String> = collect_files(root)
            .into_iter()
            .map(|f| f.rel_path)
            .collect();
        got.sort();

        assert!(got.contains(&"a.md".to_string()));
        assert!(got.contains(&"sub/b.fish".to_string()));
        assert!(got.contains(&".gitignore".to_string()));
        assert!(
            !got.iter().any(|p| p.starts_with("build/")),
            "build/ ignored"
        );
        assert!(!got.contains(&"y.log".to_string()), "*.log ignored");
        assert!(
            !got.iter().any(|p| p.starts_with(".cursor")),
            ".cursor skipped"
        );
        assert!(!got.iter().any(|p| p.starts_with(".git/")), ".git skipped");
    }
}
