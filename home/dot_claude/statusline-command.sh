#!/usr/bin/env bash
# Claude Code status line: model, context usage, rate limits (Nerd Fonts)

input=$(cat)

model=$(echo "$input" | jq -r '.model.display_name // "Unknown"')

used_pct=$(echo "$input" | jq -r '.context_window.used_percentage // empty')
remaining_pct=$(echo "$input" | jq -r '.context_window.remaining_percentage // empty')

five_hour=$(echo "$input" | jq -r '.rate_limits.five_hour.used_percentage // empty')
five_hour_resets=$(echo "$input" | jq -r '.rate_limits.five_hour.resets_at // empty')
seven_day=$(echo "$input" | jq -r '.rate_limits.seven_day.used_percentage // empty')
seven_day_resets=$(echo "$input" | jq -r '.rate_limits.seven_day.resets_at // empty')

# ANSI color codes
COLOR_RESET="\033[0m"
COLOR_YELLOW="\033[33m"
COLOR_RED="\033[31m"

# Return the ANSI color code based on percentage thresholds
# Usage: pct_color <percentage>
pct_color() {
  local pct="$1"
  if awk "BEGIN {exit !($pct >= 90)}"; then
    printf "%s" "$COLOR_RED"
  elif awk "BEGIN {exit !($pct >= 75)}"; then
    printf "%s" "$COLOR_YELLOW"
  else
    printf "%s" "$COLOR_RESET"
  fi
}

# Build a 10-block progress bar from a percentage value (0-100)
# Usage: make_bar <percentage>
make_bar() {
  local pct="$1"
  local filled=$(echo "$pct" | awk '{printf "%d", ($1 + 5) / 10}')
  [ "$filled" -gt 10 ] && filled=10
  local empty=$((10 - filled))
  local bar=""
  local i
  for i in $(seq 1 "$filled"); do bar="${bar}█"; done
  for i in $(seq 1 "$empty");  do bar="${bar}░"; done
  printf "%s" "$bar"
}

# Format seconds remaining into human-readable countdown
# Usage: fmt_reset <resets_at_epoch>
fmt_reset() {
  local resets_at="$1"
  local now
  now=$(date +%s)
  local diff=$((resets_at - now))
  [ "$diff" -le 0 ] && printf "↺now" && return

  local days=$((diff / 86400))
  local hours=$(( (diff % 86400) / 3600 ))
  local mins=$(( (diff % 3600) / 60 ))

  if [ "$days" -gt 0 ]; then
    printf "↺%dd%dh" "$days" "$hours"
  elif [ "$hours" -gt 0 ]; then
    printf "↺%dh%dm" "$hours" "$mins"
  else
    printf "↺%dm" "$mins"
  fi
}

# Model
printf " %s" "$model"

# Context usage
if [ -n "$used_pct" ] && [ -n "$remaining_pct" ]; then
  used_int=$(printf '%.0f' "$used_pct")
  bar=$(make_bar "$used_int")
  color=$(pct_color "$used_int")
  printf "  │  󰾅 ${color}%s %d%%${COLOR_RESET}" "$bar" "$used_int"
fi

# Rate limits
if [ -n "$five_hour" ] || [ -n "$seven_day" ]; then
  printf "  │ "
  if [ -n "$five_hour" ]; then
    five_int=$(printf '%.0f' "$five_hour")
    bar=$(make_bar "$five_int")
    color=$(pct_color "$five_int")
    printf "  󰔛 5h ${color}%s %d%%${COLOR_RESET}" "$bar" "$five_int"
    if [ -n "$five_hour_resets" ]; then
      printf " %s" "$(fmt_reset "$five_hour_resets")"
    fi
  fi
  if [ -n "$seven_day" ]; then
    week_int=$(printf '%.0f' "$seven_day")
    bar=$(make_bar "$week_int")
    color=$(pct_color "$week_int")
    printf "  󰃰 7d ${color}%s %d%%${COLOR_RESET}" "$bar" "$week_int"
    if [ -n "$seven_day_resets" ]; then
      printf " %s" "$(fmt_reset "$seven_day_resets")"
    fi
  fi
fi

echo ""
