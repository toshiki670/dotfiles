#!/usr/bin/env bash
set -euo pipefail

mode="${1:-}"
if [[ "${mode}" != "fix" && "${mode}" != "check" ]]; then
  echo "Usage: $0 {fix|check}" >&2
  exit 2
fi

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

list_files() {
  git ls-files
}

is_markdown() { [[ "$1" == *.md ]]; }
is_lua() { [[ "$1" == *.lua ]]; }
is_toml() { [[ "$1" == *.toml ]]; }
is_fish() { [[ "$1" == *.fish || "$1" == *.fish.tmpl ]]; }
is_zsh() { [[ "$1" == *.zsh || "$1" == *.zsh.tmpl ]]; }
is_shell_ext() { [[ "$1" == *.sh || "$1" == *.sh.tmpl ]]; }
is_shell_path() { [[ "$1" == bin/* || "$1" == bash/* ]]; }

# Chezmoi source root for this repo (.chezmoiroot names the "home" subtree).
chezmoi_source_dir() { printf '%s/home' "${repo_root}"; }

# home/*.sh.tmpl with Go template syntax — shellcheck/shfmt must not parse raw file.
is_home_chezmoi_shell_template() {
  local f="$1"
  [[ "$f" == home/*.sh.tmpl ]] || return 1
  grep -qE '\{\{' "$f" 2>/dev/null
}

# Render template to a temp file; exit non-zero if chezmoi fails.
chezmoi_render_shell_template() {
  local f="$1"
  local out="$2"
  chezmoi -S "$(chezmoi_source_dir)" execute-template -f "${repo_root}/${f}" >"$out" 2>"${tmp_dir}/chezmoi.err" ||
    {
      echo "lint: chezmoi execute-template failed: ${f}" >&2
      cat "${tmp_dir}/chezmoi.err" >&2
      return 1
    }
}

shellcheck_shell_file() {
  local f="$1"
  local rendered
  if is_home_chezmoi_shell_template "$f"; then
    rendered="${tmp_dir}/shellcheck.$(printf '%s' "$f" | tr '/.' '__')"
    chezmoi_render_shell_template "$f" "$rendered" || return 1
    shellcheck -s bash "$rendered" || {
      echo "lint: shellcheck failed on expanded template (source: ${f})" >&2
      return 1
    }
  else
    shellcheck "$f" || return 1
  fi
}

shebang() {
  local f="$1"
  IFS= read -r line <"$f"
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

fix_fish_inplace() {
  local f="$1"
  local out
  out="${tmp_dir}/fish.$(printf '%s' "$f" | tr '/.' '__')"
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
        if ! is_home_chezmoi_shell_template "$f"; then
          shfmt -w -i 2 -ci "$f"
        fi
      fi
    fi
  done

  for f in "${files[@]}"; do
    if is_lua "$f"; then
      stylua "$f"
    elif is_toml "$f"; then
      taplo fmt "$f"
    elif is_markdown "$f"; then
      markdownlint-cli2 --fix ":$f"
    elif is_fish "$f"; then
      fix_fish_inplace "$f"
    fi
  done

  if git diff --quiet -- "${files[@]}"; then
    echo "lint(fix): no auto-fix changes"
  else
    echo "lint(fix): auto-fix updated files"
    git status --short -- "${files[@]}"
  fi
fi

failed=0

for f in "${files[@]}"; do
  if is_shell_ext "$f" || is_shell_path "$f"; then
    sf="$(shell_flavor "$f")"
    if [[ "$sf" == "bash" || "$sf" == "sh" || -z "$sf" ]]; then
      if ! is_home_chezmoi_shell_template "$f"; then
        shfmt -d -i 2 -ci "$f" >/dev/null || failed=1
      fi
      shellcheck_shell_file "$f" || failed=1
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
    markdownlint-cli2 ":$f" || failed=1
  fi
done

exit "$failed"
