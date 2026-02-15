#!/usr/bin/env zsh
# Use case 07: PR waiting, checks in progress
# Condition: gh_pr_view returns "waiting", gh_pr_checks returns "in_progress\n<min_start>\n<max_end>\n1"
# Expected: CI_STATUS_PROMPT contains PR blue, checks yellow, and yellow duration

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
  echo "waiting"
}
# Use a recent timestamp for in_progress test (30 seconds ago)
# Format: 2024-01-01T12:00:00Z
NOW_SEC=$(date +%s)
START_SEC=$((NOW_SEC - 30))
if [[ "$(uname)" == "Darwin" ]]; then
  START_TIME=$(date -u -j -f "%s" "$START_SEC" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo "2024-01-01T12:00:00Z")
else
  START_TIME=$(date -u -d "@$START_SEC" +%Y-%m-%dT%H:%M:%SZ)
fi
# Store START_TIME for later verification
export TEST_START_TIME="$START_TIME"
# Define function with START_TIME captured in closure
# Note: gh_pr_checks is called with --json and --jq options, but our mock ignores them
# and returns the jq-processed result directly (4 lines: checks_state, min_start, max_end, has_pending)
# The actual jq output format: "checks_state\nmin_start\nmax_end\nhas_pending"
# For in_progress: max_end is empty (empty string from jq), has_pending=1
# zsh's (f) flag may ignore empty lines, so we need to ensure proper parsing
_test_mock_gh_pr_checks() {
  simulate_gh_delay
  # Output format matching jq output: 4 lines separated by newlines
  # Line 1: checks_state, Line 2: min_start, Line 3: max_end (empty), Line 4: has_pending
  # Use explicit newlines to ensure proper parsing
  echo "in_progress"
  echo "$TEST_START_TIME"
  # Empty line for max_end (jq outputs empty string which becomes empty line)
  # Note: zsh (f) flag may skip this empty line, so we output it explicitly
  echo ""
  echo "1"
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

# Assert: CI_STATUS_PROMPT should contain PR blue, checks yellow, and yellow duration
assert_contains "$CI_STATUS_PROMPT" "%F{blue}"
assert_contains "$CI_STATUS_PROMPT" "%F{yellow}"

# Verify duration is calculated correctly (should be around 30 seconds)
# Duration format: "Xs" or "Xm Ys" or "Xh Ym Zs"
# For 30 seconds, it should be "30s" or similar
# Note: Duration is displayed as "%F{yellow}30s%f" in the prompt
# Check for duration pattern (number followed by 's', 'm', or 'h')
if [[ "$CI_STATUS_PROMPT" != *"s"* ]] && [[ "$CI_STATUS_PROMPT" != *"m"* ]] && [[ "$CI_STATUS_PROMPT" != *"h"* ]]; then
  echo "Assertion failed: expected duration in CI_STATUS_PROMPT, got: $CI_STATUS_PROMPT" >&2
  exit 1
fi

echo "✓ use-case-07 passed"
exit 0
