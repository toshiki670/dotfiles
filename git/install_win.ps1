Write-Host -NoNewline 'Writing ...'

$GitconfigPath = "$env:USERPROFILE\.gitconfig"
$DotfilesGitconfigPath = "$env:USERPROFILE\dotfiles\git\.gitconfig"

Get-Content $DotfilesGitconfigPath | Add-Content $GitconfigPath

Write-Host ' OK'

pause

