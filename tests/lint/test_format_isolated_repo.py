"""Integration tests: format only inside tmp dirs (never the real dotfiles tree)."""

from pathlib import Path

import pytest


def _write_minimal_repo(root: Path) -> dict[str, Path]:
    """Tiny flake repo with one messy file per lint rule type."""
    (root / "flake.nix").write_text("{}\n", encoding="utf-8")
    (root / "bin").mkdir(parents=True, exist_ok=True)

    paths: dict[str, Path] = {
        "shell": root / "bin" / "messy.sh",
        "fish": root / "messy.fish",
        "zsh": root / "messy.zsh",
        "lua": root / "messy.lua",
        "python": root / "messy.py",
        "toml": root / "messy.toml",
        "markdown": root / "messy.md",
    }

    paths["shell"].write_text(
        "#!/usr/bin/env bash\n"
        "# shellcheck shell=bash\n"
        "if true; then\n"
        "echo unindented\n"
        "fi\n",
        encoding="utf-8",
    )

    paths["fish"].write_text(
        "if true\necho hi\nend\n",
        encoding="utf-8",
    )

    paths["zsh"].write_text(
        "#!/usr/bin/env zsh\n"
        "# Only syntax check applies; keep content stable but valid.\n"
        "true\n",
        encoding="utf-8",
    )

    paths["lua"].write_text(
        "local  function  f(  )\nreturn  1\nend\n",
        encoding="utf-8",
    )

    paths["python"].write_text("x=1+2\n\n\n", encoding="utf-8")

    paths["toml"].write_text(
        "[sec]\na=1\n  b  =  2\n",
        encoding="utf-8",
    )

    # MD047 / no single trailing newline (auto-fixable by markdownlint-cli2 with --fix).
    paths["markdown"].write_bytes(b"# Messy title")

    return paths


@pytest.mark.usefixtures("lint_toolchain")
def test_fix_formats_each_type_in_isolated_repo(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    paths = _write_minimal_repo(tmp_path)
    before = {key: paths[key].read_bytes() for key in paths}

    monkeypatch.chdir(tmp_path)

    from lint.collect import collect_files

    for fc in collect_files(tmp_path):
        try:
            fc.abs_path.relative_to(tmp_path)
        except ValueError as exc:
            msg = f"unexpected path outside tmp repo: {fc.abs_path}"
            raise AssertionError(msg) from exc

    from lint.cli import main

    assert main(["fix", "--check-after-fix", "on"]) == 0

    assert before["shell"] != paths["shell"].read_bytes()
    she = paths["shell"].read_bytes()
    assert b"echo unindented" in she
    assert b"  echo unindented" in she or b"\techo unindented" in she

    assert before["fish"] != paths["fish"].read_bytes()

    assert before["lua"] != paths["lua"].read_bytes()

    assert before["python"] != paths["python"].read_bytes()
    assert b"x = 1 + 2" in paths["python"].read_bytes()

    assert before["toml"] != paths["toml"].read_bytes()

    assert before["markdown"] != paths["markdown"].read_bytes()
    text = paths["markdown"].read_text(encoding="utf-8")
    assert text.endswith("\n")

    # zsh: no formatter; content unchanged while syntax check passes.
    assert paths["zsh"].read_bytes() == before["zsh"]
