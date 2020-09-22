$dotfiles = "$env:USERPROFILE\dotfiles"


# Nvim Setup
$NvimConfPath = "$env:LOCALAPPDATA\nvim"
$NvimConfName = 'init.vim'
$DotfilesNvimConfPath = "$dotfiles\vim\.vimrc"

Get-Command nvim | Out-Null
if($? -eq $true) {
    Write-Host 'Neovim installing ...'

    if (-Not(Test-Path $NvimConfPath)) {
        New-Item -ItemType Directory -Path $NvimConfPath
    }
    New-Item -ItemType SymbolicLink -Value $DotfilesNvimConfPath -Path $NvimConfPath -Name $NvimConfName
    nvim -c 'cal dein#update()' -c 'qa!'

    Write-Host 'Successed!'
} else {
    Write-Warning "Neovim isn't found."
}

