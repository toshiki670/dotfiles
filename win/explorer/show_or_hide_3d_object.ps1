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
    Start-Process pwsh -ArgumentList $list -Verb RunAs -Wait
}


if(-Not (Test-Admin)) {
    Start-ScriptAsAdmin -ScriptPath $PSCommandPath
} else {
    $3dObjectPath = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FolderDescriptions\{31C0DD25-9439-4F12-BF41-7FF4EDA38722}'

    # If the PropertyBag key is not present
    if (-Not (Test-Path -LiteralPath "$3dObjectPath\PropertyBag")) {
        New-Item -Path "$3dObjectPath\PropertyBag"
        New-ItemProperty -Path "$3dObjectPath\PropertyBag" -Name ThisPCPolicy -PropertyType String -Value Show
    }

    # Toggle Show/Hide
    $CurrentState = (Get-ItemProperty -Path "$3dObjectPath\PropertyBag" -Name ThisPCPolicy).ThisPCPolicy
    if('Show' -eq $CurrentState) {
        Set-ItemProperty -Path "$3dObjectPath\PropertyBag" -Name ThisPCPolicy -Value Hide
    } else {
        Set-ItemProperty -Path "$3dObjectPath\PropertyBag" -Name ThisPCPolicy -Value Show
    }

    # Show result
    $CurrentState = (Get-ItemProperty -Path "$3dObjectPath\PropertyBag" -Name ThisPCPolicy).ThisPCPolicy
    echo Successed!
    echo "Current state is '$CurrentState'." ""
    echo "Please reboot Explorer or PC." ""

    pause
} 


