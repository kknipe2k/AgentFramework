# Prototyping Skill

> Generate prototype specifications for implementation by executing.md

## Purpose

This skill creates a **prototype specification** (not code). The spec is then handed to `executing.md` which uses the standard agent loop (analyzer → implementer → verify-app → verify.sh) to build and verify the prototype.

**This skill outputs:** `.aria/prototypes/SPEC-[name].json`
**Building happens in:** `executing.md` with standard agent pattern

---

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

---

## Prototype Types

| Type | Spec Output | Best For |
|------|-------------|----------|
| HTML Mockup | Component/screen definitions | Web UI, dashboards |
| ASCII Wireframe | Text layout (inline, no spec) | Quick CLI discussion |
| API Spec | Endpoint definitions | REST/GraphQL APIs |
| CLI Spec | Command structure | CLI tools |
| Data Model | Schema definitions | Database design |

---

## Spec Generation Flow

```
1. User describes what they want
2. Determine prototype type and variant
3. Generate SPEC-[name].json with requirements
4. Present spec for approval
5. Hand off to executing.md for implementation
```

---

## Prototype Variants

When user requests a prototype, determine the variant:

| Variant | Description | Spec Focus |
|---------|-------------|------------|
| **[1] Working mockup** | Minimal functional demo | Core interactions only |
| **[2] Learning tool** | Guided, interactive, educational | Full interactivity, tooltips, progressive disclosure |
| **[3] Reference impl** | Production-style patterns | Extensible structure, best practices |

**Learning tools require the most comprehensive spec** - every interaction must be defined.

---

## Spec Schema: HTML Prototype

Output this JSON to `.aria/prototypes/SPEC-[name].json`:

```json
{
  "id": "prototype-YYYYMMDD-HHMMSS",
  "name": "Descriptive name",
  "type": "html-mockup",
  "variant": "mockup|learning-tool|reference",
  "created": "ISO timestamp",
  "source": "IDEA.md reference or user request",

  "design": {
    "theme": "light",
    "colorPalette": {
      "background": "#ffffff",
      "surface": "#fafafa",
      "primary": "#2563eb",
      "text": "#333333",
      "border": "#e8e8e8"
    },
    "typography": {
      "fontFamily": "system-ui, sans-serif",
      "lineHeight": 1.6
    },
    "spacing": {
      "containerMaxWidth": "1100px",
      "containerPadding": "40px 32px"
    }
  },

  "screens": [
    {
      "id": "screen-1",
      "name": "Main Dashboard",
      "route": "/",
      "layout": "grid|flex|single-column",
      "components": [
        {
          "id": "comp-1",
          "type": "header|card|form|table|chart|nav|modal",
          "name": "Component name",
          "content": {
            "title": "Section title",
            "description": "What this shows",
            "data": ["Example data items"]
          },
          "interactions": [
            {
              "trigger": "click|hover|change|submit",
              "element": "button#id or .class",
              "action": "navigate|toggle|calculate|display|validate",
              "target": "target element or screen",
              "details": "What happens"
            }
          ],
          "states": {
            "default": "Initial state",
            "hover": "Hover state",
            "active": "Active/pressed state",
            "disabled": "Disabled state",
            "loading": "Loading state",
            "error": "Error state",
            "success": "Success state"
          }
        }
      ]
    }
  ],

  "interactions": {
    "navigation": [
      {"from": "screen-1", "to": "screen-2", "trigger": "click on X"}
    ],
    "dataFlow": [
      {"input": "form-field", "transform": "calculation", "output": "display-area"}
    ]
  },

  "testRequirements": {
    "playwright": [
      {
        "name": "Test name",
        "steps": ["step 1", "step 2"],
        "assertions": ["expected result"]
      }
    ],
    "accessibility": ["WCAG requirements"],
    "responsive": ["breakpoints to test"]
  },

  "qualityGates": {
    "allTabsWork": true,
    "allDropdownsWork": true,
    "allButtonsWork": true,
    "allFormsValidate": true,
    "noConsoleErrors": true,
    "accessibilityPasses": true
  }
}
```

---

## Spec Schema: API Prototype

```json
{
  "id": "api-spec-YYYYMMDD-HHMMSS",
  "name": "API name",
  "type": "api-spec",
  "created": "ISO timestamp",

  "baseUrl": "/api/v1",
  "authentication": {
    "type": "bearer|api-key|oauth",
    "header": "Authorization",
    "format": "Bearer {token}"
  },

  "endpoints": [
    {
      "method": "GET|POST|PUT|DELETE",
      "path": "/resource",
      "description": "What it does",
      "parameters": {
        "query": [
          {"name": "param", "type": "string", "required": false, "default": "value"}
        ],
        "body": {
          "field": {"type": "string", "required": true}
        }
      },
      "responses": {
        "200": {"description": "Success", "schema": {}},
        "400": {"description": "Bad request"},
        "401": {"description": "Unauthorized"}
      }
    }
  ],

  "testRequirements": {
    "endpoints": ["list of endpoints to test"],
    "errorHandling": ["error cases to verify"],
    "authentication": ["auth scenarios"]
  }
}
```

---

## Spec Schema: CLI Prototype

