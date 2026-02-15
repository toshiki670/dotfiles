#!/usr/bin/env zsh
# Run all ci-status E2E tests in parallel

set -e

# Get script directory and repo root
SCRIPT_DIR="${0:a:h}"
REPO_ROOT="${SCRIPT_DIR:h:h}"

# Change to repo root
cd "$REPO_ROOT"

# Find all use-case files and sort them
USE_CASES=("${SCRIPT_DIR}"/use-case-*.zsh)
USE_CASES=(${(o)USE_CASES})

if [[ ${#USE_CASES} -eq 0 ]]; then
  echo "No use-case files found" >&2
  exit 1
fi

echo "Found ${#USE_CASES} use-case files"
echo "Running tests in parallel..."

# Run all use-cases in parallel
declare -a PIDS=()
declare -a NAMES=()
FAILED=0

for use_case in "${USE_CASES[@]}"; do
  use_case_name="${use_case:t}"
  echo "Starting: $use_case_name"
  zsh "$use_case" &
  PIDS+=($!)
  NAMES+=("$use_case_name")
done

# Wait for all jobs and collect results
FAILED_NAMES=()
for i in {1..${#PIDS}}; do
  pid=${PIDS[$i]}
  name=${NAMES[$i]}
  wait $pid
  exit_code=$?
  if [[ $exit_code -ne 0 ]]; then
    FAILED=$((FAILED + 1))
    FAILED_NAMES+=("$name")
    echo "✗ $name failed (exit code: $exit_code)" >&2
  else
    echo "✓ $name passed"
  fi
done

# Report results
echo ""
if [[ $FAILED -eq 0 ]]; then
  echo "All ${#USE_CASES} use cases passed"
  exit 0
else
  echo "$FAILED of ${#USE_CASES} use cases failed:"
  for name in "${FAILED_NAMES[@]}"; do
    echo "  - $name" >&2
  done
  exit 1
fi
