# Show remote CI status (GitHub Actions) in Pure prompt.
# Requires: gh CLI (https://cli.github.com/), jq, and GitHub repo.
# Supports GitHub.com and GitHub Enterprise Server (on-premises); uses origin URL to detect host.
#
# Spec (when zsh-async is available):
# - Triggers: (1) terminal open, (2) Enter, (3) exec $SHELL -l.
# - On trigger: run CI check in background; when the job completes, show the checkmark (no second Enter).
# - When a trigger runs within CI_STATUS_CACHE_SECONDS (default 15s): skip fetch, show cached result when job completes.
# - Result is written only in the callback (job complete → PROMPT update + zle .reset-prompt).
(( ${+CI_STATUS_CACHE_SECONDS} )) || typeset -g CI_STATUS_CACHE_SECONDS=15

# Echo prompt string for status (success/failure/pending/skipped/unknown). Caller: CI_STATUS_PROMPT=$(ci_status_prompt_from_result "$result")
ci_status_prompt_from_result() {
  local -A m=(success '%F{green}✓%f' failure '%F{red}✗%f' pending '%F{yellow}◐%f' skipped '%F{242}−%f')
  echo "${m[$1]:-}"
}

# Succeeds if origin URL contains "github" (fast check; no gh auth call).
ci_status_gh_available() {
  local remote
  remote=$(git remote get-url origin 2>/dev/null) || return 1
  [[ "$remote" == *github* ]]
}

# Output mtime of file (seconds since epoch), or 0 if unavailable. Portable (Darwin/Linux).
ci_status_file_mtime() {
  local m=0
  if [[ -f "$1" ]]; then
    if [[ "$(uname -s)" == Darwin ]]; then
      m=$(stat -f %m "$1" 2>/dev/null)
    else
      m=$(stat -c %Y "$1" 2>/dev/null)
    fi
  fi
  echo "${m:-0}"
}

# Prints path to cache file: ~/.cache/ci-status/<path-under-HOME>/<branch>
# e.g. ~/.local/share/chezmoi + main -> ~/.cache/ci-status/.local/share/chezmoi/main
# e.g. ~/Repositories/github.com/owner/repo + main -> ~/.cache/ci-status/Repositories/github.com/owner/repo/main
ci_status_cache_file() {
  local abs_top suffix branch cache_dir
  abs_top=$(cd "$(git rev-parse --show-toplevel 2>/dev/null)" && pwd) || return 1
  branch=$(git branch --show-current 2>/dev/null) || return 1
  if [[ "$abs_top" == "$HOME"/* ]]; then
    suffix="${abs_top#$HOME/}"
  else
    suffix="${abs_top#/}"
  fi
  # branch may contain / (e.g. feature/foo) -> use as filename-safe part
  cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/ci-status"
  echo "$cache_dir/$suffix/${branch//\//_}"
}

ci_status_fetch() {
  ci_status_gh_available || return 0

  local cache_file
  cache_file=$(ci_status_cache_file) || return 0
  local branch
  branch=$(git branch --show-current 2>/dev/null) || return 0
  mkdir -p "${cache_file:h}"

  local result
  result=$(gh run list -b "$branch" -L 1 --json conclusion,status -q '
    if .[0] == null then "unknown"
    elif .[0].status != "completed" then "pending"
    elif .[0].conclusion == "success" then "success"
    elif .[0].conclusion == "failure" or .[0].conclusion == "cancelled" then "failure"
    elif .[0].conclusion == "skipped" then "skipped"
    else "unknown"
    end
  ' 2>/dev/null)
  echo "${result:-unknown}" > "$cache_file"
}

# Runs in zsh-async worker. If cache is older than CI_STATUS_CACHE_SECONDS, fetch; else use cache. Output "path\nstatus".
# Use date +%s because worker is a separate process and may not have EPOCHSECONDS (zsh/datetime).
ci_status_async_fetch() {
  local cache_file
  cache_file=$(ci_status_cache_file) || return 1
  local mtime now
  mtime=$(ci_status_file_mtime "$cache_file")
  now=$(date +%s 2>/dev/null) || now=0
  # 15s rule: only fetch when cache is stale (older than CI_STATUS_CACHE_SECONDS)
  if (( mtime + CI_STATUS_CACHE_SECONDS < now )); then
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

  local cache_file
  cache_file=$(ci_status_cache_file) || { CI_STATUS_PROMPT=""; return; }

  if (( $+functions[async_start_worker] )); then
    (( ${+ci_status_async_inited} )) || {
      typeset -g ci_status_async_inited=1
      async_start_worker "ci_status" -u -n
      async_register_callback "ci_status" ci_status_async_callback
    }
    async_worker_eval "ci_status" builtin cd -q $PWD
    async_job "ci_status" ci_status_async_fetch
  else
    local mtime
    mtime=$(ci_status_file_mtime "$cache_file")
    if (( mtime + CI_STATUS_CACHE_SECONDS < EPOCHSECONDS )); then
      ( ci_status_fetch ) &!
    fi
    CI_STATUS_PROMPT=$(ci_status_prompt_from_result "$(cat "$cache_file" 2>/dev/null)")
    if [[ -n "$CI_STATUS_PROMPT" ]]; then
      PROMPT="${PROMPT//$prompt_newline/ $CI_STATUS_PROMPT$prompt_newline}"
    fi
  fi
}

if command -v gh >/dev/null 2>&1; then
  add-zsh-hook precmd precmd_ci_status
fi
