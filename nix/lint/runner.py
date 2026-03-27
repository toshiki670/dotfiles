import json

from .models import FileContext, LintContext
from .rules import RULES


def run_fix_for_file(ctx: LintContext, file_ctx: FileContext) -> int:
    failed = 0
    for rule in RULES:
        if rule.match(ctx, file_ctx) and rule.fix(ctx, file_ctx) != 0:
            failed = 1
    return failed


def run_check_for_file(ctx: LintContext, file_ctx: FileContext) -> int:
    failed = 0
    for rule in RULES:
        if rule.match(ctx, file_ctx) and rule.check(ctx, file_ctx) != 0:
            failed = 1
    return failed


def print_summary(ctx: LintContext) -> None:
    print(f"lint: failures={len(ctx.failures)}")
    for rec in ctx.failures:
        print(f"- {rec.phase}:{rec.rule} {rec.file} -> ({rec.exit_code}) {rec.command}")


def print_json(ctx: LintContext, failed: int) -> None:
    payload = {
        "failed": failed,
        "failureCount": len(ctx.failures),
        "failures": [
            {
                "file": rec.file,
                "rule": rec.rule,
                "phase": rec.phase,
                "command": rec.command,
                "exitCode": rec.exit_code,
            }
            for rec in ctx.failures
        ],
    }
    print(json.dumps(payload))
