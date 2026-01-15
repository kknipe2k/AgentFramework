@echo off
REM ARIA Project Workspace Setup (Windows)
REM Usage: setup-project.bat <project-path> <aria-path>
REM Example: setup-project.bat "C:\aria-eval\Projects\SVM" "C:\aria-test"

setlocal enabledelayedexpansion

if "%~1"=="" (
    echo Usage: setup-project.bat ^<project-path^> ^<aria-path^>
    echo.
    echo Examples:
    echo   setup-project.bat "C:\aria-eval\Projects\SVM" "C:\aria-test"
    echo   setup-project.bat "C:\my-test" "C:\aria-test"
    echo.
    echo Arguments:
    echo   project-path: Full path to create workspace
    echo   aria-path:    Path to cloned ARIA framework
    exit /b 1
)

if "%~2"=="" (
    echo ERROR: aria-path is required
    echo Usage: setup-project.bat ^<project-path^> ^<aria-path^>
    exit /b 1
)

set PROJECT=%~1
set ARIA=%~2

REM Extract project name from path for display
for %%i in ("%PROJECT%") do set PROJECT_NAME=%%~ni

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

REM ============================================
REM Create project directory structure
REM ============================================
mkdir "%PROJECT%"
mkdir "%PROJECT%\.aria"
mkdir "%PROJECT%\.aria\state"
mkdir "%PROJECT%\.aria\docs"
mkdir "%PROJECT%\.aria\outputs"
mkdir "%PROJECT%\.aria\prototypes"
mkdir "%PROJECT%\.aria\logs"
mkdir "%PROJECT%\.aria\reports"
mkdir "%PROJECT%\sources"

REM ============================================
REM Symlink immutable framework files
REM ============================================
echo Linking framework files...

REM Root CLAUDE.md
mklink "%PROJECT%\CLAUDE.md" "%ARIA%\CLAUDE.md" >nul

REM Skill definitions (read-only) - use /J junction for Windows compatibility
mklink /J "%PROJECT%\.aria\skills" "%ARIA%\.aria\skills" >nul

REM Scripts (read-only)
mklink /J "%PROJECT%\.aria\scripts" "%ARIA%\.aria\scripts" >nul

REM Templates (read-only)
if exist "%ARIA%\.aria\templates" mklink /J "%PROJECT%\.aria\templates" "%ARIA%\.aria\templates" >nul 2>&1

REM Dashboard (read-only)
if exist "%ARIA%\.aria\dashboard" mklink /J "%PROJECT%\.aria\dashboard" "%ARIA%\.aria\dashboard" >nul 2>&1

REM Git hooks (read-only)
if exist "%ARIA%\.aria\hooks" mklink /J "%PROJECT%\.aria\hooks" "%ARIA%\.aria\hooks" >nul 2>&1

REM Safety rails (read-only)
if exist "%ARIA%\.aria\rails" mklink /J "%PROJECT%\.aria\rails" "%ARIA%\.aria\rails" >nul 2>&1

REM Planner (read-only)
if exist "%ARIA%\.aria\planner" mklink /J "%PROJECT%\.aria\planner" "%ARIA%\.aria\planner" >nul 2>&1

REM Ralph executor (read-only)
if exist "%ARIA%\.aria\ralph" mklink /J "%PROJECT%\.aria\ralph" "%ARIA%\.aria\ralph" >nul 2>&1

REM Claude IDE integration (read-only)
if exist "%ARIA%\.claude" mklink /J "%PROJECT%\.claude" "%ARIA%\.claude" >nul 2>&1

REM ============================================
REM Symlink core shell scripts
REM ============================================
echo Linking core scripts...
for %%s in (verify.sh common.sh git-ops.sh hitl.sh aria-engine.sh verify-executor.sh rails-executor.sh model-selector.sh agent-runner.sh design-notes.sh discover.sh pause.sh) do (
    if exist "%ARIA%\.aria\%%s" mklink "%PROJECT%\.aria\%%s" "%ARIA%\.aria\%%s" >nul 2>&1
)

REM ============================================
REM Create empty state files
REM ============================================
echo Initializing state files...

REM progress.json - task tracking
(
echo {
echo   "tasks": [],
echo   "mode": null,
echo   "started": null,
echo   "completed": null
echo }
) > "%PROJECT%\.aria\state\progress.json"

