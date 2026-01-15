# Prototyping Skill

> Generate quick visual or structural prototypes before implementation

## When to Use

Use this skill when:
- FULL+ mode (mandatory before coding)
- User asks for "mockup", "prototype", "wireframe", "sketch"
- Visual UI is core to the project
- API structure needs validation before building
- User says "show me what it would look like"

**Skip when:**
- LITE mode (unless explicitly requested)
- Pure backend/CLI with no UI
- User says "just build it"

## Prototype Types

| Type | Output | Best For |
|------|--------|----------|
| HTML Mockup | Single HTML file with inline CSS | Web UI, dashboards |
| ASCII Wireframe | Text-based layout | Quick CLI discussion |
| API Spec | Endpoint list with request/response | REST/GraphQL APIs |
| CLI Spec | Command structure with examples | CLI tools |
| Data Model | Schema diagram or table | Database design |

---

## HTML Mockup Generation

### When to Generate

- Web apps, dashboards, forms
- User needs to visualize layout
- Multiple screens/flows to validate

### Output Format

**⚠️ MANDATORY: LIGHT MODE ONLY**

> **DO NOT use dark backgrounds.** No `#1a1a1a`, `#2d2d2d`, `#333`, or similar dark colors for backgrounds. Prototypes must feel open, airy, and approachable.

| DO | DON'T |
|----|-------|
| `background: #ffffff` | `background: #1a1a1a` |
| `background: #fafafa` | `background: #2d2d2d` |
| `background: #f5f5f5` | `background: #333` |
| Light, breathable | Dark, heavy |

**Why:** Dark prototypes feel closed-in and harder to review. Light mode is better for screenshots, annotations, and stakeholder presentations.

---

