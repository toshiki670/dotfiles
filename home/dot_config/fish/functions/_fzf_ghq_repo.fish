function _fzf_ghq_repo
    if not command -v ghq >/dev/null 2>&1
        echo "ghq: command not found"
        return 1
    end

    set -l tab (printf '\t')
    set -l root (ghq root)

    set -l lines
    for rel in (ghq list 2>/dev/null)
        set -l repo_path "$root/$rel"
        # repo 行: 表示 / 種別 / 対象パス / ghq 相対パス
        set -a lines (printf '%s\t%s\t%s\t%s' "$rel" repo "$repo_path" "$rel")

        # リンク worktree（ismain=0）のみ repo 行直下にツリー表示
        set -l linked
        for w in (__fzf_worktree_list "$repo_path")
            set -l p (string split -- $tab $w)
            test "$p[1]" = 0; and set -a linked "$w"
        end
        set -l n (count $linked)
        set -l i 0
        for w in $linked
            set i (math $i + 1)
            set -l p (string split -- $tab $w)
            set -l marker '├─'
            test $i -eq $n; and set marker '└─'
            set -a lines (printf '%s %s\t%s\t%s\t%s' "$marker" "$p[3]" worktree "$p[2]" "")
        end
    end

    set -l selection (
        printf '%s\n' $lines | fzf \
            --delimiter=$tab \
            --with-nth=1 \
            --preview 'fish -c "__fzf_picker_preview {2} {3} {4}"' \
            --preview-window=right:60%
    )

    if test -n "$selection"
        set -l parts (string split -- $tab $selection)
        # Jump through the command line so the move goes via fish_postexec and
        # auto-ls (45-auto-ls) lists the destination. Leading space keeps the
        # cd out of history; string escape keeps paths with spaces safe.
        commandline --replace -- ' cd '(string escape -- "$parts[3]")
        # Repaint before execute: right after fzf's full-screen UI exits the
        # command line is half-drawn, so executing immediately overlaps the
        # prompt with the cd (shows up as "cdcd"). Repaint settles it first.
        commandline -f repaint
        commandline -f execute
    else
        commandline -f repaint
    end
end