REM current-plan.json - active plan
(
echo {
echo   "id": null,
echo   "title": null,
echo   "status": "empty",
echo   "created": null,
echo   "tasks": []
echo }
) > "%PROJECT%\.aria\state\current-plan.json"

REM Empty JSONL files for tracing
type nul > "%PROJECT%\.aria\state\decisions.jsonl"
type nul > "%PROJECT%\.aria\state\signals.jsonl"

REM ============================================
REM Create project-context.md template
REM ============================================
(
echo # Project Context
echo.
echo *Edit this file to capture project-specific knowledge for ARIA.*
echo.
echo ---
echo.
echo ## Tech Stack
echo.
echo - [List your technologies here]
echo.
echo ## Directory Structure
echo.
echo - `sources/` - Input materials ^(papers, docs, repos^)
echo - `.aria/` - ARIA framework and outputs
echo.
echo ## Don't Touch
echo.
echo - [List areas that should NOT be modified without approval]
echo.
echo ## Special Instructions
echo.
echo - [Any project-specific rules or patterns]
echo.
echo ---
echo.
echo ## Ready for ARIA
echo.
echo Run discovery to auto-populate this file.
) > "%PROJECT%\.aria\project-context.md"

REM ============================================
REM Create design-notes.md template
REM ============================================
(
echo # Design Notes
echo.
echo *AI reasoning log - decisions and rationale*
echo.
echo ---
echo.
echo ## Session Log
echo.
echo ^<!-- ARIA will append decisions here during execution --^>
) > "%PROJECT%\.aria\design-notes.md"

REM ============================================
REM Create project README
REM ============================================
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
echo   state/              - JSON state ^(plan, progress, traces^)
echo   docs/               - IDEA.md, research synthesis
echo   outputs/            - Slides, FOCUS.md
echo   prototypes/         - Working demos
echo   logs/               - Token usage, tracking
echo   reports/            - Final reports
echo ```
echo.
echo ## Workflows
echo.
echo **Research:** Drop paper in sources/ -^> IDEA.md -^> slides -^> prototype
echo **Build:** Plan -^> execute -^> verify -^> commit
echo **Modify:** Discovery -^> plan -^> execute -^> verify
echo.
echo ## Usage
echo.
echo 1. Drop source materials in `sources/`
echo 2. Open this folder in VS Code
echo 3. Run ARIA workflow ^(research, build, or modify^)
echo 4. Choose prototype variant when prompted ^(research flow^)
echo.
echo ## Mode Selection
echo.
echo ARIA auto-selects mode based on task size:
echo - **LITE** ^(1-5 tasks^): Fast, minimal overhead
echo - **STANDARD** ^(6-15 tasks^): Normal workflow with verification
echo - **FULL** ^(16-40 tasks^): Maximum oversight, design notes
echo - **FULL+** ^(40+ tasks^): Epic-level management, design doc required
) > "%PROJECT%\README.md"

REM ============================================
REM Summary
REM ============================================
echo.
echo ========================================
echo Project ready: %PROJECT%
echo ========================================
echo.
echo Structure created:
echo   sources/              - Input materials
echo   .aria/state/          - JSON state files
echo   .aria/docs/           - Research outputs ^(IDEA.md^)
echo   .aria/outputs/        - Slides, FOCUS.md
echo   .aria/prototypes/     - Working demos
echo   .aria/logs/           - Token tracking
echo   .aria/reports/        - Final reports
echo.
echo State files initialized:
echo   - progress.json       ^(task tracking^)
echo   - current-plan.json   ^(active plan^)
echo   - decisions.jsonl     ^(decision trace^)
echo   - signals.jsonl       ^(tool signals^)
echo   - project-context.md  ^(project knowledge^)
echo   - design-notes.md     ^(reasoning log^)
echo.
echo Next steps:
echo   1. Drop source materials in: %PROJECT%\sources\
echo   2. Open VS Code: code "%PROJECT%"
echo   3. Run ARIA workflow
echo.
echo Modes: LITE ^| STANDARD ^| FULL ^| FULL+
echo Flows: research ^| build ^| modify
echo.

endlocal
