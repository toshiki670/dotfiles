function _fzf_gh_complete
    if not command -q gh
        commandline -f complete
        return
    end

    set -l tokens (commandline -po)
    set -l tab (printf '\t')

    # Activate only when: gh <resource> <targeted-subcommand> (no ID yet)
    if test (count $tokens) -ne 3; or test "$tokens[1]" != gh
        commandline -f complete
        return
    end

    set -l resource $tokens[2]
    set -l subcmd $tokens[3]

    set -l candidates
    set -l preview_cmd

    if test "$resource" = issue
        if not contains -- $subcmd view edit close reopen comment pin
            commandline -f complete
            return
        end
        set candidates (gh issue list --limit 100 --json number,title,state \
            --jq '.[] | "\(.number)\t[\(.state)] \(.title)"' 2>/dev/null)
        set preview_cmd 'env GH_FORCE_TTY=100% gh issue view {1}'

    else if test "$resource" = pr
        if not contains -- $subcmd view checkout review diff merge checks update-branch
            commandline -f complete
            return
        end
        set candidates (gh pr list --limit 100 --json number,title,state \
            --jq '.[] | "\(.number)\t[\(.state)] \(.title)"' 2>/dev/null)
        set preview_cmd 'env GH_FORCE_TTY=100% gh pr view {1}'

    else if test "$resource" = run
        if not contains -- $subcmd view rerun cancel watch
            commandline -f complete
            return
        end
        set candidates (gh run list --limit 50 --json databaseId,displayTitle,status \
            --jq '.[] | "\(.databaseId)\t[\(.status)] \(.displayTitle)"' 2>/dev/null)
        set preview_cmd 'env GH_FORCE_TTY=100% gh run view {1}'

    else if test "$resource" = release
        if not contains -- $subcmd view delete edit upload
            commandline -f complete
            return
        end
        set candidates (gh release list --limit 30 --json tagName,name,publishedAt \
            --jq '.[] | "\(.tagName)\t\(.name) (\(.publishedAt))"' 2>/dev/null)
        set preview_cmd 'env GH_FORCE_TTY=100% gh release view {1}'

    else if test "$resource" = gist
        if not contains -- $subcmd view edit
            commandline -f complete
            return
        end
        set candidates (gh gist list --limit 100 2>/dev/null \
            | awk -F'\t' '{print $1 "\t[" $4 "] " $2}')
        set preview_cmd 'env GH_FORCE_TTY=100% gh gist view {1}'

    else
        commandline -f complete
        return
    end

    if test (count $candidates) -eq 0
        echo -e "\n(no $resource found)" >&2
        commandline -f repaint
        return
    end

    set -l selected (
        printf '%s\n' $candidates \
        | fzf --delimiter=$tab \
            --with-nth=2 \
            --preview=$preview_cmd \
            --preview-window='right:60%' \
            --header="gh $resource $subcmd"
    )

    if test -n "$selected"
        set -l id (string split $tab -- $selected)[1]
        commandline -i "$id"
    end

    commandline -f repaint
end
