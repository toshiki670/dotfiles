#!/usr/bin/env zsh
# Use case 08: PR ng, checks ng
# Condition: gh_pr_view returns "ng", gh_pr_checks returns "ng\n\n\n0"
# Expected: CI_STATUS_PROMPT contains both PR red and checks red

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
CI_STATUS_CTX[gh_pr_view]='() { echo "ng" }'
CI_STATUS_CTX[gh_pr_checks]='() { echo "ng\n\n\n0" }'
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

# Assert: CI_STATUS_PROMPT should contain both PR red and checks red
assert_contains "$CI_STATUS_PROMPT" "%F{red}"

echo "✓ use-case-08 passed"
exit 0
