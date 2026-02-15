#!/usr/bin/env zsh
# Use case 12: Async callback updates prompt
# Condition: async is available, gh_pr_view returns "ok", gh_pr_checks returns "ok\n\n\n0"
# Expected: CI_STATUS_PROMPT is set via callback and contains PR green + checks green

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
