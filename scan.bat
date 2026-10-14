@echo off
title BloatHound - Fast Developer Artifact and Cache Purger
cd /d "%~dp0"

echo ===================================================
echo   Scanning current workspace for disposable cache...
echo ===================================================
echo.

cargo run -- scan .

echo.
pause
