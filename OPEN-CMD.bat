@echo off
REM drop this in the SPT folder and double-click to open CMD with usage hints
pushd "%~dp0"

echo.
echo ================= SPT Mod Manager - Tarium =================
echo Basic commands:
echo   tarium.exe profile                 - list profiles
echo   tarium.exe add X --no-checks       - install mod - github owner/repo
echo   tarium.exe add-from X               - install mods from file
echo   tarium.exe list                    - list installed mods
echo   tarium.exe download                - download and install mods
echo   tarium.exe install --no-download   - only install mods already downloaded
echo   tarium.exe help                    - show full help
echo ============================================================
echo.
echo (You can start typing and press TAB to autocomplete)
echo (--no-checks is recommended by default for now)
echo.
echo Running in: %CD%
echo.
cmd.exe /k

popd
