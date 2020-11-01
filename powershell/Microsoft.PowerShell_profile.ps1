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
function Use-Git-Flow-Feature {
    git flow feature $args
}
function Use-Git-Flow-Hotfix {
    git flow hotfix $args
}
function Use-Git-Flow-Init {
    git flow init $args
}
function Use-Git-Flow-Release {
    git flow release $args
}
function Use-Git-Flow-Support {
    git flow support $args
}
function Use-Git-Flow-Version {
    git flow version $args
}

Set-Alias Gfeature Use-Git-Flow-Feature
Set-Alias Ghotfix Use-Git-Flow-Hotfix
Set-Alias Ginit Use-Git-Flow-Init
Set-Alias Grelease Use-Git-Flow-Release
Set-Alias Gsupport Use-Git-Flow-Support
Set-Alias Gversion Use-Git-Flow-Version


# For Neovim
function Open-Nvim-As-Utf8 {
    nvim -c ":e ++enc=utf8"
}
function Open-Nvim-As-Euc-Jp {
    vim -c ":e ++enc=euc-jp"
}
function Open-Nvim-As-Shift-Jis {
    vim -c ":e ++enc=shift_jis"
}

Set-Alias vim nvim
Set-Alias v vim
Set-Alias vim-utf8 Open-Nvim-As-Utf8
Set-Alias vim-euc_jp Open-Nvim-As-Euc-Jp
Set-Alias vim-shift_jis Open-Nvim-As-Shift-Jis
