@echo off
:: ARIA Launcher - Starts dashboard and Claude CLI together
:: Usage: aria-start.bat

echo.
echo ========================================
echo        ARIA Development Session
echo ========================================
echo.

:: Start dashboard in background window
echo Starting ARIA Dashboard...
start "ARIA Dashboard" cmd /c "python .aria\scripts\serve-dashboard.py"

:: Wait a moment for dashboard to start
timeout /t 2 /nobreak >nul

:: Open dashboard in browser
start http://localhost:8420

echo.
echo Dashboard: http://localhost:8420
echo.
echo Starting Claude CLI...
echo ----------------------------------------
echo.

:: Start Claude CLI in this window
claude
