# ARIA Concise Syntax

The full YAML is for the executor. Here's the **concise syntax** the LLM actually writes:

---

## Quick Example

```aria
@plan "Add user authentication"

@intent
  + User can register with email/password
  + User can login and get JWT token
  + Protected routes require valid token
  - No plain text passwords
  - No tokens in URLs

@require node>=18, express, bcrypt, jsonwebtoken
@env JWT_SECRET

---

@phase setup
  > npm install bcrypt jsonwebtoken
  ? deps_installed: package.json has bcrypt, jsonwebtoken
  * checkpoint

@phase model
  @ src/models/User.js
    ```js
    const bcrypt = require('bcrypt');
    class User {
      static async create(email, pass) {
        return { email, hash: await bcrypt.hash(pass, 10) };
      }
    }
    module.exports = User;
    ```
  ? no_plaintext: !contains(User.js, "password =")
  * checkpoint

@phase routes
  @ src/routes/auth.js
    ```js
    const jwt = require('jsonwebtoken');
    router.post('/login', async (req, res) => {
      const token = jwt.sign({email}, process.env.JWT_SECRET);
      res.json({ token });
    });
    ```
  ? no_exposure: !contains(auth.js, "console.log.*token")
  * checkpoint

@phase tests
  @ tests/auth.test.js
    ```js
    test('login returns token', async () => {
      const res = await request(app).post('/login').send(creds);
      expect(res.body.token).toBeDefined();
    });
    ```
  * checkpoint

@phase verify
  ? tests: `npm test` == 0
  ? security: `npm audit` == 0
  ? intent: llm "Does this satisfy: {intent}?"

@done
  > git commit -m "feat: add JWT auth"
  > update docs/AUTH.md
```

---

## Syntax Reference

### Headers

```aria
@plan "Task description"      # Plan name
@intent                       # What we're building
  + must have this           # Required feature
  - must NOT have this       # Anti-requirement
@require pkg1, pkg2          # Dependencies
@env VAR1, VAR2              # Required env vars
```

### Phases

```aria
@phase name                  # Start a phase
  ...actions...
  * checkpoint               # Save state for rollback
```

### Actions (Rails)

```aria
# Run command
> npm install express

# Create/edit file
@ path/to/file.js
  ```lang
  content here
  ```

# Edit existing file (find/replace)
@ path/to/file.js
  - old content to find
  + new content to replace

# Delete file
@- path/to/remove.js

# Run with capture
> npm test >> $test_output
```

### Gates (Verification)

```aria
# Command exit code
? name: `command` == 0

# File content check
? name: contains(file.js, "pattern")
? name: !contains(file.js, "bad pattern")

# File exists
? name: exists(path/to/file.js)

# LLM verification
? name: llm "Question about intent?"

# Custom check
? name: schema_valid(api.json, openapi)
```

### Failure Handlers

```aria
? tests: `npm test` == 0
  | fail: rollback          # Rollback to last checkpoint
  | fail: pause             # Stop and wait for human
  | fail: retry(3)          # Retry 3 times
  | fail: skip              # Skip with warning
```

### Completion

```aria
@done                        # On success
  > git commit -m "message"
  > generate docs

@fail                        # On failure
  > rollback
  > notify team
```

---

## Expanded Examples

### API Endpoint

```aria
@plan "Add GET /users/:id endpoint"

@intent
  + Returns user by ID
  + Returns 404 if not found
  + Requires authentication
  - No sensitive data in response

---

@phase implement
  @ src/routes/users.js
    - // TODO: add get by id
    + router.get('/:id', auth, async (req, res) => {
    +   const user = await User.findById(req.params.id);
    +   if (!user) return res.status(404).json({error: 'Not found'});
    +   res.json({ id: user.id, name: user.name, email: user.email });
    + });
  ? no_password: !contains(users.js, "password")
  * checkpoint

@phase test
  @ tests/users.test.js
    ```js
    test('GET /users/:id returns user', async () => {
      const res = await request(app).get('/users/1').set('Auth', token);
      expect(res.status).toBe(200);
      expect(res.body).not.toHaveProperty('password');
    });

    test('GET /users/:id returns 404', async () => {
      const res = await request(app).get('/users/999').set('Auth', token);
      expect(res.status).toBe(404);
    });
    ```
  * checkpoint

@phase verify
  ? tests: `npm test` == 0
  ? intent: llm "Returns user by ID, 404 if missing, needs auth, no sensitive data?"

@done
  > git commit -m "feat(api): add GET /users/:id"
```

