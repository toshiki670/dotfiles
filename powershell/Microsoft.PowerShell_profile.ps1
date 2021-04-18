# Pure-pwsh
# Import-Module pure-pwsh
Import-Module posh-git

# For git
function Add-To-Git-Stage($Path) {
    git add $Path
}
function Add-To-Git-Stage-by-Patch {
    git add -p
}
function Show-Git-Diff {
    git diff
}
function Show-Git-Diff-On-Staged {
    git diff --staged
}
function Show-Git-Status {
    git status
}

Set-Alias g git
Set-Alias gad Add-To-Git-Stage
Set-Alias gap Add-To-Git-Stage-by-Patch
Set-Alias gd Show-Git-Diff
Set-Alias gds Show-Git-Diff-On-Staged
Set-Alias gs Show-Git-Status


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


# For scoop
function Update-Scoop-All {
    scoop update
    scoop update *
    scoop cache rm *
    scoop cleanup *
}
