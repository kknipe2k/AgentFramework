@echo off
REM ARIA Project Workspace Setup (Windows)
REM Usage: setup-project.bat <project-name> [aria-path] [eval-path]
REM Example: setup-project.bat SVM
REM Example: setup-project.bat SVM c:\aria-test c:\aria\eval

setlocal enabledelayedexpansion

if "%~1"=="" (
    echo Usage: setup-project.bat ^<project-name^> [aria-path] [eval-path]
    echo.
    echo Examples:
    echo   setup-project.bat SVM
    echo   setup-project.bat SVM c:\aria-test c:\aria\eval
    echo.
    echo Defaults:
    echo   aria-path: c:\aria-test
    echo   eval-path: c:\aria\eval
    exit /b 1
)

set PROJECT_NAME=%~1
set ARIA=%~2
set EVAL=%~3

if "%ARIA%"=="" set ARIA=c:\aria-test
if "%EVAL%"=="" set EVAL=c:\aria\eval

set PROJECT=%EVAL%\%PROJECT_NAME%

REM Check ARIA source exists
if not exist "%ARIA%\CLAUDE.md" (
    echo ERROR: ARIA not found at %ARIA%
    echo Clone it first: git clone -b main https://github.com/kknipe2k/AgentFramework.git %ARIA%
    exit /b 1
)

REM Check project doesn't already exist
if exist "%PROJECT%" (
    echo ERROR: Project already exists: %PROJECT%
    echo Delete it first or choose a different name.
    exit /b 1
)

echo Creating project workspace: %PROJECT%
echo ARIA source: %ARIA%
echo.

REM Create project directory structure
mkdir "%PROJECT%"
mkdir "%PROJECT%\.aria"
mkdir "%PROJECT%\.aria\state"
mkdir "%PROJECT%\.aria\docs"
mkdir "%PROJECT%\.aria\outputs"
mkdir "%PROJECT%\.aria\prototypes"
mkdir "%PROJECT%\sources"

REM Symlink immutable framework files
echo Linking framework files...
mklink "%PROJECT%\CLAUDE.md" "%ARIA%\CLAUDE.md" >nul
mklink /D "%PROJECT%\.aria\skills" "%ARIA%\.aria\skills" >nul
mklink /D "%PROJECT%\.aria\scripts" "%ARIA%\.aria\scripts" >nul
mklink /D "%PROJECT%\.aria\templates" "%ARIA%\.aria\templates" >nul
mklink /D "%PROJECT%\.aria\dashboard" "%ARIA%\.aria\dashboard" >nul

REM Copy mutable state files (fresh per project)
echo Copying state templates...
if exist "%ARIA%\.aria\state\*.json" (
    copy "%ARIA%\.aria\state\*.json" "%PROJECT%\.aria\state\" >nul 2>&1
)

REM Create empty state files if they don't exist
if not exist "%PROJECT%\.aria\state\progress.json" (
    echo {"tasks": []} > "%PROJECT%\.aria\state\progress.json"
)
if not exist "%PROJECT%\.aria\state\decisions.jsonl" (
    type nul > "%PROJECT%\.aria\state\decisions.jsonl"
)
if not exist "%PROJECT%\.aria\state\signals.jsonl" (
    type nul > "%PROJECT%\.aria\state\signals.jsonl"
)

REM Copy verify.sh if it exists
if exist "%ARIA%\.aria\verify.sh" (
    copy "%ARIA%\.aria\verify.sh" "%PROJECT%\.aria\verify.sh" >nul
)

REM Create README for the project
(
echo # %PROJECT_NAME%
echo.
echo ARIA workspace created: %date% %time%
echo.
echo ## Structure
echo.
echo ```
echo sources/              - Drop papers, docs, repos here
echo .aria/
echo   docs/IDEA.md        - Research synthesis
echo   outputs/            - Slides, FOCUS.md
echo   prototypes/         - Working demos
echo ```
echo.
echo ## Usage
echo.
echo 1. Drop source materials in sources/
echo 2. Open this folder in VS Code
echo 3. Run ARIA research workflow
echo 4. Choose prototype variant when prompted
) > "%PROJECT%\README.md"

echo.
echo ========================================
echo Project ready: %PROJECT%
echo ========================================
echo.
echo Next steps:
echo   1. Drop source materials in: %PROJECT%\sources\
echo   2. Open VS Code: code "%PROJECT%"
echo   3. Run ARIA research workflow
echo.
echo Results will be saved in:
echo   - .aria/docs/IDEA.md       (research synthesis)
echo   - .aria/outputs/FOCUS.md   (slide outline)
echo   - .aria/outputs/slides-*   (presentation)
echo   - .aria/prototypes/        (working demos)
echo.

endlocal
