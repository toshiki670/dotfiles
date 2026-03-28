import os
from pathlib import Path

import pathspec

from .models import FileContext


def find_repo_root(start: Path) -> Path:
    cur = start.resolve()
    while True:
        if (cur / "flake.nix").is_file():
            return cur
        if cur.parent == cur:
            return start.resolve()
        cur = cur.parent


def collect_files(repo_root: Path) -> list[FileContext]:
    gitignore = repo_root / ".gitignore"
    patterns: list[str] = []
    if gitignore.is_file():
        patterns = gitignore.read_text(encoding="utf-8", errors="ignore").splitlines()
    spec = pathspec.PathSpec.from_lines("gitwildmatch", patterns)
    out: list[FileContext] = []

    for root, dirs, filenames in os.walk(repo_root):
        root_path = Path(root)
        rel_root = root_path.relative_to(repo_root).as_posix()
        if rel_root == ".":
            rel_root = ""

        kept_dirs: list[str] = []
        for d in dirs:
            if d in (".git", ".cursor"):
                continue
            rel_dir = f"{rel_root}/{d}" if rel_root else d
            if spec.match_file(rel_dir + "/"):
                continue
            kept_dirs.append(d)
        dirs[:] = kept_dirs

        for name in filenames:
            rel_file = f"{rel_root}/{name}" if rel_root else name
            if spec.match_file(rel_file):
                continue
            out.append(FileContext(rel_path=rel_file, abs_path=repo_root / rel_file))
    return out
