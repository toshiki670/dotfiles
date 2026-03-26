#!/usr/bin/env python3
import argparse
import subprocess
import sys
import tempfile
from pathlib import Path


def run(cmd: list[str], cwd: Path | None = None) -> int:
    return subprocess.run(cmd, cwd=cwd, check=False).returncode


def run_capture(cmd: list[str], cwd: Path | None = None) -> tuple[int, str, str]:
    p = subprocess.run(cmd, cwd=cwd, check=False, text=True, capture_output=True)
    return p.returncode, p.stdout, p.stderr


def git_files(repo_root: Path) -> list[str]:
    code, out, _ = run_capture(["git", "ls-files"], cwd=repo_root)
    if code != 0:
        return []
    return [f for f in out.splitlines() if (repo_root / f).is_file()]


def has_chezmoi_markers(path: Path) -> bool:
    try:
        return "{{" in path.read_text(encoding="utf-8", errors="ignore")
    except Exception:
        return False


def is_markdown(f: str) -> bool:
    return f.endswith(".md")


def is_lua(f: str) -> bool:
    return f.endswith(".lua")


def is_toml(f: str) -> bool:
    return f.endswith(".toml")


def is_fish(f: str) -> bool:
    return f.endswith(".fish") or f.endswith(".fish.tmpl")


def is_zsh(f: str) -> bool:
    return f.endswith(".zsh") or f.endswith(".zsh.tmpl") or f == "home/dot_config/zsh/dot_zshrc.tmpl"


def is_shell_ext(f: str) -> bool:
    return f.endswith(".sh") or f.endswith(".sh.tmpl")


def is_shell_path(f: str) -> bool:
    return f.startswith("bin/") or f.startswith("bash/")


def is_home_chezmoi_shell_template(f: str, repo_root: Path) -> bool:
    return f.startswith("home/") and f.endswith(".sh.tmpl") and has_chezmoi_markers(repo_root / f)


def is_home_chezmoi_fish_template(f: str, repo_root: Path) -> bool:
    return f.startswith("home/") and f.endswith(".fish.tmpl") and has_chezmoi_markers(repo_root / f)


def is_home_chezmoi_zsh_template(f: str, repo_root: Path) -> bool:
    return (
        is_zsh(f)
        and f.startswith("home/")
        and f.endswith(".tmpl")
        and has_chezmoi_markers(repo_root / f)
    )


def shell_flavor(path: Path) -> str:
    try:
        first = path.read_text(encoding="utf-8", errors="ignore").splitlines()[0]
    except Exception:
        return ""
    if "bash" in first:
        return "bash"
    if first.startswith("#!") and first.endswith("sh"):
        return "sh"
    if "zsh" in first:
        return "zsh"
    return ""


