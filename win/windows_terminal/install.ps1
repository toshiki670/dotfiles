# Reference
# http://winscript.jp/powershell/302

function Test-Admin {
    (
        [Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Start-ScriptAsAdmin {
    param(
        [string]
        $ScriptPath,
        [object[]]
        $ArgumentList
    )
    $list = @($ScriptPath)
    if($null -ne $ArgumentList) {
        $list += @($ArgumentList)
    }
    Start-Process powershell -ArgumentList $list -Verb RunAs -Wait
}


if(-Not (Test-Admin)) {
    Start-ScriptAsAdmin -ScriptPath $PSCommandPath
} else {
    $TerminalPath = "$env:ProgramFiles\WindowsApps\Microsoft.WindowsTerminal_1.2.2381.0_x64__8wekyb3d8bbwe\WindowsTerminal.exe"
    $ConfigPath = "$env:LOCALAPPDATA\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json"
    $DotfilesConfigPath = "$env:USERPROFILE\dotfiles\win\windows_terminal\settings.json"

    # If the Windows Terminal doesn't exist.
    if (-Not(Test-Path $TerminalPath)) {
        echo "Not found: Windows Terminal"
        pause
	exit
    }

    # Make simbolic link
    New-Item -ItemType SymbolicLink -Value $DotfilesConfigPath -Path $ConfigPath

    # Show result
    echo Successed!

    pause
}

