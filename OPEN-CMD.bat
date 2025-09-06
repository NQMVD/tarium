@echo off
REM drop this in the SPT folder and double-click to open CMD with usage hints
pushd "%~dp0"

echo.
echo ================= SPT Mod Manager - Tarium =================
echo Basic commands:
echo   tarium.exe profile          - list profiles
echo   tarium.exe list             - list installed mods
echo   tarium.exe add X            - install mod - github name/repo
echo   tarium.exe download          - download and install mods
echo   tarium.exe install --local  - only install mods already downloaded
echo   tarium.exe help             - show full help
echo ============================================================
echo.
echo (You can start typing and press TAB to autocomplete)
echo.
echo Running in: %CD%
echo.
cmd.exe /k

popd
