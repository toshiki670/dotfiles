#!/usr/bin/env bash
set -euo pipefail

mode="${1:-}"
if [[ "${mode}" != "fix" && "${mode}" != "check" ]]; then
  echo "Usage: $0 {fix|check}" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

tmp_dir="$(mktemp -d)"
cleanup() { rm -rf "${tmp_dir}"; }
trap cleanup EXIT

list_files() {
  git rev-parse --is-inside-work-tree >/dev/null 2>&1
  git ls-files
}

is_markdown() { [[ "$1" == *.md ]]; }
is_lua() { [[ "$1" == *.lua ]]; }
is_toml() { [[ "$1" == *.toml ]]; }
is_fish() { [[ "$1" == *.fish || "$1" == *.fish.tmpl ]]; }
is_zsh() { [[ "$1" == *.zsh || "$1" == *.zsh.tmpl ]]; }
is_shell_ext() { [[ "$1" == *.sh || "$1" == *.sh.tmpl ]]; }
is_shell_path() { [[ "$1" == bin/* || "$1" == bash/* ]]; }

shebang() {
  local f="$1"
  [[ -f "$f" ]] || return 1
  IFS= read -r line <"$f" || true
  printf '%s' "$line"
}

shell_flavor() {
  local sb
  sb="$(shebang "$1" || true)"
  case "$sb" in
    "#!"*bash*) echo "bash" ;;
    "#!"*sh) echo "sh" ;;
    "#!"*zsh*) echo "zsh" ;;
    *) echo "" ;;
  esac
}

run_markdown_fix() {
  local f="$1"
  markdownlint-cli2-fix "$f"
}

fix_fish_inplace() {
  local f="$1"
  local out="${tmp_dir}/fish.$(printf '%s' "$f" | tr '/.' '__')"
  fish_indent "$f" >"$out"
  if ! cmp -s "$f" "$out"; then
    mv "$out" "$f"
  fi
}

files=()
while IFS= read -r f; do
  [[ -n "$f" ]] || continue
  [[ -f "$f" ]] || continue
  files+=("$f")
done < <(list_files)

if [[ "$mode" == "fix" ]]; then
  for f in "${files[@]}"; do
    if is_shell_ext "$f" || is_shell_path "$f"; then
      sf="$(shell_flavor "$f")"
      if [[ "$sf" == "bash" || "$sf" == "sh" || -z "$sf" ]]; then
        shfmt -w -i 2 -ci "$f" || true
      fi
    fi
  done

  for f in "${files[@]}"; do
    if is_lua "$f"; then
      stylua "$f"
    elif is_toml "$f"; then
      taplo fmt "$f"
    elif is_markdown "$f"; then
      run_markdown_fix "$f"
    elif is_fish "$f"; then
      fix_fish_inplace "$f"
    fi
  done
fi

failed=0

for f in "${files[@]}"; do
  if is_shell_ext "$f" || is_shell_path "$f"; then
    sf="$(shell_flavor "$f")"
    if [[ "$sf" == "bash" || "$sf" == "sh" || -z "$sf" ]]; then
      shfmt -d -i 2 -ci "$f" >/dev/null || failed=1
      shellcheck "$f" || failed=1
    elif [[ "$sf" == "zsh" ]]; then
      zsh -n "$f" || failed=1
    fi
  fi

  if is_zsh "$f"; then
    zsh -n "$f" || failed=1
  fi

  if is_fish "$f"; then
    fish_indent --check "$f" || failed=1
    fish --no-execute "$f" || failed=1
  fi

  if is_lua "$f"; then
    stylua --check "$f" || failed=1
  fi

  if is_toml "$f"; then
    taplo fmt --check "$f" || failed=1
    taplo lint "$f" || failed=1
  fi

  if is_markdown "$f"; then
    markdownlint-cli2 "$f" || failed=1
  fi
done

exit "$failed"
