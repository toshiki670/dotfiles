# Show remote CI status (GitHub Actions) in Pure prompt.
# Requires: gh CLI (https://cli.github.com/), jq, and GitHub repo.
# Supports GitHub.com and GitHub Enterprise Server (on-premises); uses origin URL to detect host.
# Status is fetched in the background and cached (default: 15 seconds).
(( ${+CI_STATUS_CACHE_SECONDS} )) || typeset -g CI_STATUS_CACHE_SECONDS=15

# Succeeds if origin host is one where gh is authenticated (inferred from remote URL).
ci_status_gh_available() {
  local remote host
  remote=$(git remote get-url origin 2>/dev/null) || return 1
  if [[ "$remote" =~ ^https://([^/]+)/ ]]; then
    host="${match[1]}"
  elif [[ "$remote" =~ ^git@([^:]+): ]]; then
    host="${match[1]}"
  else
    return 1
  fi
  gh auth status -h "$host" &>/dev/null
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
  command -v gh >/dev/null 2>&1 || return 0
  command -v jq >/dev/null 2>&1 || return 0

  local cache_file
  cache_file=$(ci_status_cache_file) || return 0
  local branch
  branch=$(git branch --show-current 2>/dev/null) || return 0
  mkdir -p "${cache_file:h}"

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
  command -v gh >/dev/null 2>&1 || { CI_STATUS_PROMPT=""; return; }

  local cache_file
  cache_file=$(ci_status_cache_file) || { CI_STATUS_PROMPT=""; return; }

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
