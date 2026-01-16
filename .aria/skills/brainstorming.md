# Brainstorming Skill

> Explore ideas before committing to implementation

## When to Use

Use this skill when:
- Starting a new project or major feature
- User says "brainstorm", "explore", "what are my options"
- Problem is unclear or has multiple approaches
- STANDARD/FULL/FULL+ mode (optional for LITE)

## Workflow

### Step 1: Understand the Problem (Socratic Questioning)

Ask clarifying questions to uncover the real need. Don't assume.

**Question Categories:**

| Category | Example Questions |
|----------|-------------------|
| **Goal** | "What does success look like?" |
| **Users** | "Who will use this? What's their skill level?" |
| **Constraints** | "Any tech requirements? Timeline? Budget?" |
| **Existing** | "What exists today? What's working/not working?" |
| **Scale** | "How many users? How much data?" |
| **Integration** | "What systems does this need to connect to?" |

**Rules:**
- Ask 3-5 questions max per round
- Group related questions together
- Don't ask what you can infer from context
- If user says "just figure it out", make reasonable assumptions and state them

### Step 2: Research Best Practices

Before proposing solutions, search for existing knowledge.

**When to Research:**
- Unfamiliar domain
- Security-sensitive (auth, payments, encryption)
- Performance-critical
- User explicitly asks for best practices

**How to Research:**
```
1. Web search: "[topic] best practices 2024" or "[framework] recommended approach"
2. Look for: Official docs, reputable blogs, Stack Overflow consensus
3. Note: What works, common pitfalls, trade-offs
```

**Skip Research When:**
- Simple/obvious task
- Already know the domain well
- User says "don't overthink it"
- LITE mode (unless explicitly requested)

### Step 3: Generate Options

Present 2-4 distinct approaches. Not slight variations—actually different strategies.

**Option Format:**
```
## Option A: [Name]

**Approach:** [1-2 sentence description]

**Pros:**
- Pro 1
- Pro 2

**Cons:**
- Con 1
- Con 2

**Best for:** [When this option shines]

**Estimated effort:** SMALL / MEDIUM / LARGE
```

### Step 4: Make a Recommendation

Don't just list options—recommend one.

```
**Recommendation:** Option B

**Why:** [2-3 sentences explaining why this fits the user's situation]

**Caveat:** [Any conditions that would change this recommendation]
```

### Step 5: Capture in IDEA.md

Save brainstorm output to `.aria/docs/IDEA.md` using the template:
- Problem statement
- Options considered
- Research findings
- Recommendation
- Open questions

---

## Socratic Question Templates

### For Vague Requests

User: "Build me an app"

```
To help brainstorm the right approach:

1. **Problem:** What problem does this app solve? Who has this problem?
2. **Users:** Who will use it? (internal team, customers, public?)
3. **Platform:** Web, mobile, desktop, or all?
4. **Data:** What data does it manage? Where does it come from?
5. **Scale:** Starting small or expecting high traffic?
```

### For Technical Decisions

User: "Should I use React or Vue?"

```
A few questions to find the best fit:

1. **Team:** What does your team already know?
2. **Project:** What type of app? (dashboard, e-commerce, content site?)
3. **Timeline:** Quick prototype or long-term product?
4. **Ecosystem:** Any required libraries that favor one framework?
```

### For Architecture Decisions

User: "How should I structure the backend?"

```
Let me understand the requirements:

1. **Scale:** How many concurrent users? Requests per second?
2. **Data:** Relational, document, or graph relationships?
3. **Auth:** User accounts needed? What auth method?
4. **Integration:** External APIs or services to connect?
5. **Deploy:** Cloud preference? Serverless ok?
```

---

## Web Search Integration

### Search Patterns

| Need | Search Query |
|------|--------------|
| Best practices | "[topic] best practices 2024" |
| Comparison | "[A] vs [B] pros cons" |
| How to | "[framework] how to [task]" |
| Security | "[topic] security considerations OWASP" |
| Performance | "[topic] performance optimization" |

### Evaluating Sources

**Trust hierarchy:**
1. Official documentation
2. Reputable tech blogs (Martin Fowler, Kent C. Dodds, etc.)
3. Stack Overflow (high-vote answers)
4. Recent conference talks
5. Medium/Dev.to (verify claims)

**Red flags:**
- Outdated (2+ years for fast-moving tech)
- No code examples
- Contradicts official docs
- SEO-farm vibes

---

## Citation Format

When citing sources, use this format:

**In-line:**
```
JWT tokens should be short-lived (Source: OWASP Auth Guidelines)
```

**In research section:**
```
## Research Findings

- **Best practice:** Use httpOnly cookies for session storage
  - Source: [OWASP Session Management Cheatsheet](https://cheatsheetseries.owasp.org/...)
  - Why: Prevents XSS from accessing tokens

- **Common pitfall:** Storing JWT in localStorage
  - Source: [Auth0 Blog - Token Storage](https://auth0.com/docs/...)
  - Risk: XSS can steal tokens
```

**Confidence levels:**
```
[HIGH] Official docs, OWASP, well-established patterns
[MEDIUM] Reputable blogs, Stack Overflow consensus
[LOW] Single source, opinion pieces, outdated
```

---

## Output

After brainstorming, you should have:

1. **Clear understanding** of the problem
2. **2-4 options** with trade-offs
3. **Research findings** with citations (if applicable)
4. **Recommendation** with reasoning
5. **IDEA.md** saved to `.aria/docs/`

Then hand off to next step based on workflow:
- **Build:** → Planning skill for task breakdown
- **Research:** → Prototyping skill (if prototype requested) → outputs SPEC-*.json

---

## Handoff to Prototyping

When prototype is requested after brainstorming:

1. User selects prototype variant at HITL: `[1] mockup / [2] learning tool / [3] reference`
2. Prototyping skill creates `SPEC-*.json` with requirements
3. Executing skill builds prototype via agent loop (analyzer → implementer → verify-app)
4. verify.sh runs HTML/CSS/JS/Playwright checks

---

## Tips

- **Don't over-brainstorm** - If the path is obvious, say so and move on
- **Time-box research** - 5-10 minutes max unless complex domain
- **State assumptions** - If you're guessing, say so
- **It's ok to say "I don't know"** - Then research or ask user
- **Bias toward action** - Brainstorming should enable building, not delay it
