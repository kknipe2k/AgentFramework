# Verify All Changes

Run all verification checks on the current changes.

## Pre-computed context

```bash
echo "=== Files Changed ==="
git diff --name-only HEAD~1 2>/dev/null || git diff --name-only

echo ""
echo "=== Project Type ==="
if [ -f "package.json" ]; then echo "Node.js"; fi
if [ -f "tsconfig.json" ]; then echo "TypeScript"; fi
if [ -f "requirements.txt" ]; then echo "Python"; fi
```

## Instructions

Run these checks in order:

1. **Tests**
   ```bash
   npm test 2>/dev/null || pytest 2>/dev/null || echo "No test command found"
   ```

2. **Types** (if TypeScript)
   ```bash
   npx tsc --noEmit 2>/dev/null || echo "No TypeScript"
   ```

3. **Lint**
   ```bash
   npm run lint 2>/dev/null || echo "No lint command"
   ```

4. **Build**
   ```bash
   npm run build 2>/dev/null || echo "No build command"
   ```

5. **App loads** (if web app)
   ```bash
   curl -s -o /dev/null -w "%{http_code}" http://localhost:3000 2>/dev/null || echo "Server not running"
   ```

Report results in this format:

```
VERIFICATION RESULTS
====================
Tests:     ✅ PASS / ❌ FAIL
Types:     ✅ PASS / ❌ FAIL / ⏭️ N/A
Lint:      ✅ PASS / ❌ FAIL / ⏭️ N/A
Build:     ✅ PASS / ❌ FAIL / ⏭️ N/A
App:       ✅ PASS / ❌ FAIL / ⏭️ N/A
====================
Overall:   ✅ READY / ❌ FIX NEEDED
```
