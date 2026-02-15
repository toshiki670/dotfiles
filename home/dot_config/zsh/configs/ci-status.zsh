# Show remote CI status (GitHub Actions) in Pure prompt.
# Requires: gh CLI (https://cli.github.com/), and GitHub repo.
# Supports GitHub.com and GitHub Enterprise Server (on-premises); uses origin URL to detect host.
#
# Spec (when zsh-async is available):
# - Triggers: (1) terminal open, (2) Enter, (3) exec $SHELL -l.
# - On trigger: run CI check in background; when the job completes, show the checkmark (no second Enter).
# - When a trigger runs within CI_STATUS_CTX[cache_seconds] (default 10s): skip fetch, show cached result when job completes.
# - Result is written only in the callback (job complete → PROMPT update + zle .reset-prompt).

# Initialize CI_STATUS_CTX for dependency injection
# This allows testing by replacing git/gh commands and paths
typeset -gA CI_STATUS_CTX

# Nerd Font icon mapping (class name -> Unicode character)
# This is separate from CI_STATUS_CTX as it's not a dependency injection target
typeset -gA CI_STATUS_ICON_MAP
CI_STATUS_ICON_MAP=(
  "nf-md-check" "󰄬"                    # ok
  "nf-md-launch" "󰌧"                   # waiting
  "nf-md-close" "󰅖"                    # ng
  "nf-md-refresh" "󰑐"                  # in_progress
  "nf-md-move_resize_variant" "󰙖"      # action_required
  "nf-oct-git_pull_request_closed" ""  # closed
  "nf-oct-git_pull_request_draft" ""   # draft
  "nf-oct-git_merge" ""                # merged
)

# Define wrapper functions for git/gh commands
_ci_status_git_remote_url() {
  git ls-remote --get-url origin "$@" 2>/dev/null
}

_ci_status_git_toplevel_branch() {
  git rev-parse --show-toplevel --abbrev-ref HEAD "$@" 2>/dev/null
}

_ci_status_git_has_repo() {
  git rev-parse --git-dir >/dev/null 2>&1
}

_ci_status_gh_pr_view() {
  gh pr view "$@" 2>/dev/null
}

_ci_status_gh_pr_checks() {
  gh pr checks "$@" 2>/dev/null
}

_ci_status_gh_auth_status() {
  gh auth status "$@" 2>/dev/null
}

# Initialize CI_STATUS_CTX with default values
CI_STATUS_CTX=(
  # Settings and paths
  cache_seconds 10
  cache_dir "${XDG_CACHE_HOME:-$HOME/.cache}/ci-status"
  gh_hosts_file "${XDG_CACHE_HOME:-$HOME/.cache}/ci-status/gh_hosts"
  error_log_file "${XDG_CACHE_HOME:-$HOME/.cache}/ci-status/error.log"

  # Git commands (callable functions that take arguments)
  git_remote_url "_ci_status_git_remote_url"
  git_toplevel_branch "_ci_status_git_toplevel_branch"
  git_has_repo "_ci_status_git_has_repo"

  # gh commands (callable functions that take arguments)
  gh_pr_view "_ci_status_gh_pr_view"
  gh_pr_checks "_ci_status_gh_pr_checks"
  gh_auth_status "_ci_status_gh_auth_status"
)