Single self-contained HTML file:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>[Project] - Prototype</title>
    <style>
        /* Light mode, open feel - generous whitespace */
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: system-ui, sans-serif;
            background: #ffffff;
            color: #333;
            line-height: 1.6;
        }

        /* Airy container with breathing room */
        .container {
            max-width: 1100px;
            margin: 0 auto;
            padding: 40px 32px;
        }

        /* Clean, minimal cards */
        .card {
            background: #fafafa;
            border: 1px solid #e8e8e8;
            border-radius: 8px;
            padding: 24px;
            margin: 16px 0;
        }

        /* Subtle buttons */
        .btn {
            padding: 12px 24px;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 500;
        }
        .btn-primary { background: #2563eb; color: white; }

        /* Prototype indicator - less aggressive */
        .prototype-banner {
            background: #fef3c7;
            color: #92400e;
            text-align: center;
            padding: 8px;
            font-size: 14px;
            border-bottom: 1px solid #fcd34d;
        }
    </style>
</head>
<body>
    <div class="prototype-banner">⚠️ PROTOTYPE - Not functional</div>

    <div class="container">
        <!-- Screen content here -->
    </div>

    <script>
        // Minimal interactivity for demo purposes only
        // No real functionality
    </script>
</body>
</html>
```

### Rules

1. **Self-contained** - No external CSS/JS dependencies
2. **Clearly labeled** - Banner saying "PROTOTYPE - Not functional"
3. **Representative** - Show real content structure, not lorem ipsum
4. **Clickable states** - Show hover/active states if relevant
5. **Multiple screens** - Use sections or tabs to show different views
6. **LIGHT MODE ONLY** - White/light backgrounds, never dark themes

### Save Location

```
.aria/prototypes/
├── prototype-[name].html
├── prototype-[name]-v2.html  (iterations)
└── README.md                  (index of prototypes)
```

---

## ASCII Wireframe

For quick text-based layouts in chat:

```
┌─────────────────────────────────────────┐
│  Logo          [Search...]    [Login]   │
├─────────────────────────────────────────┤
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │  Card 1 │  │  Card 2 │  │  Card 3 │ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
│                                         │
│  [Load More]                            │
│                                         │
├─────────────────────────────────────────┤
│  Footer links          © 2024           │
└─────────────────────────────────────────┘
```

Use when:
- Quick concept validation
- CLI/terminal output
- No need for visual fidelity

---

## API Spec Generation

### Format

```markdown
## API Specification: [Name]

### Base URL
`https://api.example.com/v1`

### Authentication
Bearer token in Authorization header

### Endpoints

#### GET /users
List all users

**Query Parameters:**
| Param | Type | Required | Description |
|-------|------|----------|-------------|
| page | int | No | Page number (default: 1) |
| limit | int | No | Items per page (default: 20) |

**Response 200:**
```json
{
  "users": [
    { "id": 1, "name": "Alice", "email": "alice@example.com" }
  ],
  "total": 100,
  "page": 1
}
```

**Errors:**
| Code | Description |
|------|-------------|
| 401 | Unauthorized |
| 500 | Server error |

---

#### POST /users
Create a new user

**Request Body:**
```json
{
  "name": "string (required)",
  "email": "string (required)",
  "role": "string (optional, default: 'user')"
}
```

**Response 201:**
```json
{
  "id": 1,
  "name": "Alice",
  "email": "alice@example.com",
  "role": "user",
  "createdAt": "2024-01-15T10:30:00Z"
}
```
```

---

## CLI Spec Generation

### Format

```markdown
## CLI Specification: [tool-name]

### Installation
```bash
npm install -g tool-name
```

### Commands

#### tool-name init
Initialize a new project

```bash
tool-name init [project-name] [options]

Options:
  --template, -t   Template to use (default: basic)
  --force, -f      Overwrite existing files
  --dry-run        Show what would be created

Examples:
  tool-name init my-app
  tool-name init my-app --template=typescript
```

#### tool-name build
Build the project

```bash
tool-name build [options]

Options:
  --watch, -w      Watch for changes
  --minify         Minify output
  --output, -o     Output directory (default: dist)

Examples:
  tool-name build
  tool-name build --watch --output=public
```
```

---

## Auto-Proceed Logic

Prototyping should be fast. Don't over-ask.

### HITL Required

- Before generating (confirm scope)
- If prototype will be >500 lines
- If multiple prototypes needed

### Auto-Proceed (No HITL)

- Single screen mockup
- Simple API with <10 endpoints
- CLI with <5 commands
- Iterations on existing prototype

### Flow

```
1. User describes what they want
2. AI confirms scope: "I'll create a [type] prototype showing [screens/endpoints]"
3. Generate prototype (no approval needed)
4. Present: "Here's the prototype. [v]iew / [r]evise / [a]pprove to proceed?"
5. If approved → hand off to Planning skill
```

---

## Output

After prototyping, you should have:

1. **Prototype file(s)** in `.aria/prototypes/`
2. **User approval** to proceed
3. **Scope clarity** - what to build is now visual/concrete

Then hand off to Planning skill with the prototype as reference.

---

## Complex Prototypes

**Key constraint:** Claude's output limit is ~32K tokens ≈ 1,500-2,000 lines max per response.

For prototypes exceeding ~1,000 lines, build modularly - not because of coding best practice, but because Claude can't output large files in one response.

### Output Size Guidelines

| Expected Output | Approach |
|-----------------|----------|
| <1,000 lines | Single file OK |
| 1,000-1,500 lines | Risky, consider splitting |
| 1,500+ lines | **Must** build modularly |

### Real-World Context

| Prototype Type | Typical Size | Approach |
|----------------|--------------|----------|
| Simple mockup | 200-500 lines | Single file |
| Basic dashboard | 1,000-3,000 lines | Modular |
| Complex SPA | 5,000-15,000+ lines | Full modular |

### Modular Build Order

```
1. Shell/Layout      → prototype-shell.html (structure + nav)
2. Components        → component-[name].html (each major section)
3. Styles            → styles.css (separate file)
4. Interactivity     → app.js (separate file)
5. Assemble          → prototype-final.html (combine all)
```

### Why Modular

| Issue | One-Shot | Modular |
|-------|----------|---------|
| Token limits | Hits 32K limit, fails | Each piece <1,000 lines |
| Debugging | Hard to find issues | Isolate by component |
| Review | Overwhelming | Reviewable chunks |
| Iteration | Rewrite everything | Change one piece |

### Example Workflow

```
User: "Build a dashboard with 5 panels"

AI:
1. Create shell with navigation → prototype-shell.html
2. Panel 1: Session viewer      → component-session.html
3. Panel 2: Memory display      → component-memory.html
4. Panel 3: Event log           → component-events.html
5. Panel 4: Config              → component-config.html
6. Panel 5: Stats               → component-stats.html
7. Styles                       → dashboard.css
8. Interactivity                → dashboard.js
9. Assemble final               → prototype-dashboard.html
```

---

## Learning Tool: Math & Formulas

When building **[2] Learning tool** prototypes from technical papers:

**Expand all mathematical formulas visually:**

1. **Interactive formula cards:**
   ```html
   <div class="formula-card">
     <div class="formula">L = -Σ yᵢ log(ŷᵢ)</div>
     <div class="plain-english">Measures how wrong our predictions are</div>
     <div class="variables">
       <span class="var" data-tooltip="The loss value (lower = better)">L</span> =
       <span class="var" data-tooltip="Sum over all samples">-Σ</span>
       <span class="var" data-tooltip="Actual label (0 or 1)">yᵢ</span>
       <span class="var" data-tooltip="Natural log">log</span>
       <span class="var" data-tooltip="Predicted probability">ŷᵢ</span>
     </div>
     <div class="example">
       <button onclick="toggleExample()">Show Example</button>
       <div class="example-content">
         actual=1, predicted=0.9 → loss=0.105 ✓ (good!)
         actual=1, predicted=0.1 → loss=2.303 ✗ (bad!)
       </div>
     </div>
   </div>
   ```

2. **Required elements for each formula:**
   - Hover tooltips on every variable
   - Plain English translation
   - Concrete numeric example
   - Visual intuition (graphs/animations if helpful)
   - "Why it matters" context

3. **Progressive disclosure:**
   - Show formula first (collapsed)
   - Click to expand explanation
   - Click again for worked example
   - Advanced: interactive sliders to adjust values

---

## Learning Tool: FULLY FUNCTIONAL REQUIREMENT

**⚠️ MANDATORY: Learning tools are NOT mockups. They must be complete, working experiences.**

> A learning tool prototype with broken tabs, dead dropdowns, or non-functional buttons is **UNACCEPTABLE**. Every interactive element must work.

### Everything Must Work

| Element | Requirement |
|---------|-------------|
| **Tabs** | All tabs switch content correctly |
| **Dropdowns** | All options selectable, trigger appropriate actions |
| **Buttons** | Every button does something meaningful |
| **Click-throughs** | All navigation paths work end-to-end |
| **Simulations** | Interactive demos run correctly with real calculations |
| **Tooltips** | All hover states show helpful information |
| **Forms** | Inputs validate and respond appropriately |
| **Animations** | Transitions smooth, no janky behavior |

### Verification with Testing

**Before delivering a learning tool, run these checks:**

1. **Playwright E2E Tests:**
   ```javascript
   // test-learning-tool.spec.js
   test('all tabs are functional', async ({ page }) => {
     await page.goto('prototype.html');
     const tabs = await page.locator('[data-tab]').all();
     for (const tab of tabs) {
       await tab.click();
       await expect(page.locator('.tab-content.active')).toBeVisible();
     }
   });

   test('all dropdowns work', async ({ page }) => {
     const dropdowns = await page.locator('select').all();
     for (const dropdown of dropdowns) {
       const options = await dropdown.locator('option').all();
       expect(options.length).toBeGreaterThan(1);
     }
   });

   test('simulations calculate correctly', async ({ page }) => {
     await page.fill('#input-value', '10');
     await page.click('#calculate-btn');
     await expect(page.locator('#result')).not.toBeEmpty();
   });
   ```

2. **HTML/CSS Linting:**
   ```bash
   # Run before delivery
   npx htmlhint prototype.html
   npx stylelint "**/*.css"
   ```

3. **JavaScript Linting:**
   ```bash
   npx eslint app.js --fix
   ```

4. **Accessibility Check:**
   ```bash
   npx pa11y prototype.html
   ```

### Build Order for Learning Tools

```
1. Structure        → shell with all navigation elements
2. Styles           → CSS including hover/active states
3. Interactions     → JavaScript for ALL interactive elements
4. Content          → Real educational content, not placeholders
5. Tests            → Playwright tests for every interaction
6. Validation       → Lint + accessibility check
7. Manual QA        → Click through EVERYTHING yourself
8. Deliver          → Only after all tests pass
```

### Quality Gate

**DO NOT deliver until:**
- [ ] Every tab switches correctly
- [ ] Every dropdown has working options
- [ ] Every button triggers an action
- [ ] Every simulation runs real calculations
- [ ] Playwright tests pass
- [ ] Linting passes (HTML, CSS, JS)
- [ ] No console errors
- [ ] Accessibility check passes

**If any element is non-functional, FIX IT before delivery.**

---

## Skill Integration

**For Learning Tools, call these skills in order:**

| Step | Skill | Purpose |
|------|-------|---------|
| 1 | `prototyping.md` | Build the prototype |
| 2 | `executing.md` | Write Playwright tests |
| 3 | `verify.sh` | Run linting + tests |

**If verify.sh fails → fix and re-run. Do NOT deliver broken learning tools.**

---

## Tips

- **Fast over perfect** - Applies to mockups only, NOT learning tools
- **Real content** - Use realistic data, not "Lorem ipsum"
- **Show states** - Empty, loading, error, success
- **Mobile too** - If web, show responsive behavior
- **Link to IDEA.md** - Reference decisions from brainstorming
- **Math = explain** - Every formula needs plain English + example
- **Learning tools = fully functional** - No dead buttons, no broken tabs
