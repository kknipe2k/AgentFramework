# Aria Programming Language

> *"Write once, run anywhere - beautifully."*

**Aria** is a hybrid programming language that combines the best features of Python, JavaScript, and HTML into one cohesive, expressive language.

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

## Key Features

### From Python
- Clean, indentation-based syntax
- List/dict comprehensions
- Decorators
- Pattern matching
- Context managers

### From JavaScript
- Arrow functions (`(x) => x * 2`)
- Async/await with Promise-style chaining
- Destructuring assignment
- Spread operator (`...`)
- Optional chaining (`?.`) and nullish coalescing (`??`)

### From HTML
- Declarative UI components with `@element` syntax
- Reactive state management
- Semantic structure
- Built-in web rendering

## Quick Example

```aria
from aria.http import get
from aria.ui import state, effect, render

# Async data fetching
async def fetch_users() -> list[dict]:
    response = await get("/api/users")
    return response.json()

# Reactive UI Component
component UserList():
    users = state([])
    loading = state(True)

    async def load():
        users.set(await fetch_users())
        loading.set(False)

    effect(load, [])

    return @div(class="user-list"):
        @if loading.value:
            @p: "Loading..."
        @else:
            @for user in users.value:
                @div(class="card", key={user["id"]}):
                    @h3: user["name"]
                    @p: user["email"]

# Render to DOM
render(UserList(), document.getElementById("root"))
```

## Documentation

- [Language Specification](./ARIA_LANGUAGE_SPEC.md) - Complete language design
- [Implementation Roadmap](./IMPLEMENTATION_ROADMAP.md) - Technical implementation guide
- [Examples](./examples/) - Code samples demonstrating features

## Examples

| File | Description |
|------|-------------|
| [01_basics.aria](./examples/01_basics.aria) | Variables, functions, control flow |
| [02_async.aria](./examples/02_async.aria) | Async/await, parallel execution, channels |
| [03_components.aria](./examples/03_components.aria) | UI components, reactive state |
| [04_classes.aria](./examples/04_classes.aria) | OOP, dataclasses, generics |
| [05_interop.aria](./examples/05_interop.aria) | Python/JavaScript interoperability |

## Design Philosophy

1. **Readability First** - Code should be easy to read and understand
2. **Progressive Complexity** - Simple things should be simple, complex things possible
3. **Universal Runtime** - Same code runs on browser, server, and native
4. **Interoperability** - Leverage existing Python and JavaScript ecosystems
5. **Type Safety** - Optional but encouraged gradual typing

## File Extension

`.aria` or `.ar`

---

*Aria is a conceptual language design exploring the synthesis of modern programming paradigms.*
