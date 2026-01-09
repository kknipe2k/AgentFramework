# Commit, Push, and Create PR

Commit all changes, push to remote, and create a pull request.

## Pre-computed context (runs before prompt)

```bash
echo "=== Git Status ==="
git status --short

echo ""
echo "=== Changes ==="
git diff --stat

echo ""
echo "=== Recent Commits ==="
git log --oneline -5

echo ""
echo "=== Current Branch ==="
git branch --show-current
```

## Instructions

1. Review the changes shown above
2. Create a descriptive commit message based on the diff
3. Commit all changes:
   ```bash
   git add -A
   git commit -m "your message"
   ```
4. Push to remote:
   ```bash
   git push -u origin $(git branch --show-current)
   ```
5. Create PR using gh CLI:
   ```bash
   gh pr create --fill
   ```

If tests haven't been run recently, run them first:
```bash
npm test
```

Output the PR URL when done.