# Format duration from checks data
# Input: min_start (ISO 8601 string), max_end (ISO 8601 string), has_pending (0 or 1), current time (Unix timestamp)
# Output: Formatted duration string like "1h 1m 1s" or "" (empty if no checks)
ci_status_format_duration() {
  local min_start=$1 max_end=$2 has_pending=$3 now=$4

  if [[ -z "$min_start" ]] || [[ "$min_start" == "null" ]]; then
    echo ""
    return
  fi

  # Convert ISO 8601 (RFC3339) to Unix timestamp
  # Format: "2024-01-01T12:00:00Z" or "2024-01-01T12:00:00.123Z"
  local min_start_ts max_end_ts
  if [[ "$(uname)" == "Darwin" ]]; then
    # macOS: date -j -f format input +%s
    # Remove fractional seconds and timezone offset, handle Z suffix
    local min_start_normalized="${min_start%.*}"  # Remove fractional seconds
    min_start_normalized="${min_start_normalized%+*}"  # Remove timezone offset like +09:00
    if [[ "$min_start_normalized" == *"Z" ]]; then
      min_start_normalized="${min_start_normalized%Z}"
      min_start_ts=$(date -j -u -f "%Y-%m-%dT%H:%M:%S" "$min_start_normalized" +%s 2>/dev/null || echo "")
    else
      min_start_ts=$(date -j -f "%Y-%m-%dT%H:%M:%S" "$min_start_normalized" +%s 2>/dev/null || echo "")
    fi
  else
    # Linux: date -d input +%s (handles ISO 8601 directly)
    min_start_ts=$(date -d "$min_start" +%s 2>/dev/null || echo "")
  fi

  if [[ -z "$min_start_ts" ]]; then
    echo ""
    return
  fi

  local duration_sec
  if [[ "$has_pending" -gt 0 ]]; then
    # In progress: use current time
    duration_sec=$((now - min_start_ts))
  else
    # Completed: use max_end
    if [[ -n "$max_end" ]] && [[ "$max_end" != "null" ]]; then
      # Convert max_end to Unix timestamp
      if [[ "$(uname)" == "Darwin" ]]; then
        local max_end_normalized="${max_end%.*}"  # Remove fractional seconds
        max_end_normalized="${max_end_normalized%+*}"  # Remove timezone offset
        if [[ "$max_end_normalized" == *"Z" ]]; then
          max_end_normalized="${max_end_normalized%Z}"
          max_end_ts=$(date -j -u -f "%Y-%m-%dT%H:%M:%S" "$max_end_normalized" +%s 2>/dev/null || echo "")
        else
          max_end_ts=$(date -j -f "%Y-%m-%dT%H:%M:%S" "$max_end_normalized" +%s 2>/dev/null || echo "")
        fi
      else
        max_end_ts=$(date -d "$max_end" +%s 2>/dev/null || echo "")
      fi
      if [[ -n "$max_end_ts" ]]; then
        duration_sec=$((max_end_ts - min_start_ts))
      else
        echo ""
        return
      fi
    else
      echo ""
      return
    fi
  fi

  # Format: 1h 1m 1s
  local h m s
  h=$((duration_sec / 3600))
  m=$(((duration_sec % 3600) / 60))
  s=$((duration_sec % 60))

  local result=""
  if [[ $h -ge 1 ]]; then
    result="${h}h"
  fi
  if [[ $m -ge 1 ]]; then
    [[ -n "$result" ]] && result="${result} "
    result="${result}${m}m"
  fi
  if [[ $s -ge 0 ]]; then
    [[ -n "$result" ]] && result="${result} "
    result="${result}${s}s"
  fi

  echo "$result"
}

