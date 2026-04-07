#!/usr/bin/env python3
# Claude Code status line: model, context usage, rate limits (Nerd Fonts)

import json
import sys
import time

COLOR_RESET = "\033[0m"
COLOR_GREEN = "\033[32m"
COLOR_YELLOW = "\033[33m"
COLOR_RED = "\033[31m"


def pace_color(used_pct: float, resets_at: int, total_secs: int) -> str:
    """Return ANSI color based on pace: actual usage vs expected usage given elapsed time.

    pace < 0.7  → green  (well under pace, room to use more)
    pace < 1.0  → reset  (on pace, neutral)
    pace < 1.5  → yellow (over pace, consider slowing down)
    pace >= 1.5 → red    (significantly over pace, slow down)
    """
    now = int(time.time())
    remaining = max(resets_at - now, 0)
    elapsed = max(total_secs - remaining, 0)
    elapsed_pct = elapsed / total_secs

    if elapsed_pct < 0.05:
        return COLOR_RESET

    pace = used_pct / (elapsed_pct * 100)

    if pace >= 1.5:
        return COLOR_RED
    elif pace >= 1.0:
        return COLOR_YELLOW
    elif pace < 0.7:
        return COLOR_GREEN
    else:
        return COLOR_RESET


def make_bar(pct: float) -> str:
    """Build a 10-block progress bar from a percentage value (0-100)."""
    filled = min(int((pct + 5) / 10), 10)
    empty = 10 - filled
    return "█" * filled + "░" * empty


def fmt_remaining(resets_at: int) -> str:
    """Format seconds remaining into human-readable string."""
    diff = resets_at - int(time.time())
    if diff <= 0:
        return "now"

    days = diff // 86400
    hours = (diff % 86400) // 3600
    mins = (diff % 3600) // 60

    if days > 0:
        return f"{days}d{hours}h"
    elif hours > 0:
        return f"{hours}h{mins}m"
    else:
        return f"{mins}m"


def main() -> None:
    data = json.load(sys.stdin)

    model = data.get("model", {}).get("display_name", "Unknown")

    ctx = data.get("context_window", {})
    used_pct = ctx.get("used_percentage")
    remaining_pct = ctx.get("remaining_percentage")

    rl = data.get("rate_limits", {})
    five_hour = rl.get("five_hour", {}).get("used_percentage")
    five_hour_resets = rl.get("five_hour", {}).get("resets_at")
    seven_day = rl.get("seven_day", {}).get("used_percentage")
    seven_day_resets = rl.get("seven_day", {}).get("resets_at")

    out = []

    # Model
    out.append(f" {model}")

    # Context usage
    if used_pct is not None and remaining_pct is not None:
        used_int = round(used_pct)
        bar = make_bar(used_int)
        out.append(f"  │  \U000f0f85 {bar} {used_int}%")

    # Rate limits
    if five_hour is not None and five_hour_resets is not None:
        five_int = round(five_hour)
        bar = make_bar(five_int)
        color = pace_color(five_int, five_hour_resets, 18000)
        remaining = fmt_remaining(five_hour_resets)
        out.append(f"  │  \U000f051b {remaining}/5h {color}{bar} {five_int}%{COLOR_RESET}")

    if seven_day is not None and seven_day_resets is not None:
        week_int = round(seven_day)
        bar = make_bar(week_int)
        color = pace_color(week_int, seven_day_resets, 604800)
        remaining = fmt_remaining(seven_day_resets)
        out.append(f"  │  \U000f00f0 {remaining}/7d {color}{bar} {week_int}%{COLOR_RESET}")

    print("".join(out))


if __name__ == "__main__":
    main()
