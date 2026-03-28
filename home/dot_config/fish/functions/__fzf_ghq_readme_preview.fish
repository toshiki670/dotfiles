function __fzf_ghq_readme_preview
    set -l rel (string trim -- $argv[1])
    test -n "$rel"; or return 1
    set -l r (ghq root)/$rel
    for n in README.md readme.md README Readme.md
        if test -f "$r/$n"
            head -n 400 "$r/$n"
            return 0
        end
    end
    echo "No README"
end
