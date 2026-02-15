#!/usr/bin/env zsh
# Use case 02: prompt_newline is empty
# Condition: prompt_newline is "", git_has_repo is true
# Expected: CI_STATUS_PROMPT is not updated (remains "")

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
CI_STATUS_CTX[gh_pr_view]='() { echo "ok" }'
CI_STATUS_CTX[gh_pr_checks]='() { echo "ok\n\n\n0" }'
CI_STATUS_CTX[gh_auth_status]='() { echo "" }'

# Setup gh_hosts_file
echo "github.com" > "${CI_STATUS_CTX[gh_hosts_file]}"

# Setup prompt (prompt_newline is empty)
prompt_newline=""
CI_STATUS_PROMPT=""
PROMPT="$prompt_newline"

# Execute
precmd_ci_status

# Assert: CI_STATUS_PROMPT should remain empty because prompt_newline is empty
assert_equals "$CI_STATUS_PROMPT" ""

echo "✓ use-case-02 passed"
exit 0
