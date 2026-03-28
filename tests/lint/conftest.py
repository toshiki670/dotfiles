import shutil
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
NIX_DIR = REPO_ROOT / "nix"
if str(NIX_DIR) not in sys.path:
    sys.path.insert(0, str(NIX_DIR))

# Same binaries as the lint runner; Nix `lint-tests` puts them on PATH.
LINT_TOOLCHAIN_BINARIES = (
    "shfmt",
    "shellcheck",
    "fish_indent",
    "fish",
    "zsh",
    "stylua",
    "ruff",
    "taplo",
    "markdownlint-cli2",
)


@pytest.fixture(scope="session")
def lint_toolchain() -> None:
    missing = [name for name in LINT_TOOLCHAIN_BINARIES if shutil.which(name) is None]
    if missing:
        pytest.skip(
            f"lint toolchain not on PATH (need Nix lint-tests app); missing: {missing}"
        )
