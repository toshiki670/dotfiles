function _fzf_worktree
    if not git rev-parse --is-inside-work-tree >/dev/null 2>&1
        echo "not in a git repository"
        return 1
    end

    set -l tab (printf '\t')

    # メイン worktree はパスだけ控え、削除候補リストには含めない
    set -l main_path
    set -l lines
    for w in (__fzf_worktree_list .)
        set -l p (string split -- $tab $w)
        if test "$p[1]" = 1
            set main_path $p[2]
            continue
        end
        # リンク worktree のみ: 表示 / パス
        set -a lines (printf '%s\t%s' "$p[3]" "$p[2]")
    end

    if test (count $lines) -eq 0
        echo "削除できる worktree がありません"
        return
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

    read -l -P "WT を削除しますか? [y/N] " confirm
    if not string match -qri '^y' -- "$confirm"
        commandline -f repaint
        return
    end

    # 現在地が削除対象 worktree の内側なら、削除前にメインへ移動する
    # （現在地の worktree を消すと cwd が無効なディレクトリを指すため）
    set -l cur (path resolve -- $PWD)
    set -l target (path resolve -- $wpath)
    if test "$cur" = "$target"; or string match -q -- "$target/*" "$cur"
        test -n "$main_path"; and cd $main_path
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
