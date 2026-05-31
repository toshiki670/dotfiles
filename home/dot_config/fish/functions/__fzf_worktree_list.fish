function __fzf_worktree_list --argument-names repo
    git -C "$repo" worktree list --porcelain 2>/dev/null | awk -v OFS='\t' '
        function flush() {
            if (path == "") return
            label = (branch != "") ? branch : (det ? "(detached)" : "")
            print (idx == 0 ? 1 : 0), path, label
            idx++; path = ""; branch = ""; det = 0
        }
        BEGIN { idx = 0; path = ""; branch = ""; det = 0 }
        /^worktree / { flush(); path = substr($0, 10); next }
        /^branch refs\/heads\// { branch = substr($0, 19); next }
        /^detached/ { det = 1; next }
        END { flush() }
    '
end
