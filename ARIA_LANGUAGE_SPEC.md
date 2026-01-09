# Aria Programming Language Specification

## Vision: The Best of Three Worlds

**Aria** is a hybrid programming language that combines:
- **Python's** clean syntax and readability
- **JavaScript's** async capabilities and functional features
- **HTML's** declarative UI components

---

## Core Design Principles

1. **Readability First** - Python-style indentation, no semicolons, no curly braces
2. **Native Async** - First-class async/await with JS-style event loop
3. **Declarative UI** - HTML-like components embedded directly in code
4. **Type Flexibility** - Duck typing with optional type annotations
5. **Universal Runtime** - Compiles to both native and WebAssembly

---

## Syntax Overview

### Variables and Types

```aria
# Python-style variable declaration with optional types
name = "Aria"
count: int = 42
prices: list[float] = [19.99, 29.99, 39.99]

# JS-style destructuring
{name, age, city} = user_data
[first, second, ...rest] = items

# Immutable constants (JS const behavior)
const PI = 3.14159
const CONFIG = {
    debug: true,
    version: "1.0.0"
}
```

### Functions

```aria
# Python-style def with arrow function shorthand available
def greet(name: str) -> str:
    return f"Hello, {name}!"

# JS-style arrow functions for lambdas
double = (x) => x * 2
add = (a, b) => a + b

# Default parameters and spread operator
def create_user(name, age = 18, **options):
    return {name, age, ...options}

# Decorators (from Python)
@memoize
@log_calls
def expensive_calculation(n):
    return fibonacci(n)
```

### Async/Await (JavaScript-inspired)

```aria
# Native async functions
async def fetch_user(id: int) -> User:
    response = await http.get(f"/api/users/{id}")
    return response.json()

# Promise-style chaining also supported
fetch_data()
    .then((data) => process(data))
    .catch((err) => log_error(err))
    .finally(() => cleanup())

# Parallel execution
async def load_dashboard():
    # JS-style Promise.all equivalent
    [users, posts, stats] = await parallel(
        fetch_users(),
        fetch_posts(),
        fetch_stats()
    )
    return {users, posts, stats}
```

### Control Flow

```aria
# Python-style conditionals
if score >= 90:
    grade = "A"
elif score >= 80:
    grade = "B"
else:
    grade = "C"

# Pattern matching (Python 3.10+ inspired, enhanced)
match response:
    case {status: 200, data: content}:
        return content
    case {status: 404}:
        raise NotFoundError()
    case {status: code} if code >= 500:
        raise ServerError(code)
    case _:
        raise UnknownError()

# List comprehensions (Python)
squares = [x ** 2 for x in range(10)]
evens = [x for x in numbers if x % 2 == 0]

# Dict comprehensions
word_counts = {word: len(word) for word in words}
```

### Classes and Objects

```aria
# Python-style classes with JS-style privacy
class User:
    # Private fields (JS-style #)
    #password_hash: str

    def __init__(self, name: str, email: str):
        self.name = name
        self.email = email
        self.#password_hash = ""

    # Property decorators (Python)
    @property
    def display_name(self) -> str:
        return f"{self.name} <{self.email}>"

    # Static methods
    @staticmethod
    def validate_email(email: str) -> bool:
        return "@" in email

# JS-style object shorthand
name = "Alice"
age = 30
user = {name, age}  # Equivalent to {name: name, age: age}

# Spread operator for objects
defaults = {theme: "dark", lang: "en"}
settings = {...defaults, theme: "light"}
```

---

## Declarative UI Components (HTML-Inspired)

The killer feature: **native UI components** embedded in code.

### Basic Component Syntax

```aria
# Components use HTML-like syntax with @ prefix
component Button(text: str, on_click: fn, variant = "primary"):
    @button(class="btn btn-{variant}", onclick={on_click}):
        {text}

# Using components
def render_actions():
    return @div(class="actions"):
        Button("Save", on_click=save_data, variant="primary")
        Button("Cancel", on_click=cancel, variant="secondary")
```

### Reactive State

