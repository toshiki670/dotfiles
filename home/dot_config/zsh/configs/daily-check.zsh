# Daily package outdated check
# Run brew outdated and mise outdated once per calendar day.
# 1st run: execute in background and save result to a temp file.
# 2nd run: display the saved result, then delete the file asynchronously.
daily-check() {
  local cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/zsh"
  local timestamp_file="$cache_dir/daily-check-timestamp"
  local result_file="$cache_dir/daily-check-result"

  # If result file exists: display it and delete asynchronously
  if [[ -f "$result_file" ]]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Daily Package Check"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cat "$result_file"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    ( rm -f "$result_file" & )
    return
  fi

  # Start background job; date check and brew/mise run inside it
  setopt local_options no_monitor
  "checking outdated"() {
    local today=$(strftime %F $EPOCHSECONDS)
    if [[ -f "$timestamp_file" ]]; then
      local last_run=$(< "$timestamp_file")
      [[ "$last_run" == "$today" ]] && return 0
    fi
    [[ ! -d "$cache_dir" ]] && mkdir -p "$cache_dir"
    print -n "$today" >| "$timestamp_file"
    {
      echo "=== Homebrew Outdated Packages ==="
      echo ""
      (( $+commands[brew] )) && brew outdated 2>/dev/null
      echo ""
      echo "=== Mise Outdated Tools ==="
      echo ""
      (( $+commands[mise] )) && mise outdated 2>/dev/null
    } > "$result_file" 2>&1
  }
  ( cache_dir="$cache_dir" timestamp_file="$timestamp_file" result_file="$result_file" "checking outdated" ) &
  disown
  echo ""
  echo "━━━ ⏳ Checking Outdated ━━━"
  echo ""
}

# Run the daily check on shell startup
daily-check
