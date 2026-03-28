from .classify import (
    is_fish,
    is_home_chezmoi_fish_completion_template,
    is_home_chezmoi_fish_template,
    is_home_chezmoi_shell_template,
    is_home_chezmoi_zsh_template,
    is_lua,
    is_markdown,
    is_python,
    is_shell_ext,
    is_shell_path,
    is_toml,
    is_zsh,
    shell_flavor,
)
from .models import FailureRecord, FileContext, LintContext, Rule
from .templates import get_rendered
from .utils import format_cmd, run_capture, run_rule_cmd


def match_shell(_: LintContext, f: FileContext) -> bool:
    if not (is_shell_ext(f.rel_path) or is_shell_path(f.rel_path)):
        return False
    return shell_flavor(f.abs_path) in ("bash", "sh", "")


def fix_shell(ctx: LintContext, f: FileContext) -> int:
    if is_home_chezmoi_shell_template(f.rel_path, ctx.repo_root):
        return 0
    return run_rule_cmd(
        ctx, f, "shell", "fix", ["shfmt", "-w", "-i", "2", "-ci", str(f.abs_path)]
    )


def check_shell(ctx: LintContext, f: FileContext) -> int:
    if is_home_chezmoi_shell_template(f.rel_path, ctx.repo_root):
        rendered = get_rendered(ctx, f, ".sh")
        if rendered is None:
            return 1
        failed = 0
        if (
            run_rule_cmd(
                ctx,
                f,
                "shell",
                "check",
                ["shfmt", "-d", "-i", "2", "-ci", str(rendered)],
            )
            != 0
        ):
            failed = 1
        if (
            run_rule_cmd(
                ctx,
                f,
                "shell",
                "check",
                ["shellcheck", str(rendered)],
                template_shellcheck_hint=True,
            )
            != 0
        ):
            failed = 1
        return failed

    failed = 0
    if (
        run_rule_cmd(
            ctx, f, "shell", "check", ["shfmt", "-d", "-i", "2", "-ci", str(f.abs_path)]
        )
        != 0
    ):
        failed = 1
    if run_rule_cmd(ctx, f, "shell", "check", ["shellcheck", str(f.abs_path)]) != 0:
        failed = 1
    return failed


def match_fish(_: LintContext, f: FileContext) -> bool:
    return is_fish(f.rel_path)


def fix_fish(ctx: LintContext, f: FileContext) -> int:
    if is_home_chezmoi_fish_template(f.rel_path, ctx.repo_root):
        return 0
    cmd = ["fish_indent", str(f.abs_path)]
    if ctx.verbose:
        print(f"[fix:fish] {f.rel_path}: {format_cmd(cmd)}")
    code, formatted, _ = run_capture(cmd)
    if code != 0:
        ctx.failures.append(
            FailureRecord(
                file=f.rel_path,
                rule="fish",
                phase="fix",
                command=format_cmd(cmd),
                exit_code=code,
            )
        )
        return 1
    if formatted != f.abs_path.read_text(encoding="utf-8", errors="ignore"):
        f.abs_path.write_text(formatted, encoding="utf-8")
    return 0


def check_fish(ctx: LintContext, f: FileContext) -> int:
    target = f.abs_path
    is_chezmoi_template = is_home_chezmoi_fish_template(f.rel_path, ctx.repo_root)
    if is_chezmoi_template:
        rendered = get_rendered(ctx, f, ".fish")
        if rendered is None:
            return 1
        target = rendered

    failed = 0
    if not is_chezmoi_template and not is_home_chezmoi_fish_completion_template(
        f.rel_path, ctx.repo_root
    ):
        if (
            run_rule_cmd(
                ctx, f, "fish", "check", ["fish_indent", "--check", str(target)]
            )
            != 0
        ):
            failed = 1
    if (
        run_rule_cmd(ctx, f, "fish", "check", ["fish", "--no-execute", str(target)])
        != 0
    ):
        failed = 1
    return failed


def match_zsh(_: LintContext, f: FileContext) -> bool:
    return is_zsh(f.rel_path)


def fix_zsh(_: LintContext, __: FileContext) -> int:
    return 0


def check_zsh(ctx: LintContext, f: FileContext) -> int:
    target = f.abs_path
    if is_home_chezmoi_zsh_template(f.rel_path, ctx.repo_root):
        rendered = get_rendered(ctx, f, ".zsh")
        if rendered is None:
            return 1
        target = rendered
    return (
        0
        if run_rule_cmd(ctx, f, "zsh", "check", ["zsh", "-n", str(target)]) == 0
        else 1
    )


def match_lua(_: LintContext, f: FileContext) -> bool:
    return is_lua(f.rel_path)


def fix_lua(ctx: LintContext, f: FileContext) -> int:
    return run_rule_cmd(ctx, f, "lua", "fix", ["stylua", str(f.abs_path)])


def check_lua(ctx: LintContext, f: FileContext) -> int:
    return (
        0
        if run_rule_cmd(ctx, f, "lua", "check", ["stylua", "--check", str(f.abs_path)])
        == 0
        else 1
    )


def match_python(_: LintContext, f: FileContext) -> bool:
    return is_python(f.rel_path)


def fix_python(ctx: LintContext, f: FileContext) -> int:
    return run_rule_cmd(ctx, f, "python", "fix", ["ruff", "format", str(f.abs_path)])


def check_python(ctx: LintContext, f: FileContext) -> int:
    return (
        0
        if run_rule_cmd(
            ctx, f, "python", "check", ["ruff", "format", "--check", str(f.abs_path)]
        )
        == 0
        else 1
    )


def match_toml(_: LintContext, f: FileContext) -> bool:
    return is_toml(f.rel_path)


def fix_toml(ctx: LintContext, f: FileContext) -> int:
    return run_rule_cmd(ctx, f, "toml", "fix", ["taplo", "fmt", str(f.abs_path)])


def check_toml(ctx: LintContext, f: FileContext) -> int:
    failed = 0
    if (
        run_rule_cmd(
            ctx, f, "toml", "check", ["taplo", "fmt", "--check", str(f.abs_path)]
        )
        != 0
    ):
        failed = 1
    if (
        run_rule_cmd(
            ctx, f, "toml", "check", ["taplo", "lint", str(f.abs_path)]
        )
        != 0
    ):
        failed = 1
    return failed


def match_markdown(_: LintContext, f: FileContext) -> bool:
    return is_markdown(f.rel_path)


def fix_markdown(ctx: LintContext, f: FileContext) -> int:
    return run_rule_cmd(
        ctx,
        f,
        "markdown",
        "fix",
        ["markdownlint-cli2", "--fix", f":{f.rel_path}"],
        cwd=ctx.repo_root,
    )


def check_markdown(ctx: LintContext, f: FileContext) -> int:
    return (
        0
        if run_rule_cmd(
            ctx,
            f,
            "markdown",
            "check",
            ["markdownlint-cli2", f":{f.rel_path}"],
            cwd=ctx.repo_root,
        )
        == 0
        else 1
    )


RULES: list[Rule] = [
    Rule(name="shell", match=match_shell, fix=fix_shell, check=check_shell),
    Rule(name="fish", match=match_fish, fix=fix_fish, check=check_fish),
    Rule(name="zsh", match=match_zsh, fix=fix_zsh, check=check_zsh),
    Rule(name="lua", match=match_lua, fix=fix_lua, check=check_lua),
    Rule(name="python", match=match_python, fix=fix_python, check=check_python),
    Rule(name="toml", match=match_toml, fix=fix_toml, check=check_toml),
    Rule(name="markdown", match=match_markdown, fix=fix_markdown, check=check_markdown),
]
