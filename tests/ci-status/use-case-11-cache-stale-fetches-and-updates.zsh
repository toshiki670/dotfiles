#!/usr/bin/env zsh
# Use case 11: Cache stale fetches and updates
# Condition: Cache file is missing or mtime is old (stale), gh_pr_view returns "ng", gh_pr_checks returns "ng\n\n\n0"
# Expected: CI_STATUS_PROMPT contains PR red + checks red, cache is updated to "ng,ng,"

set -e

# Get repo root and source helper
SCRIPT_DIR="${0:a:h}"
REPO_ROOT="${SCRIPT_DIR:h:h}"
source "$SCRIPT_DIR/helper.zsh"

# Source ci-status.zsh first (to initialize CI_STATUS_CTX)
source_ci_status

# Setup test environment (after CI_STATUS_CTX is initialized)
setup_test_env
trap cleanup_test_env EXIT

# Setup mocks - define functions first, then set function names in CI_STATUS_CTX
_test_mock_git_has_repo() { return 0 }
_test_mock_git_remote_url() { echo "https://github.com/owner/repo" }
_test_mock_git_toplevel_branch() { echo "/tmp/repo\nmain" }
_test_mock_gh_pr_view() {
  simulate_gh_delay
  echo "ng"
}
_test_mock_gh_pr_checks() {
  simulate_gh_delay
  echo "ng\n\n\n0"
}
_test_mock_gh_auth_status() { echo "" }

CI_STATUS_CTX[git_has_repo]="_test_mock_git_has_repo"
CI_STATUS_CTX[git_remote_url]="_test_mock_git_remote_url"
CI_STATUS_CTX[git_toplevel_branch]="_test_mock_git_toplevel_branch"
CI_STATUS_CTX[gh_pr_view]="_test_mock_gh_pr_view"
CI_STATUS_CTX[gh_pr_checks]="_test_mock_gh_pr_checks"
CI_STATUS_CTX[gh_auth_status]="_test_mock_gh_auth_status"

# Setup gh_hosts_file
echo "github.com" > "${CI_STATUS_CTX[gh_hosts_file]}"

# Create stale cache file (or leave it missing)
# git_toplevel_branch returns "/tmp/repo\nmain", which becomes "/tmp/repo/main" after join
# Then "/" is replaced with "_" to get "_tmp_repo_main"
CACHE_FILE="${CI_STATUS_CTX[cache_dir]}/repos/_tmp_repo_main"
mkdir -p "${CACHE_FILE:h}"
echo "old,old," > "$CACHE_FILE"
# Make it stale by setting mtime to past
touch -t 202001010000 "$CACHE_FILE" 2>/dev/null || true

# Setup prompt
prompt_newline="\n"
CI_STATUS_PROMPT=""
PROMPT="$prompt_newline"

# Execute
precmd_ci_status
wait_for_async

# Assert: CI_STATUS_PROMPT should contain PR red and checks red
assert_contains "$CI_STATUS_PROMPT" "%F{red}"

# Assert: Cache should be updated
# Wait a bit more for async job to complete and write cache
sleep 0.2
if [[ -f "$CACHE_FILE" ]]; then
  CACHE_CONTENT=$(cat "$CACHE_FILE")
  assert_contains "$CACHE_CONTENT" "ng"
fi

echo "✓ use-case-11 passed"
exit 0
