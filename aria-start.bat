@echo off
:: ARIA Launcher - Starts dashboard and Claude CLI together
:: Usage: aria-start.bat

echo.
echo ========================================
echo        ARIA Development Session
echo ========================================
echo.

:: Start dashboard in background window (keeps open on error)
echo Starting ARIA Dashboard...
start "ARIA Dashboard" cmd /k "python .aria\scripts\serve-dashboard.py || pause"

:: Wait a moment for dashboard to start
timeout /t 3 /nobreak >nul

:: Check if dashboard is responding
curl -s http://localhost:8420 >nul 2>&1
if %errorlevel% neq 0 (
    echo.
    echo WARNING: Dashboard may not have started. Check the "ARIA Dashboard" window.
    echo.
    pause
)

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
