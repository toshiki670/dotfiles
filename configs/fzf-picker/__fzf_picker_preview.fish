function __fzf_picker_preview --argument-names type path rel
    if test "$type" = repo
        __fzf_ghq_readme_preview "$rel"
    else
        git -C "$path" log --oneline -20
    end
end
