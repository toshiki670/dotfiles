import argparse
import sys
import tempfile
from pathlib import Path

from .collect import collect_files, find_repo_root
from .models import LintContext
from .runner import print_json, print_summary, run_check_for_file, run_fix_for_file


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(usage="lint.py {fix|check}")
    parser.add_argument("mode", choices=["fix", "check"])
    parser.add_argument(
        "--check-after-fix",
        choices=["on", "off"],
        default="on",
        help="run check phase after fix (default: on)",
    )
    parser.add_argument("--summary", action="store_true", help="print failure summary")
    parser.add_argument(
        "--json",
        dest="json_output",
        action="store_true",
        help="print JSON summary",
    )
    parser.add_argument(
        "--verbose", action="store_true", help="print executed commands"
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)

    repo_root = find_repo_root(Path.cwd())
    files = collect_files(repo_root)

    with tempfile.TemporaryDirectory() as td:
        ctx = LintContext(
            repo_root=repo_root,
            tmp_dir=Path(td),
            mode=args.mode,
            check_after_fix=(args.check_after_fix == "on"),
            verbose=args.verbose,
            summary=args.summary,
            json_output=args.json_output,
        )
        failed = 0

        if args.mode == "fix":
            for file_ctx in files:
                if run_fix_for_file(ctx, file_ctx) != 0:
                    failed = 1
            print("lint(fix): completed")

        if args.mode == "check" or ctx.check_after_fix:
            for file_ctx in files:
                if run_check_for_file(ctx, file_ctx) != 0:
                    failed = 1

        if ctx.summary:
            print_summary(ctx)
        if ctx.json_output:
            print_json(ctx, failed)

    return failed


if __name__ == "__main__":
    sys.exit(main())