# Echo prompt string for PR state and checks status
# Input: "pr_state,checks_state,duration" format string
# Output: Formatted prompt string with symbols and duration separated by spaces
ci_status_prompt_from_result() {
  local input=$1
  if [[ -z "$input" ]]; then
    echo ""
    return
  fi

  local -a fields
  fields=(${(s:,:)input})
  local pr_state=${fields[1]} checks_state=${fields[2]} duration=${fields[3]}

  # PR state symbol mapping using Nerd Font class names
  local pr_symbol=""
  case "$pr_state" in
    ok) 
      pr_symbol="%F{green}${CI_STATUS_ICON_MAP[nf-md-check]:-}%f" 
      ;;
    waiting) 
      pr_symbol="%F{blue}${CI_STATUS_ICON_MAP[nf-md-launch]:-}%f" 
      ;;
    ng) 
      pr_symbol="%F{red}${CI_STATUS_ICON_MAP[nf-md-close]:-}%f" 
      ;;
    closed) 
      pr_symbol="%F{red}${CI_STATUS_ICON_MAP[nf-oct-git_pull_request_closed]:-}%f" 
      ;;
    draft) 
      pr_symbol="%F{blue}${CI_STATUS_ICON_MAP[nf-oct-git_pull_request_draft]:-}%f" 
      ;;
    merged) 
      pr_symbol="%F{green}${CI_STATUS_ICON_MAP[nf-oct-git_merge]:-}%f" 
      ;;
    *) 
      pr_symbol="" 
      ;;
  esac

  # If no PR state, return empty
  if [[ -z "$pr_symbol" ]]; then
    echo ""
    return
  fi

  # If no checks state, return only PR state
  if [[ -z "$checks_state" ]]; then
    echo "$pr_symbol"
    return
  fi

  # Checks state symbol mapping using Nerd Font class names
  local checks_symbol=""
  case "$checks_state" in
    ok) 
      checks_symbol="%F{green}${CI_STATUS_ICON_MAP[nf-md-check]:-}%f" 
      ;;
    in_progress) 
      checks_symbol="%F{yellow}${CI_STATUS_ICON_MAP[nf-md-refresh]:-}%f" 
      ;;
    action_required) 
      checks_symbol="%F{magenta}${CI_STATUS_ICON_MAP[nf-md-move_resize_variant]:-}%f" 
      ;;
    ng) 
      checks_symbol="%F{red}${CI_STATUS_ICON_MAP[nf-md-close]:-}%f" 
      ;;
    *) 
      checks_symbol="" 
      ;;
  esac

  # Build result: pr_symbol checks_symbol duration (space-separated)
  local result="$pr_symbol"
  if [[ -n "$checks_symbol" ]]; then
    result="${result} ${checks_symbol}"
    if [[ -n "$duration" ]]; then
      result="${result} %F{yellow}${duration}%f"
    fi
  fi

  echo "$result"
}

# Write error log to file in background (non-blocking)
ci_status_log_error() {
  local line=$1 context=$2
  (
    mkdir -p "${CI_STATUS_CTX[error_log_file]:h}"
    echo "[$(date +%Y-%m-%d\ %H:%M:%S)] F{$line} $context" >> "${CI_STATUS_CTX[error_log_file]}" 2>/dev/null
  ) &!
}

# Checks if the host is in the cached list of available GitHub hosts.
ci_status_gh_available() {
  # Get remote URL using ctx
  local remote_url
  remote_url=$(${CI_STATUS_CTX[git_remote_url]}) || {
    ci_status_log_error $LINENO "ci_status_gh_available: failed to get remote URL"
    return 1
  }
  # Load cache file into array
  local -a hosts
  local cache_content
  cache_content=$(cat "${CI_STATUS_CTX[gh_hosts_file]}" 2>/dev/null)
  hosts=(${(f)cache_content})
  # Check if any cached host is contained in the remote URL
  # Use zsh pattern matching: build pattern *host1*|*host2*|...
  local pattern
  pattern="*(${(j:|:)hosts})*"
  if [[ "$remote_url" == ${~pattern} ]]; then
    return 0
  fi
  # If no host matched, return 1
  ci_status_log_error $LINENO "ci_status_gh_available: no host matched"
  return 1
}

