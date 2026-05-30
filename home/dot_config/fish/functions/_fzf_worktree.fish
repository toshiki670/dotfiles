function _fzf_worktree
    if not git rev-parse --is-inside-work-tree >/dev/null 2>&1
        echo "not in a git repository"
        commandline -f repaint
        return 1
    end

    set -l tab (printf '\t')
    set -l toplevel (git rev-parse --show-toplevel 2>/dev/null)

    set -l lines
    for w in (__fzf_worktree_list .)
        set -l p (string split -- $tab $w)
        set -l label $p[3]
        test "$p[1]" = 1; and set label "* $label"
        # 表示 / パス / ismain
        set -a lines (printf '%s\t%s\t%s' "$label" "$p[2]" "$p[1]")
    end

    set -l selection (
        printf '%s\n' $lines | fzf \
            --delimiter=$tab \
            --with-nth=1 \
            --preview "git -C {2} log --oneline -20" \
            --preview-window=right:60%
    )

    if test -z "$selection"
        commandline -f repaint
        return
    end

    set -l parts (string split -- $tab $selection)
    set -l wpath $parts[2]
    set -l ismain $parts[3]

    # メイン / 現在地の worktree は削除不可（force でも不可）。事前にはじく
    if test "$ismain" = 1
        echo "メイン worktree は削除できません: $wpath"
        commandline -f repaint
        return 1
    end
    if test -n "$toplevel"
        set -l cur (path resolve "$toplevel")
        set -l sel (path resolve "$wpath")
        if test "$cur" = "$sel"
            echo "現在地の worktree は削除できません（別のディレクトリへ移動してから実行してください）: $wpath"
            commandline -f repaint
            return 1
        end
    end

    read -l -P "WT を削除しますか? [y/N] " confirm
    if not string match -qri '^y' -- "$confirm"
        commandline -f repaint
        return
    end

    if git worktree remove "$wpath"
        echo "削除しました: $wpath"
        commandline -f repaint
        return
    end

    read -l -P "強制削除しますか? [y/N] " force
    if string match -qri '^y' -- "$force"
        git worktree remove --force "$wpath"; and echo "強制削除しました: $wpath"
    end
    commandline -f repaint
end