```aria
component Counter():
    # Reactive state (like React hooks)
    count = state(0)

    def increment():
        count.set(count.value + 1)

    return @div(class="counter"):
        @span: "Count: {count.value}"
        @button(onclick={increment}): "+"

# Computed values (like Vue computed)
component PriceDisplay(items: list):
    total = computed(() => sum(item.price for item in items))

    return @div:
        @span: "Total: ${total.value:.2f}"
```

### Conditional Rendering

```aria
component UserProfile(user: User | None):
    return @div(class="profile"):
        @if user:
            @h1: user.name
            @p: user.bio
        @else:
            @p(class="guest"): "Please log in"

        @for post in user.posts if user:
            PostCard(post=post)
```

### Component Composition

```aria
component App():
    user = state(None)
    theme = state("light")

    return @html(lang="en", data-theme={theme.value}):
        @head:
            @title: "My Aria App"
            @style: """
                .container { max-width: 1200px; margin: 0 auto; }
                .dark { background: #1a1a1a; color: #fff; }
            """
        @body(class={theme.value}):
            @div(class="container"):
                Navbar(user={user.value}, on_logout={() => user.set(None)})
                @main:
                    Router():
                        Route(path="/", component=Home)
                        Route(path="/about", component=About)
                        Route(path="/users/:id", component=UserDetail)
                Footer()
```

---

## Module System

```aria
# Python-style imports
from math import sqrt, pi
from collections import defaultdict

# JS-style named exports
export def helper_function():
    pass

export class DataProcessor:
    pass

# Default export
export default App

# Relative imports
from ./utils import format_date
from ../models import User, Post
```

---

## Error Handling

```aria
# Python-style try/except with JS-style finally guarantee
try:
    result = await risky_operation()
except NetworkError as e:
    log_error(f"Network failed: {e}")
    result = cached_fallback()
except ValidationError:
    raise
finally:
    cleanup_resources()

# Optional chaining (JS)
username = user?.profile?.name ?? "Anonymous"

# Result type for explicit error handling
def divide(a: float, b: float) -> Result[float, DivisionError]:
    if b == 0:
        return Err(DivisionError("Cannot divide by zero"))
    return Ok(a / b)

match divide(10, 0):
    case Ok(value):
        print(f"Result: {value}")
    case Err(error):
        print(f"Error: {error}")
```

---

## Generators and Iterators

```aria
# Python-style generators
def fibonacci():
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b

# Async generators
async def stream_data(url: str):
    async for chunk in http.stream(url):
        yield process_chunk(chunk)

# Iterator protocol
for i, fib in enumerate(fibonacci()):
    if i >= 10:
        break
    print(fib)
```

---

## Built-in Data Structures

```aria
# Lists (Python-style with JS methods available)
items = [1, 2, 3, 4, 5]
items.append(6)
doubled = items.map((x) => x * 2)
filtered = items.filter((x) => x > 2)
total = items.reduce((acc, x) => acc + x, 0)

# Dictionaries/Objects
config = {
    "name": "MyApp",
    "version": "1.0.0",
    "features": ["auth", "api", "ui"]
}

# Sets
unique_tags = {"python", "javascript", "aria"}
unique_tags.add("html")

# Tuples (immutable)
point = (10, 20)
x, y = point
```

---

## Standard Library Highlights

```aria
# HTTP client (async by default)
response = await http.get("https://api.example.com/data")
data = await http.post("/api/users", json={name: "Alice"})

# File operations
content = await file.read("config.json")
await file.write("output.txt", result)

# JSON handling (native)
obj = json.parse('{"key": "value"}')
text = json.stringify(obj, indent=2)

# Date/Time
now = datetime.now()
formatted = now.format("YYYY-MM-DD HH:mm:ss")
tomorrow = now + timedelta(days=1)

# Regular expressions
pattern = regex(r"\d{3}-\d{4}")
matches = pattern.find_all(text)
```

---

## Interoperability

