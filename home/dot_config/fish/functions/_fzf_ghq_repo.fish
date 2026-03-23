function _fzf_ghq_repo
  if not command -v ghq >/dev/null 2>&1
    echo "ghq: command not found"
    commandline -f repaint
    return 1
  end
  set -l rel (
    ghq list 2>/dev/null | fzf \
      --preview 'fish -c "__fzf_ghq_readme_preview {}"' \
      --preview-window=right:60%
  )
  if test -n "$rel"
    set rel (string trim "$rel")
    set -l root (ghq root)
    cd "$root/$rel"
    commandline -f repaint
  end
end
