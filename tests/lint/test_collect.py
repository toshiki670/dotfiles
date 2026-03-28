from pathlib import Path

from lint.collect import collect_files


def test_collect_files_respects_gitignore(tmp_path: Path) -> None:
    (tmp_path / "flake.nix").write_text("{}", encoding="utf-8")
    (tmp_path / ".gitignore").write_text(
        "ignored.txt\nignored_dir/\n", encoding="utf-8"
    )
    (tmp_path / "keep.txt").write_text("ok\n", encoding="utf-8")
    (tmp_path / "ignored.txt").write_text("ng\n", encoding="utf-8")
    (tmp_path / "ignored_dir").mkdir()
    (tmp_path / "ignored_dir" / "x.txt").write_text("x\n", encoding="utf-8")

    files = sorted(fc.rel_path for fc in collect_files(tmp_path))
    assert "keep.txt" in files
    assert "ignored.txt" not in files
    assert "ignored_dir/x.txt" not in files


def test_collect_files_skips_cursor_directory(tmp_path: Path) -> None:
    (tmp_path / "flake.nix").write_text("{}", encoding="utf-8")
    (tmp_path / ".cursor" / "plans").mkdir(parents=True)
    (tmp_path / ".cursor" / "plans" / "note.md").write_text("# x\n", encoding="utf-8")
    (tmp_path / "keep.md").write_text("ok\n", encoding="utf-8")

    files = sorted(fc.rel_path for fc in collect_files(tmp_path))
    assert "keep.md" in files
    assert ".cursor/plans/note.md" not in files
