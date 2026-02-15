#!/usr/bin/env zsh
# Use case 07: PR waiting, checks in progress
# Condition: gh_pr_view returns "waiting", gh_pr_checks returns "in_progress\n<min_start>\n<max_end>\n1"
# Expected: CI_STATUS_PROMPT contains PR blue, checks yellow, and yellow duration

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

# Setup mocks
CI_STATUS_CTX[git_has_repo]='() { return 0 }'
CI_STATUS_CTX[git_remote_url]='() { echo "https://github.com/owner/repo" }'
CI_STATUS_CTX[git_toplevel_branch]='() { echo "/tmp/repo\nmain" }'
CI_STATUS_CTX[gh_pr_view]='() { echo "waiting" }'
# Use a recent timestamp for in_progress test (30 seconds ago)
# Format: 2024-01-01T12:00:00Z
NOW_SEC=$(date +%s)
START_SEC=$((NOW_SEC - 30))
if [[ "$(uname)" == "Darwin" ]]; then
  START_TIME=$(date -u -j -f "%s" "$START_SEC" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo "2024-01-01T12:00:00Z")
else
  START_TIME=$(date -u -d "@$START_SEC" +%Y-%m-%dT%H:%M:%SZ)
fi
CI_STATUS_CTX[gh_pr_checks]="() { echo \"in_progress\n${START_TIME}\n\n1\" }"
CI_STATUS_CTX[gh_auth_status]='() { echo "" }'

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

echo "✓ use-case-07 passed"
exit 0
