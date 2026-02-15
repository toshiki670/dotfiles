#!/usr/bin/env zsh
# Use case 05: PR ok, no checks
# Condition: gh_pr_view returns "ok", gh_pr_checks returns empty checks_state
# Expected: CI_STATUS_PROMPT contains only PR green (no checks yellow/red/magenta)

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
  echo "\n\n\n0"
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
wait_for_async

# Assert: CI_STATUS_PROMPT should contain PR green but not checks colors
assert_contains "$CI_STATUS_PROMPT" "%F{green}"
assert_not_contains "$CI_STATUS_PROMPT" "%F{yellow}"
assert_not_contains "$CI_STATUS_PROMPT" "%F{red}"
assert_not_contains "$CI_STATUS_PROMPT" "%F{magenta}"

echo "✓ use-case-05 passed"
exit 0
