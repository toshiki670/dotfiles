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

# Setup mocks - define functions first, then set function names in CI_STATUS_CTX
_test_mock_git_has_repo() { return 0 }
_test_mock_git_remote_url() { echo "https://github.com/owner/repo" }
_test_mock_git_toplevel_branch() { echo "/tmp/repo\nmain" }
_test_mock_gh_pr_view() { echo "ok" }
_test_mock_gh_pr_checks() { echo "ok\n\n\n0" }
_test_mock_gh_auth_status() { echo "" }

CI_STATUS_CTX[git_has_repo]="_test_mock_git_has_repo"
CI_STATUS_CTX[git_remote_url]="_test_mock_git_remote_url"
CI_STATUS_CTX[git_toplevel_branch]="_test_mock_git_toplevel_branch"
CI_STATUS_CTX[gh_pr_view]="_test_mock_gh_pr_view"
CI_STATUS_CTX[gh_pr_checks]="_test_mock_gh_pr_checks"
CI_STATUS_CTX[gh_auth_status]="_test_mock_gh_auth_status"

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
