# Pure-pwsh
# Import-Module pure-pwsh
Import-Module posh-git

# For git
function Git-Custom-Add-Stage($Path) {
    git add $Path
}
function Git-Custom-Add-Stage-by-Patch {
    git add -p
}
function Git-Custom-Show-Diff {
    git diff
}
function Git-Custom-Show-Diff-On-Staged {
    git diff --staged
}
function Git-Custom-Show-Status {
    git status
}

Set-Alias g git
Set-Alias gad Git-Custom-Add-Stage
Set-Alias gap Git-Custom-Add-Stage-by-Patch
Set-Alias gd Git-Custom-Show-Diff
Set-Alias gds Git-Custom-Show-Diff-On-Staged
Set-Alias gs Git-Custom-Show-Status


# For git flow
function Git-Custom-Flow-Feature {
    git flow feature $args
}
function Git-Custom-Flow-Hotfix {
    git flow hotfix $args
}
function Git-Custom-Flow-Init {
    git flow init $args
}
function Git-Custom-Flow-Release {
    git flow release $args
}
function Git-Custom-Flow-Support {
    git flow support $args
}
function Git-Custom-Flow-Version {
    git flow version $args
}

Set-Alias Gfeature Git-Custom-Flow-Feature
Set-Alias Ghotfix Git-Custom-Flow-Hotfix
Set-Alias Ginit Git-Custom-Flow-Init
Set-Alias Grelease Git-Custom-Flow-Release
Set-Alias Gsupport Git-Custom-Flow-Support
Set-Alias Gversion Git-Custom-Flow-Version
