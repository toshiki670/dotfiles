#!/usr/bin/env zsh
# Use case 05: PR ok, no checks
# Condition: gh_pr_view returns "ok", gh_pr_checks returns empty checks_state
# Expected: CI_STATUS_PROMPT contains only PR green (no checks yellow/red/magenta)

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
CI_STATUS_CTX[gh_pr_checks]='() { echo "\n\n\n0" }'
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

# Assert: CI_STATUS_PROMPT should contain PR green but not checks colors
assert_contains "$CI_STATUS_PROMPT" "%F{green}"
assert_not_contains "$CI_STATUS_PROMPT" "%F{yellow}"
assert_not_contains "$CI_STATUS_PROMPT" "%F{red}"
assert_not_contains "$CI_STATUS_PROMPT" "%F{magenta}"

echo "✓ use-case-05 passed"
exit 0
