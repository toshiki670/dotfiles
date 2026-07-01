function _fzf_file
    set -l selected (
    find . -path '*/\.*' -prune -o -type f -print 2>/dev/null | fzf
  )
    if test -n "$selected"
        commandline -i "$selected"
    end
    commandline -f repaint
end
