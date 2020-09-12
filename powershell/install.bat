@echo off

rem Exist check
where pwsh 2> nul > nul
if not %errorlevel% == 0 (
    echo Not found: PowerShell Core
    pause
    exit
)

rem Get PowerShell profile location
for /F %%i in ('pwsh -c echo $profile') do set PWSH_PROFILE_PATH=%%i

rem Make simbolic link
mklink %PWSH_PROFILE_PATH% %USERPROFILE%\dotfiles\powershell\Microsoft.PowerShell_profile.ps1

echo Profile placement successful!


rem PowerShell setting
pwsh -c 'Set-PSReadLineKeyHandler -Key Tab -Function MenuComplete'
echo setting successful!

pause
