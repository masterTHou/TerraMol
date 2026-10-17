@echo off
setlocal
cd /d "%~dp0"
if exist "%~dp0TerraMol.exe" (
    start "" "%~dp0TerraMol.exe"
    exit /b 0
)
if exist "%~dp0target\release\terramol_viewer.exe" (
    start "" "%~dp0target\release\terramol_viewer.exe"
    exit /b 0
)
echo TerraMol.exe was not found.
echo Download the portable Windows viewer and EXTRACT ALL files first:
echo https://github.com/masterTHou/TerraMol/releases
echo No Unity, Rust, Python or Node.js installation is required for the portable package.
pause
exit /b 1
