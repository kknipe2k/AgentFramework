# ARIA Complete Rail System

## The Problem

4 rails isn't enough. A non-coder building with Claude needs:

- Environment verification (is the server even running?)
- E2E testing (can I see the thing?)
- User verification (does it look right to YOU?)
- Safety checks (did Claude just delete my database?)
- Sequencing (what order to do things?)

## Rail Categories

### 1. ENVIRONMENT RAILS
```yaml
rails:
  env_node:
    check: "node --version"
    expect: exit_code == 0
    message: "Node.js not installed"
    fix: "Install Node.js from nodejs.org"

  env_server_running:
    check: "curl -s localhost:${PORT:-3000}"
    expect: exit_code == 0
    message: "Server not running"
    fix: "Run: npm start"

  env_deps_installed:
    check: "test -d node_modules"
    expect: exit_code == 0
    message: "Dependencies not installed"
    fix: "Run: npm install"

  env_db_connected:
    check: "pg_isready -h localhost"
    expect: exit_code == 0
    message: "Database not running"
    fix: "Start PostgreSQL"

  env_vars_set:
    check: "test -n \"$DATABASE_URL\""
    expect: exit_code == 0
    message: "DATABASE_URL not set"
    fix: "Create .env file with DATABASE_URL"
```

### 2. CODE QUALITY RAILS
```yaml
rails:
  quality_tests:
    check: "npm test"
    expect: exit_code == 0
    message: "Tests failing"
    block: [commit, deploy]

  quality_lint:
    check: "npm run lint"
    expect: exit_code == 0
    message: "Linting errors"
    block: [commit]

  quality_types:
    check: "npx tsc --noEmit"
    expect: exit_code == 0
    message: "Type errors"
    block: [commit]

  quality_build:
    check: "npm run build"
    expect: exit_code == 0
    message: "Build failed"
    block: [deploy]
```

### 3. VERIFICATION RAILS
```yaml
rails:
  verify_app_loads:
    type: playwright
    script: |
      await page.goto('http://localhost:3000');
      await expect(page).toHaveTitle(/./);
    message: "App doesn't load in browser"

  verify_user_flow:
    type: playwright
    script: |
      await page.goto('http://localhost:3000');
      await page.click('button[type="submit"]');
      await expect(page.locator('.success')).toBeVisible();
    message: "User flow broken"

  verify_visual:
    type: screenshot
    compare_to: "baseline.png"
    threshold: 0.1
    message: "Visual regression detected"

  verify_manual:
    type: user_confirm
    prompt: |
      Please check the app at http://localhost:3000

      Does it:
      - [ ] Load without errors?
      - [ ] Show the expected content?
      - [ ] Work when you click buttons?

      Type 'yes' to continue or describe the problem:
    message: "User rejected verification"
```

### 4. SAFETY RAILS
```yaml
rails:
  safety_no_secrets:
    type: content_scan
    patterns:
      - /API_KEY\s*=\s*['"][^'"]+['"]/
      - /password\s*=\s*['"][^'"]+['"]/
      - /SECRET.*=.*[A-Za-z0-9]{20,}/
    files: ["**/*.js", "**/*.ts", "**/*.env"]
    exclude: ["*.example", "*.sample"]
    message: "Possible secret in code"
    block: [commit, push]

  safety_no_destructive:
    type: command_scan
    block_patterns:
      - "rm -rf /"
      - "DROP DATABASE"
      - "DROP TABLE"
      - "git push --force"
      - ":(){ :|:& };:"
    message: "Destructive command blocked"
    block: [execute]

  safety_backup_exists:
    type: pre_action
    before: [major_refactor, database_migration]
    check: "git stash list | head -1"
    message: "Create backup before proceeding"
    fix: "git stash push -m 'backup before changes'"

  safety_branch_protection:
    type: git_check
    block_direct_push: [main, master, production]
    message: "Cannot push directly to protected branch"
```

### 5. DOCUMENTATION RAILS
```yaml
rails:
  docs_readme_exists:
    check: "test -f README.md"
    expect: exit_code == 0
    message: "No README.md"
    block: [done]

  docs_has_run_instructions:
    type: content_check
    file: "README.md"
    must_contain:
      - /npm (install|i)/
      - /npm (start|run)/
    message: "README missing run instructions"

  docs_api_documented:
    type: file_check
    if_exists: "src/api/**/*.ts"
    require: "docs/api.md"
    message: "API routes need documentation"

  docs_changelog_updated:
    type: git_check
    if_changed: ["src/**"]
    require_changed: ["CHANGELOG.md"]
    message: "Update CHANGELOG for this change"
```

