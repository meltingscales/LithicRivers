@echo off
setlocal enabledelayedexpansion

:: Get the directory where this script is located
set "SCRIPT_DIR=%~dp0"
set "GAME_BIN=%SCRIPT_DIR%lithicrivers-client.exe"
set "GAME_ARGS=%*"

:: Check if we're in a desktop environment (not headless)
:: We'll check for common environment variables that indicate a desktop session
set "IS_DESKTOP=0"
if defined SESSIONNAME (
    if not "%SESSIONNAME%" == "Console" (
        set "IS_DESKTOP=1"
    )
) else if defined SESSIONNAME (
    set "IS_DESKTOP=1"
)

:: Check if we're running in a remote desktop session
reg query "HKEY_CURRENT_USER\Volatile Environment" /v SESSIONNAME >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    set "IS_DESKTOP=1"
)

:: If not in a desktop environment or if we can't determine, run directly
if "%IS_DESKTOP%"=="0" (
    "%GAME_BIN%" %GAME_ARGS%
    goto :eof
)

:: Try to find a terminal emulator
set "TERMINAL="
where wt >nul 2>&1 && set "TERMINAL=wt" && goto :found_terminal
where cmd >nul 2>&1 && set "TERMINAL=cmd" && goto :found_terminal
where powershell >nul 2>&1 && set "TERMINAL=powershell" && goto :found_terminal

:found_terminal
if "%TERMINAL%"=="" (
    :: No terminal found, run directly
    "%GAME_BIN%" %GAME_ARGS%
    goto :eof
)

:: Run the game in the found terminal
if "%TERMINAL%"=="wt" (
    start "" wt -d "%SCRIPT_DIR%" cmd /k ""%GAME_BIN%" %GAME_ARGS% && echo. && echo Game exited with code !ERRORLEVEL! && pause"
) else if "%TERMINAL%"=="powershell" (
    start "" powershell -NoExit -Command "cd '%SCRIPT_DIR%'; & '%GAME_BIN%' %GAME_ARGS%; Write-Host 'Press Enter to exit...'; $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')"
) else (
    :: Default to cmd
    start "" cmd /k "cd /d "%SCRIPT_DIR%" && "%GAME_BIN%" %GAME_ARGS% && echo. && echo Game exited with code !ERRORLEVEL! && pause"
)