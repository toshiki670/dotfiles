# Helper functions for ci-status E2E tests

# Assert that a string contains a substring
# Usage: assert_contains "$actual" "$expected_substring"
assert_contains() {
  local actual=$1 expected=$2
  if [[ "$actual" != *"$expected"* ]]; then
    echo "Assertion failed: expected '$actual' to contain '$expected'" >&2
    return 1
  fi
}

# Assert that a string does not contain a substring
# Usage: assert_not_contains "$actual" "$unexpected_substring"
assert_not_contains() {
  local actual=$1 unexpected=$2
  if [[ "$actual" == *"$unexpected"* ]]; then
    echo "Assertion failed: expected '$actual' not to contain '$unexpected'" >&2
    return 1
  fi
}

# Assert that a string equals expected value
# Usage: assert_equals "$actual" "$expected"
assert_equals() {
  local actual=$1 expected=$2
  if [[ "$actual" != "$expected" ]]; then
    echo "Assertion failed: expected '$expected', got '$actual'" >&2
    return 1
  fi
}

# Setup test environment: create temp dir and reset CI_STATUS_CTX paths
# Usage: setup_test_env
# Sets: TEST_TMPDIR (exported)
# Note: CI_STATUS_CTX must be initialized (via source_ci_status) before calling this
setup_test_env() {
  export TEST_TMPDIR=$(mktemp -d)
  # Ensure CI_STATUS_CTX is initialized as associative array
  if (( ! ${+CI_STATUS_CTX} )); then
    typeset -gA CI_STATUS_CTX
  fi
  CI_STATUS_CTX[cache_dir]="$TEST_TMPDIR/cache"
  CI_STATUS_CTX[gh_hosts_file]="$TEST_TMPDIR/gh_hosts"
  CI_STATUS_CTX[error_log_file]="$TEST_TMPDIR/error.log"
  mkdir -p "${CI_STATUS_CTX[cache_dir]}"
}

# Cleanup test environment
# Usage: cleanup_test_env
cleanup_test_env() {
  if [[ -n "$TEST_TMPDIR" ]] && [[ -d "$TEST_TMPDIR" ]]; then
    rm -rf "$TEST_TMPDIR"
  fi
}

# Get repository root (parent of tests/ci-status)
# Usage: repo_root=$(get_repo_root)
# Note: This function must be called from a script file, not interactively
get_repo_root() {
  # Use ${(%):-%x} to get the script file path even when called from a function
  local script_file="${(%):-%x}"
  local script_dir="${script_file:a:h}"
  echo "${script_dir:h:h}"
}

# Source ci-status.zsh with proper setup
# Usage: source_ci_status
source_ci_status() {
  # Get script directory from the calling script
  local calling_script="${(%):-%x}"
  local script_dir="${calling_script:a:h}"
  local repo_root="${script_dir:h:h}"
  local ci_status_file="$repo_root/home/dot_config/zsh/configs/ci-status.zsh"
  
  # Mock add-zsh-hook if not available (for test environment)
  if ! (( $+functions[add-zsh-hook] )); then
    add-zsh-hook() {
      # No-op for testing
      :
    }
  fi
  
  # Source zsh-async if available
  if [[ -f "$repo_root/home/dot_config/zsh/configs/zsh-async/async.zsh" ]]; then
    source "$repo_root/home/dot_config/zsh/configs/zsh-async/async.zsh"
  elif command -v async_start_worker >/dev/null 2>&1; then
    # zsh-async is already loaded
    :
  fi
  
  # Source ci-status.zsh
  source "$ci_status_file"
}

# Wait for async jobs to complete
# Usage: wait_for_async
wait_for_async() {
  if (( $+functions[async_flush_jobs] )); then
    async_flush_jobs "ci_status"
  else
    # If async is not available, wait a bit for background jobs
    sleep 0.1
  fi
}
