# cdabbr - cd by expanding prompt_pwd-style abbreviated path
# Usage: cdabbr ~/R/g/t/dotfiles
# - Recursively expands each segment (same initial → branch and collect all matches).
# - 1 match → cd immediately; 2+ matches → fzf to choose, then cd.

function _cdabbr_expand_recursive
  set -l base "$argv[1]"
  set -l segments $argv[2..-1]

  if test (count $segments) -eq 0
    echo "$base"
    return
  end

  set -l seg "$segments[1]"
  set -l rest $segments[2..-1]

  set -l subdirs (find "$base" -maxdepth 1 -mindepth 1 -type d 2>/dev/null)
  for d in $subdirs
    set -l name (path basename "$d")
    if string match -q "$seg*" "$name"
      _cdabbr_expand_recursive "$d" $rest
    end
  end
end

function cdabbr --description 'cd by expanding prompt_pwd-style abbreviated path'
  set -l abbr_path "$argv[1]"
  if test -z "$abbr_path"
    echo "usage: cdabbr <abbreviated-path>" >&2
    return 1
  end

  set -l base
  set -l segments (string split '/' "$abbr_path")

  if string match -q '~*' "$abbr_path"
    set base $HOME
    set -e segments[1]
  else if string match -q '/*' "$abbr_path"
    set base "/"
    set -e segments[1]
  else
    echo "cdabbr: path must start with ~ or /" >&2
    return 1
  end

  set -l segs
  for s in $segments
    if test -n "$s"
      set segs $segs $s
    end
  end

  set -l candidates (_cdabbr_expand_recursive "$base" $segs | string trim)

  if test (count $candidates) -eq 0
    echo "cdabbr: no matching path for '$abbr_path'" >&2
    return 1
  end

  if test (count $candidates) -eq 1
    cd "$candidates[1]"
    return
  end

  if not command -q fzf
    echo "cdabbr: multiple matches (install fzf to choose):" >&2
    for c in $candidates
      echo "  $c" >&2
    end
    return 1
  end

  set -l result (
    string join \n $candidates \
    | fzf --height=40% \
          --reverse \
          --preview='eza -1 --color=always --icons {}' \
          --preview-window='right:50%:wrap' \
          --bind='ctrl-/:toggle-preview'
  )
  if test -n "$result"
    cd "$result"
  end
end

abbr -a ca 'cdabbr '
