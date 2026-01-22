# Daily package outdated check
# Run brew outdated and mise outdated once per day

daily-check() {
  local timestamp_file="${XDG_CACHE_HOME:-$HOME/.cache}/zsh/daily-check-timestamp"
  
  # Check if file is older than 24 hours using glob qualifiers
  local should_run=0
  if [[ ! -f "$timestamp_file" ]]; then
    should_run=1
  else
    () {
      setopt local_options null_glob
      local old_files=($timestamp_file(mh+24))
      (( ${#old_files} > 0 )) && should_run=1
    }
  fi
  
  if (( should_run )); then
    # Create cache directory only when needed
    [[ ! -d "${timestamp_file:h}" ]] && mkdir -p "${timestamp_file:h}"
    
    # Update timestamp immediately to prevent multiple runs
    : >| "$timestamp_file"
    
    # Display synchronously before prompt to avoid display issues
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Daily Package Check"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    (( $+commands[brew] )) && {
      echo ""
      echo "=== Homebrew Outdated Packages ==="
      brew outdated 2>/dev/null
    }
    
    (( $+commands[mise] )) && {
      echo ""
      echo "=== Mise Outdated Tools ==="
      mise outdated 2>/dev/null
    }
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
  fi
}

# Run the daily check on shell startup
daily-check
