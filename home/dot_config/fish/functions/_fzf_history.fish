function _fzf_history
  set -l line (history | fzf +s --tiebreak=index)
  if test -n "$line"
    commandline -r "$line"
  end
  commandline -f repaint
end
