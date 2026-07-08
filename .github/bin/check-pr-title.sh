#!/usr/bin/env bash
set -uo pipefail

title="${PR_TITLE:-}"

if [ -z "$title" ]; then
  printf 'ERROR: PR_TITLE is required.\n' >&2
  exit 1
fi

conventional_title='^(build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)(\([a-zA-Z0-9._/-]+\))?(!)?: .+'

if [[ "$title" =~ $conventional_title ]]; then
  echo "PR title is Conventional Commit compatible. OK"
  exit 0
fi

cat >&2 <<'MSG'
ERROR: PR title must use Conventional Commit format.

Accepted examples:
  feat: add watch PR column
  fix(daemon): avoid startup race
  chore!: remove legacy config

Allowed types:
  build, chore, ci, docs, feat, fix, perf, refactor, revert, style, test
MSG
printf '\nActual title:\n  %s\n' "$title" >&2
exit 1
