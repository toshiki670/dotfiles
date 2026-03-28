import os
import subprocess
import sys
from pathlib import Path


def _nix_dir() -> Path:
    return Path(__file__).resolve().parents[2] / "nix"


def test_cli_module_invokes_main_on_fix(tmp_path: Path) -> None:
    """Regression: `python -m lint.cli` must execute main(), not exit silently."""
    (tmp_path / "flake.nix").write_text("{}", encoding="utf-8")
    env = os.environ.copy()
    env["PYTHONPATH"] = str(_nix_dir())
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "lint.cli",
            "fix",
            "--check-after-fix",
            "off",
        ],
        cwd=tmp_path,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, (result.stdout, result.stderr)
    assert "lint(fix): completed" in result.stdout
