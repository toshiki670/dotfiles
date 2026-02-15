#!/usr/bin/env zsh
# Use case 13: Checks completed - verify duration calculation
# Condition: gh_pr_view returns "ok", gh_pr_checks returns completed checks with min_start and max_end
# Expected: CI_STATUS_PROMPT contains duration calculated from max_end - min_start

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
# Create timestamps for completed CI: started 2 minutes ago, completed 1 minute ago
# Duration should be: 2 minutes - 1 minute = 1 minute
NOW_SEC=$(date +%s)
START_SEC=$((NOW_SEC - 120))  # 2 minutes ago
END_SEC=$((NOW_SEC - 60))     # 1 minute ago
if [[ "$(uname)" == "Darwin" ]]; then
  START_TIME=$(date -u -j -f "%s" "$START_SEC" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo "2024-01-01T12:00:00Z")
  END_TIME=$(date -u -j -f "%s" "$END_SEC" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo "2024-01-01T12:01:00Z")
else
  START_TIME=$(date -u -d "@$START_SEC" +%Y-%m-%dT%H:%M:%SZ)
  END_TIME=$(date -u -d "@$END_SEC" +%Y-%m-%dT%H:%M:%SZ)
fi
# Store for verification
export TEST_START_TIME="$START_TIME"
export TEST_END_TIME="$END_TIME"
export TEST_EXPECTED_DURATION_SEC=60  # 1 minute = 60 seconds
_test_mock_gh_pr_checks() {
  simulate_gh_delay
  # Format: checks_state\nmin_start\nmax_end\nhas_pending
  # For completed: has_pending=0, both min_start and max_end are set
  echo "ok\n${START_TIME}\n${END_TIME}\n0"
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

# Assert: CI_STATUS_PROMPT should contain PR green and checks green
assert_contains "$CI_STATUS_PROMPT" "%F{green}"

# Assert: Duration should be calculated from max_end - min_start (approximately 1 minute)
# Duration format: "1m" or "1m 0s" or "60s"
assert_contains "$CI_STATUS_PROMPT" "m"

echo "✓ use-case-13 passed"
exit 0
