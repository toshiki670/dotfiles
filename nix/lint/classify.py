from pathlib import Path


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


def is_python(f: str) -> bool:
    return f.endswith(".py")


def is_fish(f: str) -> bool:
    return f.endswith(".fish") or f.endswith(".fish.tmpl")


def is_zsh(f: str) -> bool:
    return (
        f.endswith(".zsh")
        or f.endswith(".zsh.tmpl")
        or f == "home/dot_config/zsh/dot_zshrc.tmpl"
    )


def is_shell_ext(f: str) -> bool:
    return f.endswith(".sh") or f.endswith(".sh.tmpl")


def is_shell_path(f: str) -> bool:
    return f.startswith("bin/") or f.startswith("bash/")


def is_home_chezmoi_shell_template(f: str, repo_root: Path) -> bool:
    return (
        f.startswith("home/")
        and f.endswith(".sh.tmpl")
        and has_chezmoi_markers(repo_root / f)
    )


def is_home_chezmoi_fish_template(f: str, repo_root: Path) -> bool:
    return (
        f.startswith("home/")
        and f.endswith(".fish.tmpl")
        and has_chezmoi_markers(repo_root / f)
    )


def is_home_chezmoi_fish_completion_template(f: str, repo_root: Path) -> bool:
    return is_home_chezmoi_fish_template(f, repo_root) and (
        "dot_config/fish/completions/" in f
    )


def is_home_chezmoi_zsh_template(f: str, repo_root: Path) -> bool:
    return (
        is_zsh(f)
        and f.startswith("home/")
        and f.endswith(".tmpl")
        and has_chezmoi_markers(repo_root / f)
    )


def has_python_shebang(path: Path) -> bool:
    try:
        first = path.read_text(encoding="utf-8", errors="ignore").splitlines()[0]
    except Exception:
        return False
    return first.startswith("#!") and "python" in first


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
