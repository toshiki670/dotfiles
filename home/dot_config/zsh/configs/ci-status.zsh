# Show remote CI status (GitHub Actions) in Pure prompt.
# Requires: gh CLI (https://cli.github.com/), jq, and GitHub repo.
# Status is fetched in the background and cached (default: 15 seconds).
(( ${+CI_STATUS_CACHE_SECONDS} )) || typeset -g CI_STATUS_CACHE_SECONDS=15

ci_status_repo_key() {
  local top
  top=$(git rev-parse --show-toplevel 2>/dev/null) || return 1
  local remote
  remote=$(git -C "$top" remote get-url origin 2>/dev/null) || return 1
  if [[ "$remote" =~ 'github.com[:/]([^/]+)/([^/]+)(\.git)?$' ]]; then
    echo "${match[1]}/${match[2]:r}"
    return 0
  fi
  return 1
}

ci_status_fetch() {
  local repo_key branch
  repo_key=$(ci_status_repo_key) || return 0
  branch=$(git branch --show-current 2>/dev/null) || return 0
  command -v gh >/dev/null 2>&1 || return 0
  command -v jq >/dev/null 2>&1 || return 0

  local cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/ci-status"
  mkdir -p "$cache_dir"
  local cache_file="$cache_dir/${repo_key//\//_}_${branch//\//_}.txt"

  local result
  result=$(gh run list -b "$branch" -L 1 --json conclusion,status 2>/dev/null | jq -r '
    if .[0] == null then "unknown"
    elif .[0].status != "completed" then "pending"
    elif .[0].conclusion == "success" then "success"
    elif .[0].conclusion == "failure" or .[0].conclusion == "cancelled" then "failure"
    elif .[0].conclusion == "skipped" then "skipped"
    else "unknown"
    end
  ')
  echo "${result:-unknown}" > "$cache_file"
}

precmd_ci_status() {
  [[ -z "$prompt_newline" ]] && return
  if ! git rev-parse --git-dir >/dev/null 2>&1; then
    CI_STATUS_PROMPT=""
    return
  fi
  local repo_key branch
  repo_key=$(ci_status_repo_key) || { CI_STATUS_PROMPT=""; return; }
  command -v gh >/dev/null 2>&1 || { CI_STATUS_PROMPT=""; return; }

  branch=$(git branch --show-current 2>/dev/null)
  local cache_dir="${XDG_CACHE_HOME:-$HOME/.cache}/ci-status"
  local cache_file="$cache_dir/${repo_key//\//_}_${branch//\//_}.txt"

  # Refresh in background if cache missing or older than CI_STATUS_CACHE_SECONDS
  local mtime=0
  if [[ -f "$cache_file" ]]; then
    if [[ "$(uname -s)" == Darwin ]]; then
      mtime=$(stat -f %m "$cache_file" 2>/dev/null)
    else
      mtime=$(stat -c %Y "$cache_file" 2>/dev/null)
    fi
  fi
  if (( mtime + CI_STATUS_CACHE_SECONDS < EPOCHSECONDS )); then
    ( ci_status_fetch ) &!
  fi

  if [[ -f "$cache_file" ]]; then
    case "$(cat "$cache_file")" in
      success) CI_STATUS_PROMPT='%F{green}✓%f' ;;
      failure) CI_STATUS_PROMPT='%F{red}✗%f' ;;
      pending) CI_STATUS_PROMPT='%F{yellow}◐%f' ;;
      skipped) CI_STATUS_PROMPT='%F{242}−%f' ;;
      *) CI_STATUS_PROMPT="" ;;
    esac
  else
    CI_STATUS_PROMPT=""
  fi

  if [[ -n "$CI_STATUS_PROMPT" ]]; then
    PROMPT="${PROMPT//$prompt_newline/ $CI_STATUS_PROMPT$prompt_newline}"
  fi
}

add-zsh-hook precmd precmd_ci_status
