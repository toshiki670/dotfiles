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
COLOR_RESET=$'\033[0m'
COLOR_GREEN=$'\033[32m'
COLOR_YELLOW=$'\033[33m'
COLOR_RED=$'\033[31m'

# Return color based on pace: actual usage vs expected usage given elapsed time.
#   pace < 0.7  → green  (well under pace, room to use more)
#   pace < 1.0  → reset  (on pace, neutral)
#   pace < 1.5  → yellow (over pace, consider slowing down)
#   pace >= 1.5 → red    (significantly over pace, slow down)
# Usage: pace_color <used_pct> <resets_at_epoch> <total_window_seconds>
pace_color() {
  local used_pct="$1"
  local resets_at="$2"
  local total_secs="$3"
  local now
  now=$(date +%s)

  local result
  result=$(awk -v used="$used_pct" -v resets="$resets_at" \
               -v total="$total_secs" -v now="$now" 'BEGIN {
    remaining = resets - now
    if (remaining < 0) remaining = 0
    elapsed = total - remaining
    if (elapsed < 0) elapsed = 0
    elapsed_pct = elapsed / total  # 0.0-1.0

    # Not enough elapsed time to judge pace
    if (elapsed_pct < 0.05) { print "reset"; exit }

    pace = used / (elapsed_pct * 100)

    if (pace >= 1.5)     print "red"
    else if (pace >= 1.0) print "yellow"
    else if (pace < 0.7)  print "green"
    else                  print "reset"
  }')

  case "$result" in
    red)    printf "%s" "$COLOR_RED" ;;
    yellow) printf "%s" "$COLOR_YELLOW" ;;
    green)  printf "%s" "$COLOR_GREEN" ;;
    *)      printf "%s" "$COLOR_RESET" ;;
  esac
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

# Format seconds remaining into human-readable string
# Usage: fmt_remaining <resets_at_epoch>
fmt_remaining() {
  local resets_at="$1"
  local now
  now=$(date +%s)
  local diff=$((resets_at - now))
  [ "$diff" -le 0 ] && printf "now" && return

  local days=$((diff / 86400))
  local hours=$(( (diff % 86400) / 3600 ))
  local mins=$(( (diff % 3600) / 60 ))

  if [ "$days" -gt 0 ]; then
    printf "%dd%dh" "$days" "$hours"
  elif [ "$hours" -gt 0 ]; then
    printf "%dh%dm" "$hours" "$mins"
  else
    printf "%dm" "$mins"
  fi
}

# Model
printf " %s" "$model"

# Context usage (pace-based if resets info available, otherwise no color)
if [ -n "$used_pct" ] && [ -n "$remaining_pct" ]; then
  used_int=$(printf '%.0f' "$used_pct")
  bar=$(make_bar "$used_int")
  printf "  │  󰾅 %s %d%%" "$bar" "$used_int"
fi

# Rate limits
if [ -n "$five_hour" ] && [ -n "$five_hour_resets" ]; then
  five_int=$(printf '%.0f' "$five_hour")
  bar=$(make_bar "$five_int")
  color=$(pace_color "$five_int" "$five_hour_resets" 18000)
  printf "  │  󰔛 %s/5h ${color}%s %d%%${COLOR_RESET}" "$(fmt_remaining "$five_hour_resets")" "$bar" "$five_int"
fi
if [ -n "$seven_day" ] && [ -n "$seven_day_resets" ]; then
  week_int=$(printf '%.0f' "$seven_day")
  bar=$(make_bar "$week_int")
  color=$(pace_color "$week_int" "$seven_day_resets" 604800)
  printf "  │  󰃰 %s/7d ${color}%s %d%%${COLOR_RESET}" "$(fmt_remaining "$seven_day_resets")" "$bar" "$week_int"
fi

echo ""