```aria
# Import Python libraries
from python:numpy as np
from python:pandas as pd

# Import JavaScript packages
from js:lodash import debounce, throttle
from js:axios as http_client

# Import Web APIs
from web:dom import document, window
from web:fetch import fetch

# Call Python code
data = np.array([1, 2, 3, 4, 5])
mean = np.mean(data)

# Call JavaScript code
debounced_search = debounce(search, 300)
```

---

## Concurrency Model

```aria
# Async/await for I/O bound operations
async def fetch_all_users():
    return await parallel([
        fetch_user(1),
        fetch_user(2),
        fetch_user(3)
    ])

# Workers for CPU-bound operations
worker = spawn_worker(heavy_computation, data)
result = await worker.result()

# Channels for communication (Go-inspired)
channel = Channel[int](buffer_size=10)

async def producer():
    for i in range(100):
        await channel.send(i)
    channel.close()

async def consumer():
    async for value in channel:
        print(f"Received: {value}")
```

---

## File Extension

`.aria` or `.ar`

---

## Example: Full Application

```aria
# app.aria - A complete web application

from aria.http import Server, Router, json_response
from aria.db import Database
from aria.ui import render

# Database setup
db = Database("postgres://localhost/myapp")

# API Routes
router = Router()

@router.get("/api/users")
async def list_users(request):
    users = await db.query("SELECT * FROM users")
    return json_response(users)

@router.get("/api/users/:id")
async def get_user(request):
    user = await db.query_one(
        "SELECT * FROM users WHERE id = $1",
        request.params.id
    )
    if not user:
        return json_response({"error": "Not found"}, status=404)
    return json_response(user)

@router.post("/api/users")
async def create_user(request):
    data = await request.json()
    user = await db.query_one(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        data.name, data.email
    )
    return json_response(user, status=201)

# UI Components
component UserCard(user: dict):
    return @div(class="card"):
        @h3: user["name"]
        @p: user["email"]
        @button(onclick={() => delete_user(user["id"])}):
            "Delete"

component UserList():
    users = state([])
    loading = state(True)

    async def load_users():
        response = await http.get("/api/users")
        users.set(response.json())
        loading.set(False)

    # Effect hook (runs on mount)
    effect(load_users, [])

    return @div(class="user-list"):
        @if loading.value:
            @div(class="spinner"): "Loading..."
        @else:
            @for user in users.value:
                UserCard(user=user, key=user["id"])

component App():
    return @div(class="app"):
        @header:
            @h1: "User Management"
        @main:
            UserList()
        @footer:
            @p: "Built with Aria"

# Entry point
def main():
    # Start API server
    server = Server(router, port=8000)

    # Render UI (for SSR or hydration)
    html = render(App())

    print("Server running at http://localhost:8000")
    server.run()

if __name__ == "__main__":
    main()
```

---

## Why Aria?

| Feature | Python | JavaScript | HTML | **Aria** |
|---------|--------|------------|------|----------|
| Clean syntax | Yes | No | N/A | **Yes** |
| Async/await | Yes | Yes | N/A | **Yes** |
| Declarative UI | No | JSX | Yes | **Yes** |
| Type hints | Yes | TypeScript | No | **Yes** |
| List comprehensions | Yes | No | N/A | **Yes** |
| Destructuring | Limited | Yes | N/A | **Yes** |
| Native web support | No | Yes | Yes | **Yes** |
| Backend capable | Yes | Yes | No | **Yes** |

---

## Implementation Roadmap

### Phase 1: Core Language
- Lexer and parser for Aria syntax
- Basic type system
- Variable declarations and functions
- Control flow statements

### Phase 2: Advanced Features
- Async/await runtime
- Class system with decorators
- Module system and imports
- Pattern matching

### Phase 3: UI Components
- Component syntax parsing
- Virtual DOM implementation
- Reactive state system
- Event handling

### Phase 4: Ecosystem
- Package manager (`aria install`)
- Standard library
- Python/JS interop bridges
- IDE plugins and tooling

---

## Conclusion

Aria represents a vision for a language that feels natural for both backend and frontend development, combining the expressiveness of Python, the async power of JavaScript, and the declarative clarity of HTML into one cohesive experience.

*"Write once, run anywhere - beautifully."*
