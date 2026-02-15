#!/usr/bin/env zsh
# Use case 03: GitHub host unavailable
# Condition: gh_hosts_file is empty, or git_remote_url returns host not in cache
# Expected: CI_STATUS_PROMPT is ""

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
CI_STATUS_CTX[git_remote_url]='() { echo "https://unknown-host.com/owner/repo" }'
CI_STATUS_CTX[git_toplevel_branch]='() { echo "/tmp/repo\nmain" }'
CI_STATUS_CTX[gh_pr_view]='() { echo "ok" }'
CI_STATUS_CTX[gh_pr_checks]='() { echo "ok\n\n\n0" }'
CI_STATUS_CTX[gh_auth_status]='() { echo "" }'

# Setup gh_hosts_file (empty or different host)
echo "github.com" > "${CI_STATUS_CTX[gh_hosts_file]}"

# Setup prompt
prompt_newline="\n"
CI_STATUS_PROMPT=""
PROMPT="$prompt_newline"

# Execute
precmd_ci_status
wait_for_async

# Assert: CI_STATUS_PROMPT should be empty because host is unavailable
assert_equals "$CI_STATUS_PROMPT" ""

echo "✓ use-case-03 passed"
exit 0