def render_template(repo_root: Path, src_rel: str, out_path: Path) -> bool:
    source_dir = repo_root / "home"
    cmd = [
        "chezmoi",
        "-S",
        str(source_dir),
        "execute-template",
        "-f",
        str(repo_root / src_rel),
    ]
    p = subprocess.run(cmd, check=False, text=True, capture_output=True)
    if p.returncode != 0:
        print(f"lint: chezmoi execute-template failed: {src_rel}", file=sys.stderr)
        if p.stderr:
            print(p.stderr.rstrip(), file=sys.stderr)
        return False
    out_path.write_text(p.stdout, encoding="utf-8")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(usage="lint.py {fix|check}")
    parser.add_argument("mode", choices=["fix", "check"])
    args = parser.parse_args()

    code, out, _ = run_capture(["git", "rev-parse", "--show-toplevel"])
    if code != 0:
        return 1
    repo_root = Path(out.strip())

    files = git_files(repo_root)
    failed = 0

    with tempfile.TemporaryDirectory() as td:
        tmp_dir = Path(td)

        if args.mode == "fix":
            for f in files:
                abs_f = repo_root / f
                if is_shell_ext(f) or is_shell_path(f):
                    sf = shell_flavor(abs_f)
                    if sf in ("bash", "sh", "") and not is_home_chezmoi_shell_template(f, repo_root):
                        run(["shfmt", "-w", "-i", "2", "-ci", str(abs_f)])

            for f in files:
                abs_f = repo_root / f
                if is_lua(f):
                    run(["stylua", str(abs_f)])
                elif is_toml(f):
                    run(["taplo", "fmt", str(abs_f)])
                elif is_markdown(f):
                    run(["markdownlint-cli2", "--fix", f":{f}"], cwd=repo_root)
                elif is_fish(f):
                    if not is_home_chezmoi_fish_template(f, repo_root):
                        code, formatted, _ = run_capture(["fish_indent", str(abs_f)])
                        if code == 0 and formatted != abs_f.read_text(encoding="utf-8", errors="ignore"):
                            abs_f.write_text(formatted, encoding="utf-8")

            if run(["git", "diff", "--quiet", "--", *files], cwd=repo_root) == 0:
                print("lint(fix): no auto-fix changes")
            else:
                print("lint(fix): auto-fix updated files")
                run(["git", "status", "--short", "--", *files], cwd=repo_root)

        for f in files:
            abs_f = repo_root / f

            if is_shell_ext(f) or is_shell_path(f):
                sf = shell_flavor(abs_f)
                if sf in ("bash", "sh", ""):
                    if is_home_chezmoi_shell_template(f, repo_root):
                        rendered = tmp_dir / f"rendered_{f.replace('/', '_').replace('.', '_')}.sh"
                        if render_template(repo_root, f, rendered):
                            if run(["shfmt", "-d", "-i", "2", "-ci", str(rendered)]) != 0:
                                failed = 1
                            if run(["shellcheck", str(rendered)]) != 0:
                                print(
                                    f"lint: shellcheck failed on expanded template (source: {f})",
                                    file=sys.stderr,
                                )
                                failed = 1
                        else:
                            failed = 1
                    else:
                        if run(["shfmt", "-d", "-i", "2", "-ci", str(abs_f)]) != 0:
                            failed = 1
                        if run(["shellcheck", str(abs_f)]) != 0:
                            failed = 1

            if is_zsh(f):
                if is_home_chezmoi_zsh_template(f, repo_root):
                    rendered = tmp_dir / f"rendered_{f.replace('/', '_').replace('.', '_')}.zsh"
                    if render_template(repo_root, f, rendered):
                        if run(["zsh", "-n", str(rendered)]) != 0:
                            failed = 1
                    else:
                        failed = 1
                else:
                    if run(["zsh", "-n", str(abs_f)]) != 0:
                        failed = 1

            if is_fish(f):
                if is_home_chezmoi_fish_template(f, repo_root):
                    rendered = tmp_dir / f"rendered_{f.replace('/', '_').replace('.', '_')}.fish"
                    if render_template(repo_root, f, rendered):
                        if run(["fish_indent", "--check", str(rendered)]) != 0:
                            failed = 1
                        if run(["fish", "--no-execute", str(rendered)]) != 0:
                            failed = 1
                    else:
                        failed = 1
                else:
                    if run(["fish_indent", "--check", str(abs_f)]) != 0:
                        failed = 1
                    if run(["fish", "--no-execute", str(abs_f)]) != 0:
                        failed = 1

            if is_lua(f):
                if run(["stylua", "--check", str(abs_f)]) != 0:
                    failed = 1

            if is_toml(f):
                if run(["taplo", "fmt", "--check", str(abs_f)]) != 0:
                    failed = 1
                if run(["taplo", "lint", str(abs_f)]) != 0:
                    failed = 1

            if is_markdown(f):
                if run(["markdownlint-cli2", f":{f}"], cwd=repo_root) != 0:
                    failed = 1

    return failed


if __name__ == "__main__":
    sys.exit(main())
