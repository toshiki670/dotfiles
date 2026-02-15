#!/usr/bin/env zsh
# Use case 12: Async callback updates prompt
# Condition: async is available, gh_pr_view returns "ok", gh_pr_checks returns "ok\n\n\n0"
# Expected: CI_STATUS_PROMPT is set via callback and contains PR green + checks green

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
  echo "ok"
}
_test_mock_gh_pr_checks() {
  simulate_gh_delay
  echo "ok\n\n\n0"
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

# Setup prompt
prompt_newline="\n"
CI_STATUS_PROMPT=""
PROMPT="$prompt_newline"

# Execute
precmd_ci_status

# Wait for async jobs to complete
if (( $+functions[async_flush_jobs] )); then
  async_flush_jobs "ci_status"
else
  # If async is not available, wait a bit
  sleep 0.2
fi

# Assert: CI_STATUS_PROMPT should be set via callback and contain PR green + checks green
assert_contains "$CI_STATUS_PROMPT" "%F{green}"

echo "✓ use-case-12 passed"
exit 0
