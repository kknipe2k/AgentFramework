# Design Document: [Project Name]

> [One-line description of what this project builds]

---
version: 1.0.0
mode: FULL+
status: draft
created: [YYYY-MM-DD]
updated: [YYYY-MM-DD]
author: [AI-assisted / Human]
---

## 1. Overview

### 1.1 Problem Statement

[What problem does this project solve? Why is it needed?]

### 1.2 Goals

- [ ] [Primary goal]
- [ ] [Secondary goal]
- [ ] [Success metric]

### 1.3 Non-Goals

Things explicitly out of scope:

- [Non-goal 1]
- [Non-goal 2]

### 1.4 Project Sizing

| Factor | Value | Notes |
|--------|-------|-------|
| Estimated Tasks | | |
| Lines of Code | | |
| Files Affected | | |
| New Dependencies | | |
| Critical Systems | Auth / Payments / DB / None | |

**Calculated Mode:** FULL+

---

## 2. Architecture

### 2.1 System Diagram

```
[System architecture diagram]
```

### 2.2 Components

| Component | Purpose | Technology | Owner |
|-----------|---------|------------|-------|
| [Name] | [What it does] | [Tech stack] | [Team/person] |

### 2.3 Data Flow

1. **Input:** [Where data comes from]
2. **Processing:** [What happens to it]
3. **Output:** [Where it goes]

### 2.4 Key Decisions

| Decision | Options Considered | Chosen | Rationale |
|----------|-------------------|--------|-----------|
| [Decision 1] | A, B, C | B | [Why B was chosen] |

---

## 3. Security Considerations

### 3.1 Authentication

- [ ] Auth method: [OAuth / JWT / API Key / Session]
- [ ] Identity provider: [Provider name]

### 3.2 Authorization

| Role | Permissions | Scope |
|------|------------|-------|
| Admin | Full access | Global |
| User | Read/Write own | User-scoped |

### 3.3 "Don't Touch" Areas

Areas requiring HITL approval before modification:

1. **[Area 1]** - [Why it's sensitive]
2. **[Area 2]** - [Why it's sensitive]

---

## 4. Risk Assessment

### 4.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| [Risk 1] | High/Med/Low | High/Med/Low | [How to mitigate] |

### 4.2 Dependencies

| Dependency | Version | Risk Level | Notes |
|------------|---------|------------|-------|
| [Package] | [Version] | Low/Med/High | [Notes] |

### 4.3 Rollback Strategy

If deployment fails:

1. [Step 1]
2. [Step 2]
3. [Step 3]

---

## 5. Implementation Plan

### 5.1 Epics

| Epic | Description | Tasks | Priority |
|------|-------------|-------|----------|
| E1 | [Epic 1 description] | ~[N] | P0/P1/P2 |
| E2 | [Epic 2 description] | ~[N] | |

### 5.2 Epic Details

#### Epic 1: [Name]

**Goal:** [What this epic achieves]

**Tasks:**
1. [ ] Task 1.1 - [Description]
2. [ ] Task 1.2 - [Description]
3. [ ] Task 1.3 - [Description]

**HITL Checkpoints:**
- [ ] [Checkpoint requiring approval]

**Dependencies:** [What must be complete first]

---

### 5.3 Testing Strategy

| Test Type | Coverage Target | Tools |
|-----------|-----------------|-------|
| Unit | [%] | [Testing framework] |
| Integration | [%] | [Tools] |
| E2E | [Scenarios] | [Tools] |

---

## 6. Open Questions

Questions requiring human input before proceeding:

1. **[Question 1]**
   - Options: [A, B, C]
   - Recommendation: [Recommended option]
   - Impact: [What depends on this decision]

---

## 7. Review Checklist

Before approval, verify:

- [ ] All security considerations addressed
- [ ] "Don't touch" areas identified
- [ ] Rollback strategy defined
- [ ] All open questions resolved
- [ ] Epic breakdown is complete
- [ ] Dependencies documented
- [ ] HITL checkpoints identified

---

## Approval

```
HITL CHECKPOINT: Architecture Review

Status: pending

Proceed with implementation? [y]es / [r]evise / [c]ancel
```

---

*Template version 1.0.0 - See CLAUDE.md for FULL+ mode requirements*
