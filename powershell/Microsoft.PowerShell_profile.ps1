# Pure-pwsh
Import-Module pure-pwsh
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
