#!/usr/bin/env zsh
# Use case 10: Cache fresh returns cached
# Condition: Cache file exists with content "ok,ok," and mtime is recent (fresh)
# Expected: CI_STATUS_PROMPT is based on cache (PR green + checks green), gh is not called

set -e

# Get repo root and source helper
SCRIPT_DIR="${0:a:h}"
REPO_ROOT="${SCRIPT_DIR:h:h}"
source "$SCRIPT_DIR/helper.zsh"

# Setup test environment
setup_test_env
trap cleanup_test_env EXIT

# Source ci-status.zsh
source_ci_status

# Setup mocks (these should not be called if cache is fresh)
CI_STATUS_CTX[git_has_repo]='() { return 0 }'
CI_STATUS_CTX[git_remote_url]='() { echo "https://github.com/owner/repo" }'
CI_STATUS_CTX[git_toplevel_branch]='() { echo "/tmp/repo\nmain" }'
CI_STATUS_CTX[gh_pr_view]='() { echo "ERROR: should not be called" >&2; echo "" }'
CI_STATUS_CTX[gh_pr_checks]='() { echo "ERROR: should not be called" >&2; echo "" }'
CI_STATUS_CTX[gh_auth_status]='() { echo "" }'

# Setup gh_hosts_file
echo "github.com" > "${CI_STATUS_CTX[gh_hosts_file]}"

# Create cache file with fresh content
CACHE_FILE="${CI_STATUS_CTX[cache_dir]}/repos/tmp_repo_main"
mkdir -p "${CACHE_FILE:h}"
echo "ok,ok," > "$CACHE_FILE"
# Touch to make it fresh (within cache_seconds)
touch "$CACHE_FILE"

# Setup prompt
prompt_newline="\n"
CI_STATUS_PROMPT=""
PROMPT="$prompt_newline"

# Execute
precmd_ci_status
wait_for_async

# Assert: CI_STATUS_PROMPT should contain PR green and checks green from cache
assert_contains "$CI_STATUS_PROMPT" "%F{green}"

echo "✓ use-case-10 passed"
exit 0
