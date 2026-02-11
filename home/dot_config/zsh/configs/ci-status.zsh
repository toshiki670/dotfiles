# Show remote CI status (GitHub Actions) in Pure prompt.
# Requires: gh CLI (https://cli.github.com/), and GitHub repo.
# Supports GitHub.com and GitHub Enterprise Server (on-premises); uses origin URL to detect host.
#
# Spec (when zsh-async is available):
# - Triggers: (1) terminal open, (2) Enter, (3) exec $SHELL -l.
# - On trigger: run CI check in background; when the job completes, show the checkmark (no second Enter).
# - When a trigger runs within CI_STATUS_CACHE_SECONDS (default 15s): skip fetch, show cached result when job completes.
# - Result is written only in the callback (job complete → PROMPT update + zle .reset-prompt).
(( ${+CI_STATUS_CACHE_SECONDS} )) || typeset -g CI_STATUS_CACHE_SECONDS=15
(( ${+CI_STATUS_CACHE_DIR} )) || typeset -g CI_STATUS_CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/ci-status"
(( ${+CI_STATUS_GH_HOSTS_CACHE_FILE} )) || typeset -g CI_STATUS_GH_HOSTS_CACHE_FILE="${CI_STATUS_CACHE_DIR}/gh_hosts"
(( ${+CI_STATUS_ERROR_LOG_FILE} )) || typeset -g CI_STATUS_ERROR_LOG_FILE="${CI_STATUS_CACHE_DIR}/error.log"

# Echo prompt string for status (success/failure/in_progress/waiting/action_required/skipped/cancelled/unknown). Caller: CI_STATUS_PROMPT=$(ci_status_prompt_from_result "$result")
ci_status_prompt_from_result() {
  local -A m=(success '%F{green}✓%f' failure '%F{red}✗%f' in_progress '%F{yellow}⟳%f' waiting '%F{blue}○%f' action_required '%F{magenta}⏸%f' skipped '%F{242}−%f' cancelled '%F{214}⊖%f' unknown '%F{242}?%f')
  echo "${m[$1]:-}"
}

# Write error log to file in background (non-blocking)
ci_status_log_error() {
  local line=$1 context=$2
  (
    mkdir -p "${CI_STATUS_ERROR_LOG_FILE:h}"
    echo "[$(date +%Y-%m-%d\ %H:%M:%S)] F{$line} $context" >> "$CI_STATUS_ERROR_LOG_FILE" 2>/dev/null
  ) &!
}

# Checks if the host is in the cached list of available GitHub hosts.
ci_status_gh_available() {
  # Get remote URL
  local remote_url
  remote_url=$(git ls-remote --get-url origin 2>/dev/null) || {
    ci_status_log_error $LINENO "ci_status_gh_available: failed to get remote URL"
    return 1
  }
  # Load cache file into array
  local -a hosts
  local cache_content
  cache_content=$(cat "$CI_STATUS_GH_HOSTS_CACHE_FILE" 2>/dev/null)
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

# Check if cache file is stale (older than CI_STATUS_CACHE_SECONDS)
ci_status_is_cache_stale() {
  local cache_file=$1
  # If file doesn't exist, consider it stale
  [[ ! -f "$cache_file" ]] && return 0
  setopt local_options null_glob
  local old_files=($cache_file(ms+$CI_STATUS_CACHE_SECONDS))
  (( ${#old_files} > 0 ))
}

# Prints path to cache file: ~/.cache/ci-status/repos/<toplevel_path>_<branch> (single file, / replaced with _)
ci_status_cache_file() {
  local path_joined filename
  path_joined="${(j:/:)${(f)$(git rev-parse --show-toplevel --abbrev-ref HEAD 2>/dev/null)}}"
  [[ -z "$path_joined" ]] && {
    ci_status_log_error $LINENO "ci_status_cache_file: failed to get path"
    return 1
  }
  filename="${path_joined//\//_}"
  echo "$CI_STATUS_CACHE_DIR/repos/$filename"
}

ci_status_fetch() {
  ci_status_gh_available || return 0

  local cache_file
  cache_file=$(ci_status_cache_file) || {
    ci_status_log_error $LINENO "ci_status_fetch: ci_status_cache_file failed"
    return 1
  }
  local branch
  branch=$(git branch --show-current 2>/dev/null) || {
    ci_status_log_error $LINENO "ci_status_fetch: failed to get branch"
    return 1
  }
  mkdir -p "${cache_file:h}"

  local result
  result=$(gh run list -b "$branch" -L 1 --json conclusion,status -q '
    if .[0] == null then "unknown"
    elif .[0].status == "in_progress" then "in_progress"
    elif .[0].status == "queued" or .[0].status == "waiting" or .[0].status == "requested" then "waiting"
    elif .[0].status == "action_required" or .[0].status == "pending" then "action_required"
    elif .[0].status == "completed" then
      if .[0].conclusion == "success" then "success"
      elif .[0].conclusion == "failure" then "failure"
      elif .[0].conclusion == "cancelled" then "cancelled"
      elif .[0].conclusion == "skipped" then "skipped"
      else "unknown"
      end
    else "unknown"
    end
  ' 2>/dev/null)
  echo "${result:-unknown}" > "$cache_file"
}

# Runs in zsh-async worker. If cache is older than CI_STATUS_CACHE_SECONDS, fetch; else use cache. Output "path\nstatus".
ci_status_async_fetch() {
  local cache_file
  cache_file=$(ci_status_cache_file) || {
    ci_status_log_error $LINENO "ci_status_async_fetch: ci_status_cache_file failed"
    return 1
  }
  # Only fetch when cache is stale (older than CI_STATUS_CACHE_SECONDS)
  if ci_status_is_cache_stale "$cache_file"; then
    ci_status_fetch
  fi
  echo "$cache_file"
  cat "$cache_file" 2>/dev/null || echo "unknown"
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
  if ! git rev-parse --git-dir >/dev/null 2>&1; then
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
  local cache_file
  cache_file=$(ci_status_cache_file) || { CI_STATUS_PROMPT=""; return; }
  if ci_status_is_cache_stale "$cache_file"; then
    ( ci_status_fetch ) &!
  fi
  local result
  result=$(cat "$cache_file" 2>/dev/null)
  CI_STATUS_PROMPT=$(ci_status_prompt_from_result "${result:-unknown}")
  if [[ -n "$CI_STATUS_PROMPT" ]]; then
    PROMPT="${PROMPT//$prompt_newline/ $CI_STATUS_PROMPT$prompt_newline}"
  fi
}

# Initialize GitHub hosts cache by running gh auth status in background
ci_status_init_gh_hosts_cache() {
  mkdir -p "$CI_STATUS_CACHE_DIR"

  # Run gh auth status in background and save active hosts to cache file
  (
    gh auth status --json hosts --jq '[.hosts | to_entries[] | .key as $host | .value[] | select(.active == true) | $host] | unique[]' 2>/dev/null > "$CI_STATUS_GH_HOSTS_CACHE_FILE" || true
  ) &!
}

if command -v gh >/dev/null 2>&1; then
  ci_status_init_gh_hosts_cache
  add-zsh-hook precmd precmd_ci_status
fi
