# Context condition helpers

function __gh_completing_issue_id
    set -l cmd (commandline -po)
    test (count $cmd) -ge 3
        and test "$cmd[2]" = issue
        and contains -- "$cmd[3]" view edit close reopen comment pin
end

function __gh_completing_pr_id
    set -l cmd (commandline -po)
    test (count $cmd) -ge 3
        and test "$cmd[2]" = pr
        and contains -- "$cmd[3]" view checkout review diff merge checks update-branch
end

function __gh_completing_run_id
    set -l cmd (commandline -po)
    test (count $cmd) -ge 3
        and test "$cmd[2]" = run
        and contains -- "$cmd[3]" view rerun cancel watch
end

function __gh_completing_release_id
    set -l cmd (commandline -po)
    test (count $cmd) -ge 3
        and test "$cmd[2]" = release
        and contains -- "$cmd[3]" view delete edit upload
end

function __gh_completing_gist_id
    set -l cmd (commandline -po)
    test (count $cmd) -ge 3
        and test "$cmd[2]" = gist
        and contains -- "$cmd[3]" view edit
end

# Candidate generators

function __gh_issue_candidates
    gh issue list --limit 100 --json number,title,state \
        --jq '.[] | "\(.number)\t[\(.state)] \(.title)"' 2>/dev/null
end

function __gh_pr_candidates
    gh pr list --limit 100 --json number,title,state \
        --jq '.[] | "\(.number)\t[\(.state)] \(.title)"' 2>/dev/null
end

function __gh_run_candidates
    gh run list --limit 50 --json databaseId,displayTitle,status \
        --jq '.[] | "\(.databaseId)\t[\(.status)] \(.displayTitle)"' 2>/dev/null
end

function __gh_release_candidates
    gh release list --limit 30 --json tagName,name,publishedAt \
        --jq '.[] | "\(.tagName)\t\(.name) (\(.publishedAt))"' 2>/dev/null
end

function __gh_gist_candidates
    gh gist list --limit 100 2>/dev/null \
        | awk -F'\t' '{print $1 "\t" $2}'
end

# Completion rules: -f suppresses file completions when condition is met

complete -c gh -n __gh_completing_issue_id -f -a '(__gh_issue_candidates)'
complete -c gh -n __gh_completing_pr_id -f -a '(__gh_pr_candidates)'
complete -c gh -n __gh_completing_run_id -f -a '(__gh_run_candidates)'
complete -c gh -n __gh_completing_release_id -f -a '(__gh_release_candidates)'
complete -c gh -n __gh_completing_gist_id -f -a '(__gh_gist_candidates)'