### 6. SEQUENCING RAILS
```yaml
rails:
  sequence_install_first:
    before: [quality_tests, quality_lint, quality_build]
    require: env_deps_installed
    message: "Install dependencies first"

  sequence_server_for_e2e:
    before: [verify_app_loads, verify_user_flow]
    require: env_server_running
    message: "Start server before E2E tests"

  sequence_tests_before_commit:
    before: [commit]
    require: [quality_tests, quality_lint]
    message: "Tests must pass before commit"

  sequence_build_before_deploy:
    before: [deploy]
    require: [quality_build, quality_tests]
    message: "Build must succeed before deploy"
```

---

## Rail Selection: How Do We Know Which to Use?

### Option 1: Project Type Detection

```yaml
# Auto-detect and load relevant rails
project_types:
  node_app:
    detect:
      - file_exists: package.json
      - file_contains: [package.json, "express|react|next"]
    rails:
      - env_node
      - env_deps_installed
      - quality_tests
      - quality_lint

  python_app:
    detect:
      - file_exists: [requirements.txt, pyproject.toml, setup.py]
    rails:
      - env_python
      - env_venv_active
      - quality_pytest
      - quality_mypy

  static_site:
    detect:
      - file_exists: index.html
      - not_file_exists: package.json
    rails:
      - verify_html_valid
      - verify_links_work

  database_app:
    detect:
      - file_contains: [package.json, "prisma|sequelize|typeorm"]
      - or_file_exists: [schema.prisma, models/]
    rails:
      - env_db_connected
      - safety_migration_backup
```

### Option 2: Task Type Matching

```yaml
# Different rails for different tasks
task_types:
  new_feature:
    description: "Adding new functionality"
    rails:
      - intent_defined
      - quality_tests      # Must write tests
      - quality_lint
      - docs_updated
      - verify_user_flow   # E2E for new feature
    cadence:
      test_every: 3 edits
      commit_every: 5 edits

  bug_fix:
    description: "Fixing existing bug"
    rails:
      - intent_defined     # What bug?
      - quality_tests      # Test reproduces bug, then passes
      - safety_no_regression
    cadence:
      test_every: 2 edits  # Tighter for bug fixes
      commit_every: 3 edits

  refactor:
    description: "Restructuring without behavior change"
    rails:
      - safety_backup_exists  # Backup first!
      - quality_tests         # Tests must stay green
      - verify_no_behavior_change
    cadence:
      test_every: 1 edit   # Test after EVERY change
      commit_every: 3 edits

  deployment:
    description: "Shipping to production"
    rails:
      - quality_build
      - quality_tests
      - verify_staging_works
      - safety_backup_exists
      - docs_changelog_updated
    require_manual_approval: true

  exploration:
    description: "Trying things out, not production"
    rails:
      - intent_defined     # Still need intent
      # Relaxed everything else
    cadence:
      test_every: 10 edits
      commit_every: 10 edits
```

### Option 3: Intent Analysis (LLM Decides)

```yaml
# LLM analyzes intent and selects rails
intent_analysis:
  prompt: |
    User intent: {intent}
    Project type: {detected_project_type}
    Files in scope: {files}

    Select which rails should be active:

    ENVIRONMENT:
    - [ ] env_server_running (if building web app)
    - [ ] env_db_connected (if using database)

    QUALITY:
    - [ ] quality_tests (if tests exist or should exist)
    - [ ] quality_lint (if linter configured)

    VERIFICATION:
    - [ ] verify_app_loads (if has UI)
    - [ ] verify_manual (if user should check)

    SAFETY:
    - [ ] safety_backup_exists (if major changes)
    - [ ] safety_no_secrets (always for commits)

    Return selected rails as JSON array.
```

---

## Agents vs Skills vs Rails

```
┌─────────────────────────────────────────────────────────────────┐
│                        TERMINOLOGY                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  RAILS = Hard constraints (MUST do X before Y)                 │
│          "You cannot commit until tests pass"                   │
│          Enforced by hooks, cannot be bypassed                 │
│                                                                 │
│  SKILLS = Capabilities (HOW to do X)                           │
│          "Here's how to run Playwright tests"                   │
│          Knowledge + scripts + templates                        │
│                                                                 │
│  AGENTS = Workers (DO X autonomously)                          │
│          "Run tests and fix any failures"                       │
│          LLM + tools + decision making                         │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  EXAMPLE FLOW:                                                 │
│                                                                 │
│  1. RAIL triggers: "Must run E2E tests"                        │
│                         │                                       │
│                         ▼                                       │
│  2. SKILL provides: "Playwright test scripts for this project"│
│                         │                                       │
│                         ▼                                       │
│  3. AGENT executes: "Run tests, analyze failures, fix code"    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Do We Care About the Distinction?

**YES.** Because:

| Type | Who Controls | Can Bypass? | Needs LLM? |
|------|--------------|-------------|------------|
| Rail | Hook script | NO | No |
| Skill | LLM choice | Yes | Yes |
| Agent | LLM autonomy | Yes | Yes |

**Rails are the only thing that actually constrain the LLM.**

Skills and agents are just capabilities - the LLM can choose to use them or not. Rails are enforced by the system, not the LLM.

---

## Importing Existing Rails

### From wshobson/agents Repository

```yaml
# Import rails from existing agent definitions
imports:
  - source: github:wshobson/agents
    agents:
      - code-quality-agent → rails: [lint, type-check, test]
      - security-scanner → rails: [no-secrets, no-vulnerabilities]
      - test-runner → rails: [unit-tests, e2e-tests]
      - documentation-agent → rails: [readme-exists, api-docs]

