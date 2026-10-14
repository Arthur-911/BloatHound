@echo off
title dev-cleaner - Fast Developer Artifact and Cache Purger
cd /d "%~dp0"

echo ===================================================
echo   Scanning current workspace for disposable cache...
echo ===================================================
echo.

cargo run -- scan .

echo.
pause
