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

:: NotebookLM HITL prompt
echo ----------------------------------------
echo Will you be using NotebookLM for slides?
echo ----------------------------------------
echo.
echo NotebookLM requires Google login for slide generation.
echo.
choice /c YN /m "Open NotebookLM to verify login? [Y/N]"
if %errorlevel%==1 (
    echo.
    echo Opening NotebookLM...
    start https://notebooklm.google.com
    echo.
    echo Please ensure you are logged into Google.
    echo Press any key when ready to continue...
    pause >nul
)

echo.
echo ----------------------------------------
echo Starting Claude CLI...
echo ----------------------------------------
echo.

:: Start Claude CLI in this window
claude