# Map agent capabilities to rails
agent_to_rail_mapping:
  code-quality-agent:
    provides_rails:
      - quality_lint
      - quality_types
      - quality_tests
    trigger_on:
      - pre_commit
      - file_change: ["*.ts", "*.js"]

  security-scanner:
    provides_rails:
      - safety_no_secrets
      - safety_no_vulnerabilities
    trigger_on:
      - pre_commit
      - pre_push
```

### Rail Registry Format

```yaml
# .aria/rails/quality_tests.yaml
rail:
  name: quality_tests
  description: "Ensure tests pass before continuing"
  category: quality

  # When does this rail apply?
  applies_when:
    project_has: [package.json, pytest.ini, Cargo.toml]
    task_type: [feature, bug_fix, refactor]

  # How to check
  check:
    node: "npm test"
    python: "pytest"
    rust: "cargo test"
    auto: "detect_test_command()"

  # What constitutes passing
  expect:
    exit_code: 0

  # What to block
  blocks:
    - commit
    - merge
    - deploy

  # How to fix
  fix_suggestions:
    - "Review test output above"
    - "Run tests locally: {check_command}"
    - "Fix failing tests before continuing"

  # Related agents/skills
  uses:
    skill: test-runner
    agent: test-fixer (if auto_fix enabled)
```

---

## Complete Rail Set for Non-Coder

```yaml
# Minimum viable rails for someone who doesn't code

non_coder_rails:

  # BEFORE STARTING
  - intent_defined:
      prompt: "What are you trying to build? Be specific."
      required: true

  # DURING DEVELOPMENT
  - server_running:
      check: "Is the app accessible at localhost?"
      cadence: before_any_ui_check
      auto_start: true  # Claude starts it if not running

  - tests_exist:
      check: "Are there tests for new code?"
      cadence: every_feature
      auto_create: true  # Claude writes tests

  - tests_pass:
      check: "Do all tests pass?"
      cadence: every_3_edits
      block: commit

  - user_can_see:
      type: playwright
      check: "Does the app load in browser?"
      cadence: every_ui_change
      screenshot: true  # Show user what it looks like

  # VERIFICATION
  - user_confirms:
      type: manual
      prompt: |
        I made these changes: {change_summary}

        Please check: {url}

        Does it look right? (yes/no/describe problem)
      cadence: before_commit
      required: true

  # SAFETY
  - no_disasters:
      block_commands: [rm -rf, DROP, --force]
      warn_commands: [delete, remove, reset]
      require_confirm: true

  - backup_exists:
      check: "Is there a recent commit/stash?"
      before: major_changes
      auto_create: true

  # COMPLETION
  - intent_satisfied:
      type: llm_verify
      prompt: "Does what we built match what you asked for?"
      required: true
      before: done
```

---

## How Claude Knows Which Rails

```
┌─────────────────────────────────────────────────────────────────┐
│                    RAIL SELECTION FLOW                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. USER STATES INTENT                                         │
│     "Build me a todo app with React"                           │
│                         │                                       │
│                         ▼                                       │
│  2. AUTO-DETECT PROJECT TYPE                                   │
│     - Has package.json? → Node                                 │
│     - Has React? → Frontend                                    │
│     - Has database config? → Needs DB rails                    │
│                         │                                       │
│                         ▼                                       │
│  3. LLM ANALYZES INTENT                                        │
│     - New feature? → Feature rails                             │
│     - Has UI? → E2E rails                                      │
│     - User is non-coder? → More verification rails             │
│                         │                                       │
│                         ▼                                       │
│  4. LOAD RAIL SET                                              │
│     active_rails = [                                           │
│       intent_defined,                                          │
│       env_node,                                                │
│       env_deps_installed,                                      │
│       server_running,                                          │
│       quality_tests,                                           │
│       verify_app_loads,                                        │
│       user_confirms,                                           │
│       safety_no_secrets                                        │
│     ]                                                          │
│                         │                                       │
│                         ▼                                       │
│  5. EXECUTE WITH RAILS ACTIVE                                  │
│     - Hook checks rails before each action                     │
│     - Block if rail not satisfied                              │
│     - Suggest fixes when blocked                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Next Step

Should I build this as actual code?

1. **Rail registry loader** - Parse YAML rail definitions
2. **Project detector** - Auto-detect project type and load relevant rails
3. **Rail executor** - Hook that checks active rails
4. **User verification** - Playwright + screenshot + prompt

This would be ~500 lines of real code, not just documentation.
