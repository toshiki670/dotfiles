#!/usr/bin/env zsh
# Use case 01: Not a git repository
# Condition: git_has_repo returns false
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

# Setup mocks - define functions first, then set function names in CI_STATUS_CTX
_test_mock_git_has_repo() { return 1 }
_test_mock_git_remote_url() { echo "https://github.com/owner/repo" }
_test_mock_git_toplevel_branch() { echo "/tmp/repo\nmain" }
_test_mock_gh_pr_view() { echo "" }
_test_mock_gh_pr_checks() { echo "" }
_test_mock_gh_auth_status() { echo "" }

CI_STATUS_CTX[git_has_repo]="_test_mock_git_has_repo"
CI_STATUS_CTX[git_remote_url]="_test_mock_git_remote_url"
CI_STATUS_CTX[git_toplevel_branch]="_test_mock_git_toplevel_branch"
CI_STATUS_CTX[gh_pr_view]="_test_mock_gh_pr_view"
CI_STATUS_CTX[gh_pr_checks]="_test_mock_gh_pr_checks"
CI_STATUS_CTX[gh_auth_status]="_test_mock_gh_auth_status"

# Setup prompt
prompt_newline="\n"
CI_STATUS_PROMPT=""
PROMPT="$prompt_newline"

# Execute
precmd_ci_status

# Assert
assert_equals "$CI_STATUS_PROMPT" ""

echo "✓ use-case-01 passed"
exit 0
