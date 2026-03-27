import shlex
import subprocess
import sys
from pathlib import Path

from .models import FailureRecord, FileContext, LintContext


def format_cmd(cmd: list[str]) -> str:
    return " ".join(shlex.quote(part) for part in cmd)


def run(cmd: list[str], cwd: Path | None = None) -> int:
    return subprocess.run(cmd, cwd=cwd, check=False).returncode


def run_capture(cmd: list[str], cwd: Path | None = None) -> tuple[int, str, str]:
    p = subprocess.run(cmd, cwd=cwd, check=False, text=True, capture_output=True)
    return p.returncode, p.stdout, p.stderr


def run_rule_cmd(
    ctx: LintContext,
    f: FileContext,
    rule_name: str,
    phase: str,
    cmd: list[str],
    cwd: Path | None = None,
    template_shellcheck_hint: bool = False,
) -> int:
    if ctx.verbose:
        where = f" (cwd={cwd})" if cwd else ""
        print(f"[{phase}:{rule_name}] {f.rel_path}: {format_cmd(cmd)}{where}")
    code = run(cmd, cwd=cwd)
    if code != 0:
        if template_shellcheck_hint:
            print(
                f"lint: shellcheck failed on expanded template (source: {f.rel_path})",
                file=sys.stderr,
            )
        ctx.failures.append(
            FailureRecord(
                file=f.rel_path,
                rule=rule_name,
                phase=phase,
                command=format_cmd(cmd),
                exit_code=code,
            )
        )
    return code
