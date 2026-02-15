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

# Setup mocks - define functions first, then set function names in CI_STATUS_CTX
_test_mock_git_has_repo() { return 0 }
_test_mock_git_remote_url() { echo "https://github.com/owner/repo" }
_test_mock_git_toplevel_branch() { echo "/tmp/repo\nmain" }
_test_mock_gh_pr_view() { echo "ng" }
_test_mock_gh_pr_checks() { echo "ng\n\n\n0" }
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

# Assert: CI_STATUS_PROMPT should contain both PR red and checks red
assert_contains "$CI_STATUS_PROMPT" "%F{red}"

echo "✓ use-case-08 passed"
exit 0