```json
{
  "id": "cli-spec-YYYYMMDD-HHMMSS",
  "name": "tool-name",
  "type": "cli-spec",
  "created": "ISO timestamp",

  "installation": "npm install -g tool-name",
  "commands": [
    {
      "name": "init",
      "description": "Initialize a new project",
      "usage": "tool-name init [project-name] [options]",
      "arguments": [
        {"name": "project-name", "required": false, "default": "."}
      ],
      "options": [
        {"flag": "--template, -t", "description": "Template to use", "default": "basic"},
        {"flag": "--force, -f", "description": "Overwrite existing files"}
      ],
      "examples": [
        "tool-name init my-app",
        "tool-name init my-app --template=typescript"
      ]
    }
  ],

  "testRequirements": {
    "commands": ["commands to test"],
    "flags": ["flag combinations"],
    "errors": ["error cases"]
  }
}
```

---

## Learning Tool Spec Requirements

**Learning tools have additional mandatory spec fields:**

```json
{
  "variant": "learning-tool",

  "educationalContent": {
    "concepts": [
      {
        "id": "concept-1",
        "name": "Concept name",
        "explanation": "Plain English explanation",
        "formula": "Mathematical formula if applicable",
        "variables": [
          {"symbol": "x", "meaning": "What x represents", "example": "e.g., 10"}
        ],
        "examples": [
          {"input": "Sample input", "output": "Expected output", "explanation": "Why"}
        ]
      }
    ],
    "progressiveDisclosure": [
      {"level": 1, "shows": "Basic concept"},
      {"level": 2, "shows": "Detailed explanation"},
      {"level": 3, "shows": "Worked examples"},
      {"level": 4, "shows": "Interactive simulation"}
    ]
  },

  "interactiveElements": {
    "tabs": [
      {"id": "tab-1", "label": "Tab name", "contentId": "content-1"}
    ],
    "tooltips": [
      {"target": ".variable", "content": "Explanation text"}
    ],
    "sliders": [
      {"id": "slider-1", "min": 0, "max": 100, "affects": "calculation-output"}
    ],
    "simulations": [
      {"id": "sim-1", "inputs": ["field1", "field2"], "calculation": "formula", "output": "result-area"}
    ]
  },

  "qualityGates": {
    "allTabsWork": true,
    "allDropdownsWork": true,
    "allButtonsWork": true,
    "allSimulationsCalculate": true,
    "allTooltipsShow": true,
    "progressiveDisclosureWorks": true,
    "noConsoleErrors": true,
    "accessibilityPasses": true,
    "playwrightTestsPass": true
  }
}
```

---

## ASCII Wireframe (Inline Only)

For quick text-based layouts in chat (no spec file needed):

```
+---------------------------------------+
|  Logo          [Search...]    [Login] |
+---------------------------------------+
|                                       |
|  +--------+  +--------+  +--------+   |
|  | Card 1 |  | Card 2 |  | Card 3 |   |
|  |        |  |        |  |        |   |
|  +--------+  +--------+  +--------+   |
|                                       |
|  [Load More]                          |
|                                       |
+---------------------------------------+
|  Footer links          (c) 2024       |
+---------------------------------------+
```

Use ASCII wireframes for:
- Quick concept validation in chat
- CLI/terminal output mockups
- No need for actual implementation

---

## Design Constraints (Carry to Spec)

**MANDATORY: LIGHT MODE ONLY**

All specs must include:
```json
"design": {
  "theme": "light",
  "colorPalette": {
    "background": "#ffffff",  // NOT #1a1a1a
    "surface": "#fafafa",     // NOT #2d2d2d
    "text": "#333333"         // NOT #ffffff
  }
}
```

| DO | DON'T |
|----|-------|
| `background: #ffffff` | `background: #1a1a1a` |
| `background: #fafafa` | `background: #2d2d2d` |
| Light, breathable | Dark, heavy |

**Why:** Light mode is better for screenshots, annotations, and stakeholder presentations.

---

## Handoff to executing.md

After spec approval:

```
PROTOTYPE SPEC READY: .aria/prototypes/SPEC-[name].json

Next: executing.md will build using standard agent loop:
1. analyzer - Review spec, plan implementation
2. implementer - Build each component
3. verify-app - Test interactions
4. verify.sh - Run linting, Playwright, accessibility

Proceed? [y]es / [r]evise spec / [c]ancel
```

**executing.md receives:**
- Spec file path
- Variant (mockup/learning-tool/reference)
- Quality gates to verify

---

## HITL Checkpoints

### HITL Required
- Before generating spec (confirm scope)
- If prototype has >10 screens/components
- If multiple prototypes needed
- Before handoff to executing.md

### Auto-Proceed (No HITL)
- Single screen mockup spec
- Simple API with <10 endpoints
- CLI with <5 commands
- Iterations on existing spec

---

## Save Location

```
.aria/prototypes/
+-- SPEC-[name].json           (prototype specification)
+-- SPEC-[name]-v2.json        (iterations)
+-- prototype-[name].html      (built by executing.md)
+-- prototype-[name]-v2.html   (iterations)
+-- tests/
|   +-- [name].spec.js         (Playwright tests)
+-- README.md                  (index of prototypes)
```

---

## Output Summary

After running this skill, you should have:

1. **Spec file** at `.aria/prototypes/SPEC-[name].json`
2. **User approval** on the spec
3. **Clear requirements** for executing.md to build

Then hand off to executing.md with:
```
Build prototype from spec: .aria/prototypes/SPEC-[name].json
Variant: [mockup|learning-tool|reference]
```

---

## Tips

- **Spec over code** - This skill writes requirements, not implementation
- **Be comprehensive** - Include all interactions, states, test requirements
- **Learning tools need more** - Every tooltip, every calculation, every tab
- **Real content** - Use realistic data in examples, not "Lorem ipsum"
- **Link to IDEA.md** - Reference decisions from brainstorming
- **Let executing.md build** - Trust the agent loop for implementation
