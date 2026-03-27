import subprocess
import sys
from pathlib import Path

from .models import FileContext, LintContext


def render_template(ctx: LintContext, src_rel: str, out_path: Path) -> bool:
    source_dir = ctx.repo_root / "home"
    cmd = [
        "chezmoi",
        "-S",
        str(source_dir),
        "execute-template",
        "-f",
        str(ctx.repo_root / src_rel),
    ]
    p = subprocess.run(cmd, check=False, text=True, capture_output=True)
    if p.returncode != 0:
        print(f"lint: chezmoi execute-template failed: {src_rel}", file=sys.stderr)
        if p.stderr:
            print(p.stderr.rstrip(), file=sys.stderr)
        return False
    out_path.write_text(p.stdout, encoding="utf-8")
    return True


def get_rendered(ctx: LintContext, f: FileContext, ext: str) -> Path | None:
    if ext in f.rendered_cache:
        return f.rendered_cache[ext]
    rendered_name = f"rendered_{f.rel_path.replace('/', '_').replace('.', '_')}{ext}"
    out = ctx.tmp_dir / rendered_name
    if not render_template(ctx, f.rel_path, out):
        return None
    f.rendered_cache[ext] = out
    return out