### Bug Fix

```aria
@plan "Fix: Login not hashing passwords"

@intent
  + Passwords are hashed before storage
  + Existing functionality unchanged
  - No plain text passwords anywhere

---

@phase fix
  @ src/auth/register.js
    - await db.users.insert({ email, password });
    + const hash = await bcrypt.hash(password, 10);
    + await db.users.insert({ email, password: hash });
  ? uses_bcrypt: contains(register.js, "bcrypt.hash")
  * checkpoint

@phase verify
  ? unit: `npm test -- register` == 0
  ? all: `npm test` == 0
  ? no_plain: !grep(src/**, "password.*=.*req.body")

@done
  > git commit -m "fix(auth): hash passwords on registration"
```

### Refactor

```aria
@plan "Refactor: Extract validation middleware"

@intent
  + Validation logic in middleware
  + Routes use middleware
  + All existing tests pass
  - No behavior change

---

@phase extract
  @ src/middleware/validate.js
    ```js
    const { validationResult } = require('express-validator');

    const validate = (req, res, next) => {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        return res.status(400).json({ errors: errors.array() });
      }
      next();
    };

    module.exports = validate;
    ```
  * checkpoint

@phase update_routes
  @ src/routes/users.js
    - const { validationResult } = require('express-validator');
    + const validate = require('../middleware/validate');

    - const errors = validationResult(req);
    - if (!errors.isEmpty()) {
    -   return res.status(400).json({ errors: errors.array() });
    - }
    + // validation handled by middleware

  @ src/routes/auth.js
    - const { validationResult } = require('express-validator');
    + const validate = require('../middleware/validate');
  * checkpoint

@phase verify
  ? tests: `npm test` == 0
    | fail: rollback
  ? no_behavior_change: diff_test(before, after) == identical

@done
  > git commit -m "refactor: extract validation middleware"
```

---

## Parallel Execution

```aria
@plan "Setup microservice infrastructure"

@phase setup [parallel]      # Run these in parallel
  @task api
    > mkdir -p services/api
    @ services/api/package.json
      ```json
      {"name": "api", "version": "1.0.0"}
      ```

  @task worker
    > mkdir -p services/worker
    @ services/worker/package.json
      ```json
      {"name": "worker", "version": "1.0.0"}
      ```

  @task gateway
    > mkdir -p services/gateway
    @ services/gateway/package.json
      ```json
      {"name": "gateway", "version": "1.0.0"}
      ```

? all_created: exists(services/*/package.json)
* checkpoint
```

---

## LLM Directives

Special directives for LLM behavior:

```aria
@llm.think            # LLM explains reasoning before action
@llm.verify           # LLM checks own work
@llm.cautious         # Extra verification on destructive ops
@llm.explain          # Generate inline comments

@phase implement
  @llm.think          # Before each action, explain why
  @ src/utils.js
    ```js
    // LLM will add explanation comments
    ```
```

---

## Why This Syntax?

| Symbol | Meaning | Rationale |
|--------|---------|-----------|
| `@` | Structural element | Markdown-familiar, visible |
| `>` | Run command | Shell convention |
| `?` | Verification gate | Question = check |
| `+` | Must have / Add | Diff convention |
| `-` | Must not / Remove | Diff convention |
| `*` | Checkpoint | Bookmark/star |
| `\|` | Failure handler | Pipe convention |
| `$` | Variable reference | Shell convention |

**Concise but unambiguous.** Every line has clear intent. No room for misinterpretation.