# Check if cache file is stale (older than CI_STATUS_CTX[cache_seconds])
ci_status_is_cache_stale() {
  local cache_file=$1
  # If file doesn't exist, consider it stale
  [[ ! -f "$cache_file" ]] && return 0
  setopt local_options null_glob
  local old_files=($cache_file(ms+${CI_STATUS_CTX[cache_seconds]}))
  (( ${#old_files} > 0 ))
}

# Prints path to cache file: ~/.cache/ci-status/repos/<toplevel_path>_<branch> (single file, / replaced with _)
ci_status_cache_file() {
  local path_joined filename
  path_joined="${(j:/:)${(f)$(${CI_STATUS_CTX[git_toplevel_branch]})}}"
  [[ -z "$path_joined" ]] && {
    ci_status_log_error $LINENO "ci_status_cache_file: failed to get path"
    return 1
  }
  filename="${path_joined//\//_}"
  echo "${CI_STATUS_CTX[cache_dir]}/repos/$filename"
}

# Get cache file path and status. If cache is stale, fetch and update; else return cached result.
# Output: "cache_file_path\npr_state,checks_state,duration"
ci_status_cache_or_fetch() {
  ci_status_gh_available || return 0

  local cache_file
  cache_file=$(ci_status_cache_file) || {
    ci_status_log_error $LINENO "ci_status_cache_or_fetch: ci_status_cache_file failed"
    return 1
  }
  mkdir -p "${cache_file:h}"

  # If cache is fresh, return cached result
  if ! ci_status_is_cache_stale "$cache_file"; then
    echo "$cache_file"
    cat "$cache_file" 2>/dev/null || echo ""
    return 0
  fi

  # Cache is stale, fetch and update
  # Fetch PR view and checks data in parallel for better performance
  local pr_view_tmp checks_tmp
  pr_view_tmp=$(mktemp) || {
    ci_status_log_error $LINENO "ci_status_cache_or_fetch: failed to create temp file for pr_view"
    return 1
  }
  checks_tmp=$(mktemp) || {
    ci_status_log_error $LINENO "ci_status_cache_or_fetch: failed to create temp file for checks"
    rm -f "$pr_view_tmp"
    return 1
  }

  # Run gh_pr_view and gh_pr_checks in parallel
  (
    ${CI_STATUS_CTX[gh_pr_view]} --json state,mergedAt,closed,mergeable,mergeStateStatus,reviewDecision,isDraft --jq '
      if . == null then
        ""
      elif .state == "MERGED" or (.mergedAt != null and .mergedAt != "") then
        "merged"
      elif .state == "CLOSED" or .closed == true then
        "closed"
      elif .mergeable == "CONFLICTING" or .reviewDecision == "CHANGES_REQUESTED" then
        "ng"
      elif .isDraft == true then
        "draft"
      elif .reviewDecision == "REVIEW_REQUIRED" or .mergeStateStatus == "BEHIND" then
        "waiting"
      else
        "ok"
      end
    ' > "$pr_view_tmp" 2>/dev/null || echo "" > "$pr_view_tmp"
  ) &
  local pr_view_pid=$!

  (
    ${CI_STATUS_CTX[gh_pr_checks]} --json state,bucket,startedAt,completedAt --jq '
      (
        if length == 0 then
          ""
        elif [.[] | select(.bucket == "fail" or .bucket == "cancel")] | length > 0 then
          "ng"
        elif [.[] | select(.state == "ACTION_REQUIRED")] | length > 0 then
          "action_required"
        elif [.[] | select(.bucket == "pending")] | length > 0 then
          "in_progress"
        else
          "ok"
        end
      ) + "\n" +
      ([.[] | select(.startedAt != null and .startedAt != "") | .startedAt] | min // empty) + "\n" +
      ([.[] | select(.completedAt != null and .completedAt != "") | .completedAt] | max // empty) + "\n" +
      ([.[] | select(.bucket == "pending" or .state == "ACTION_REQUIRED")] | length | tostring)
    ' > "$checks_tmp" 2>/dev/null || echo "" > "$checks_tmp"
  ) &
  local checks_pid=$!

  # Wait for both commands to complete
  wait $pr_view_pid
  local pr_view_exit=$?
  wait $checks_pid
  local checks_exit=$?

  # Read results from temporary files
  local pr_state checks_output
  pr_state=$(cat "$pr_view_tmp" 2>/dev/null || echo "")
  checks_output=$(cat "$checks_tmp" 2>/dev/null || echo "")

  # Clean up temporary files
  rm -f "$pr_view_tmp" "$checks_tmp"

  if [[ -z "$pr_state" ]]; then
    # PR doesn't exist, return empty string (same as before parallelization)
    echo "" > "$cache_file"
    echo "$cache_file"
    echo ""
    return 0
  fi

  local checks_state duration result
  if [[ -n "$checks_output" ]]; then
    # Parse the output: checks_state\nmin_start\nmax_end\nhas_pending
    # Note: zsh's (f) flag may skip empty lines, so we handle missing lines
    local -a lines
    lines=(${(f)checks_output})
    checks_state=${lines[1]}
    local min_start=${lines[2]} max_end=${lines[3]:-""} has_pending=${lines[4]:-${lines[3]}}

    if [[ -n "$checks_state" ]]; then
      # Calculate duration
      local now_ts
      now_ts=$(date +%s 2>/dev/null || echo "")
      if [[ -n "$now_ts" ]]; then
        duration=$(ci_status_format_duration "$min_start" "$max_end" "$has_pending" "$now_ts")
      else
        duration=""
      fi
      result="${pr_state},${checks_state},${duration}"
    else
      result="${pr_state},,"
    fi
  else
    # No checks exist, return PR state only
    result="${pr_state},,"
  fi

  echo "$result" > "$cache_file"
  echo "$cache_file"
  echo "$result"
}

# Runs in zsh-async worker. Output "path\nstatus".
ci_status_async_fetch() {
  ci_status_cache_or_fetch
}

ci_status_async_callback() {
  local job=$1 code=$2 output=$3 next_pending=$6
  [[ $job != ci_status_async_fetch ]] && return
  (( code )) && return

  local -a lines
  lines=("${(f)output}")
  local job_cache_file=${lines[1]} result=${lines[2]}
  local current_cache_file
  current_cache_file=$(ci_status_cache_file 2>/dev/null) || return
  [[ "$job_cache_file" != "$current_cache_file" ]] && return

  CI_STATUS_PROMPT=$(ci_status_prompt_from_result "$result")
  if (( ! next_pending )); then
    if [[ -n "$prompt_newline" && -n "$CI_STATUS_PROMPT" ]]; then
      PROMPT="${PROMPT//$prompt_newline/ $CI_STATUS_PROMPT$prompt_newline}"
    fi
    zle && zle .reset-prompt
  fi
}

precmd_ci_status() {
  [[ -z "$prompt_newline" ]] && return
  if ! ${CI_STATUS_CTX[git_has_repo]}; then
    CI_STATUS_PROMPT=""
    return
  fi

  if (( $+functions[async_start_worker] )); then
    (( ${+ci_status_async_inited} )) || {
      typeset -g ci_status_async_inited=1
      async_start_worker "ci_status" -u -n
      async_register_callback "ci_status" ci_status_async_callback
    }
    async_worker_eval "ci_status" builtin cd -q $PWD
    async_job "ci_status" ci_status_async_fetch
  else
    ci_status_precmd_sync
  fi
}

# Sync path: used only when zsh-async is not available.
ci_status_precmd_sync() {
  local output
  output=$(ci_status_cache_or_fetch) || { CI_STATUS_PROMPT=""; return; }
  local -a lines
  lines=("${(f)output}")
  local result=${lines[2]}
  CI_STATUS_PROMPT=$(ci_status_prompt_from_result "${result:-}")
  if [[ -n "$CI_STATUS_PROMPT" ]]; then
    PROMPT="${PROMPT//$prompt_newline/ $CI_STATUS_PROMPT$prompt_newline}"
  fi
}

# Initialize GitHub hosts cache by running gh auth status in background
ci_status_init_gh_hosts_cache() {
  mkdir -p "${CI_STATUS_CTX[cache_dir]}"

  # Run gh auth status in background and save active hosts to cache file
  (
    ${CI_STATUS_CTX[gh_auth_status]} --json hosts --jq '[.hosts | to_entries[] | .key as $host | .value[] | select(.active == true) | $host] | unique[]' > "${CI_STATUS_CTX[gh_hosts_file]}" || true
  ) &!
}

if command -v gh >/dev/null 2>&1; then
  ci_status_init_gh_hosts_cache
  add-zsh-hook precmd precmd_ci_status
fi
