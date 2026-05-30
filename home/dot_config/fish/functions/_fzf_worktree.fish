function _fzf_worktree
    if not git rev-parse --is-inside-work-tree >/dev/null 2>&1
        echo "not in a git repository"
        commandline -f repaint
        return 1
    end

    set -l tab (printf '\t')
    set -l selection (
        git worktree list --porcelain | awk -v OFS='\t' '
            function flush() {
                if (path == "") return
                label = (branch != "") ? branch : (det ? "(detached)" : "")
                if (idx == 0) label = "* " label
                print label, path
                idx++
                path = ""; branch = ""; det = 0
            }
            BEGIN { idx = 0; path = ""; branch = ""; det = 0 }
            /^worktree / { flush(); path = substr($0, 10); next }
            /^branch refs\/heads\// { branch = substr($0, 19); next }
            /^detached/ { det = 1; next }
            END { flush() }
        ' | fzf \
            --delimiter=$tab \
            --with-nth=1 \
            --preview "git -C {2} log --oneline -20" \
            --preview-window=right:60%
    )

    if test -n "$selection"
        set -l parts (string split -m 1 -- $tab $selection)
        cd $parts[2]
    end
    commandline -f repaint
end
