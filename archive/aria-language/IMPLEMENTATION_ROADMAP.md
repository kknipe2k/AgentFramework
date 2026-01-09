# Aria Implementation Roadmap

A technical guide for implementing the Aria programming language.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Aria Source Code                         │
│                          (.aria files)                          │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                            LEXER                                │
│         Tokenizes source into tokens (keywords, symbols,        │
│         identifiers, literals, operators)                       │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                           PARSER                                │
│        Builds Abstract Syntax Tree (AST) with support for:      │
│        - Python-style indentation                               │
│        - JS-style expressions                                   │
│        - HTML-like component syntax                             │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      SEMANTIC ANALYZER                          │
│        - Type checking (gradual typing)                         │
│        - Scope resolution                                       │
│        - Decorator processing                                   │
│        - Component validation                                   │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      IR GENERATOR                               │
│        Generates intermediate representation                    │
└─────────────────────────────────────────────────────────────────┘
                                 │
            ┌────────────────────┼────────────────────┐
            ▼                    ▼                    ▼
   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
   │  WASM Backend   │  │   JS Backend    │  │ Native Backend  │
   │                 │  │                 │  │                 │
   │ WebAssembly for │  │ JavaScript for  │  │  LLVM for       │
   │ browsers/WASI   │  │ Node.js/Browser │  │  native binary  │
   └─────────────────┘  └─────────────────┘  └─────────────────┘
```

---

## Phase 1: Core Language Foundation

### 1.1 Lexer Implementation

**Tokens to recognize:**

```
Keywords:
  def, class, if, elif, else, for, while, return, yield,
  import, from, export, async, await, try, except, finally,
  match, case, const, component, state, computed, effect

Operators:
  +, -, *, /, //, %, **, ==, !=, <, >, <=, >=,
  and, or, not, in, is, =, +=, -=, *=, /=,
  =>, ??, ?., ...

Delimiters:
  (, ), [, ], {, }, :, ,, ., @, #

Literals:
  Integers, Floats, Strings (single, double, f-strings, template literals)
  Boolean (True, False), None

Special:
  INDENT, DEDENT, NEWLINE (for Python-style blocks)
```

**Key Implementation Notes:**
- Track indentation levels for Python-style blocks
- Support both `#` (Python) and `//` (JS) comments
- Handle f-strings and template literals with expression interpolation

### 1.2 Parser Implementation

**Grammar Highlights (EBNF-like):**

```ebnf
program        := statement*

statement      := import_stmt
               | export_stmt
               | function_def
               | class_def
               | component_def
               | assignment
               | expression_stmt
               | if_stmt
               | for_stmt
               | while_stmt
               | try_stmt
               | match_stmt
               | return_stmt

function_def   := decorator* "async"? "def" IDENTIFIER "(" params? ")"
                  ("->" type)? ":" block

class_def      := decorator* "class" IDENTIFIER ("(" bases ")")? ":" class_block

component_def  := "component" IDENTIFIER "(" params? ")" ":" component_block

# Arrow functions (JS-style)
arrow_function := "(" params? ")" "=>" (expression | "{" statement* "}")

# Destructuring
destructure    := "{" IDENTIFIER ("," IDENTIFIER)* "}" "=" expression
               | "[" pattern ("," pattern)* ("," "..." IDENTIFIER)? "]" "=" expression

# UI Elements (HTML-like)
ui_element     := "@" IDENTIFIER ("(" attributes ")")? ":" (ui_content | expression)
ui_content     := INDENT (ui_element | expression | ui_control)* DEDENT

# Control flow in UI
ui_control     := "@if" expression ":" ui_content ("@elif" expression ":" ui_content)* ("@else" ":" ui_content)?
               | "@for" IDENTIFIER "in" expression ("if" expression)? ":" ui_content
               | "@match" expression ":" match_ui_arms
```

### 1.3 AST Node Types

```python
# Core expression nodes
class Expression(ABC): pass
class Literal(Expression): pass
class Identifier(Expression): pass
class BinaryOp(Expression): pass
class UnaryOp(Expression): pass
class Call(Expression): pass
class Attribute(Expression): pass
class Subscript(Expression): pass
class Lambda(Expression): pass  # Arrow functions
class Comprehension(Expression): pass
class Await(Expression): pass
class Spread(Expression): pass  # ... operator
class OptionalChain(Expression): pass  # ?.
class NullishCoalesce(Expression): pass  # ??

# Statement nodes
class Statement(ABC): pass
class Assignment(Statement): pass
class FunctionDef(Statement): pass
class AsyncFunctionDef(Statement): pass
class ClassDef(Statement): pass
class ComponentDef(Statement): pass  # UI components
class If(Statement): pass
class For(Statement): pass
class While(Statement): pass
class Match(Statement): pass
class TryExcept(Statement): pass
class Return(Statement): pass
class Yield(Statement): pass
class Import(Statement): pass
class Export(Statement): pass

# UI-specific nodes
class UIElement(Expression): pass
class UIAttribute(Expression): pass
class UIConditional(Expression): pass
class UILoop(Expression): pass
class StateDeclaration(Statement): pass
class ComputedDeclaration(Statement): pass
class EffectDeclaration(Statement): pass
```

---

## Phase 2: Type System

### 2.1 Gradual Typing

Aria uses gradual typing - types are optional but encouraged.

```python
class Type(ABC): pass

class AnyType(Type): pass  # Default when no annotation
class NoneType(Type): pass
class BoolType(Type): pass
class IntType(Type): pass
class FloatType(Type): pass
class StringType(Type): pass
class ListType(Type):  # list[T]
    element_type: Type
class DictType(Type):  # dict[K, V]
    key_type: Type
    value_type: Type
class TupleType(Type):  # tuple[T1, T2, ...]
    element_types: list[Type]
class FunctionType(Type):
    param_types: list[Type]
    return_type: Type
    is_async: bool
class UnionType(Type):  # T1 | T2
    types: list[Type]
class OptionalType(Type):  # T | None
    inner_type: Type
class GenericType(Type):  # Generic[T]
    name: str
    type_params: list[str]
class ProtocolType(Type):  # Structural typing
    methods: dict[str, FunctionType]
class ComponentType(Type):
    props: dict[str, Type]
    returns: Type  # UIElement
```

### 2.2 Type Inference

```python
def infer_type(node: Expression, context: TypeContext) -> Type:
    match node:
        case Literal(value=int()):
            return IntType()
        case Literal(value=float()):
            return FloatType()
        case Literal(value=str()):
            return StringType()
        case Identifier(name=name):
            return context.lookup(name)
        case BinaryOp(op="+", left=left, right=right):
            left_type = infer_type(left, context)
            right_type = infer_type(right, context)
            return unify_types(left_type, right_type)
        case Call(func=func, args=args):
            func_type = infer_type(func, context)
            # Check argument types against function signature
            return func_type.return_type
        case Lambda(params=params, body=body):
            # Infer parameter types from usage
            return FunctionType(...)
        # ... more cases
```

---

## Phase 3: Async Runtime

### 3.1 Event Loop

```python
class EventLoop:
    """Core event loop for async operations"""

    def __init__(self):
        self.ready_queue: deque[Coroutine] = deque()
        self.waiting: dict[int, Coroutine] = {}
        self.io_selector = Selector()

    def run(self, main_coro: Coroutine):
        self.schedule(main_coro)
        while self.ready_queue or self.waiting:
            # Run ready coroutines
            while self.ready_queue:
                coro = self.ready_queue.popleft()
                self.run_once(coro)

            # Wait for I/O
            if self.waiting:
                events = self.io_selector.select(timeout=0.1)
                for key, _ in events:
                    coro = self.waiting.pop(key.fd)
                    self.schedule(coro)

    def schedule(self, coro: Coroutine):
        self.ready_queue.append(coro)

    def wait_for_io(self, fd: int, coro: Coroutine):
        self.waiting[fd] = coro
        self.io_selector.register(fd, EVENT_READ)
```

### 3.2 Parallel Execution

```python
async def parallel(*coroutines) -> list:
    """Run coroutines in parallel (like Promise.all)"""
    results = [None] * len(coroutines)
    pending = set(range(len(coroutines)))
    exceptions = []

    for i, coro in enumerate(coroutines):
        # Schedule all coroutines
        schedule(coro, callback=lambda r, idx=i: set_result(idx, r))

    # Wait for all to complete
    while pending:
        await suspend()

    if exceptions:
        raise ParallelError(exceptions)

    return results
```

---

## Phase 4: UI Component System

### 4.1 Virtual DOM

```python
class VNode:
    """Virtual DOM node"""
    tag: str
    props: dict
    children: list[VNode | str]
    key: any

def create_vnode(tag: str, props: dict, children: list) -> VNode:
    return VNode(tag, props, flatten(children), props.get("key"))

def diff(old: VNode, new: VNode) -> list[Patch]:
    """Calculate minimal patches to transform old tree to new"""
    patches = []

    if old is None:
        patches.append(CreatePatch(new))
    elif new is None:
        patches.append(RemovePatch(old))
    elif old.tag != new.tag:
        patches.append(ReplacePatch(old, new))
    else:
        # Diff props
        for key, value in new.props.items():
            if old.props.get(key) != value:
                patches.append(PropPatch(key, value))

        # Diff children (keyed diffing)
        patches.extend(diff_children(old.children, new.children))

    return patches
```

### 4.2 Reactive State

```python
class State[T]:
    """Reactive state container"""

    def __init__(self, initial: T):
        self._value = initial
        self._subscribers: set[Callable] = set()

    @property
    def value(self) -> T:
        # Track access for dependency collection
        if current_effect:
            current_effect.add_dependency(self)
        return self._value

    def set(self, new_value: T):
        if self._value != new_value:
            self._value = new_value
            self._notify()

    def _notify(self):
        for subscriber in self._subscribers:
            subscriber()

class Computed[T]:
    """Computed value that auto-updates when dependencies change"""

    def __init__(self, compute_fn: Callable[[], T]):
        self._compute_fn = compute_fn
        self._cached_value: T = None
        self._dirty = True
        self._dependencies: set[State] = set()

    @property
    def value(self) -> T:
        if self._dirty:
            self._recompute()
        return self._cached_value

    def _recompute(self):
        # Track dependencies during computation
        old_deps = self._dependencies
        self._dependencies = set()

        global current_effect
        current_effect = self

        self._cached_value = self._compute_fn()
        self._dirty = False

        current_effect = None

        # Unsubscribe from old deps, subscribe to new
        for dep in old_deps - self._dependencies:
            dep._subscribers.discard(self._mark_dirty)
        for dep in self._dependencies - old_deps:
            dep._subscribers.add(self._mark_dirty)
```

### 4.3 Component Rendering

```python
def render_component(component: ComponentDef, props: dict) -> VNode:
    """Render a component to virtual DOM"""

    # Create component context
    context = ComponentContext(props)

    # Execute component body
    with context:
        result = component.body(props)

    # Process UI elements
    return process_ui_element(result)

def process_ui_element(element: UIElement) -> VNode:
    """Convert Aria UIElement to VNode"""

    # Handle built-in HTML elements
    if element.tag.startswith("@"):
        tag = element.tag[1:]  # Remove @ prefix

        # Process attributes
        props = {}
        for attr in element.attributes:
            if attr.name.startswith("on"):
                # Event handler
                props[attr.name] = wrap_event_handler(attr.value)
            elif attr.name == "class" and isinstance(attr.value, dict):
                # Conditional classes
                props["class"] = compute_classes(attr.value)
            else:
                props[attr.name] = evaluate(attr.value)

        # Process children
        children = []
        for child in element.children:
            if isinstance(child, UIElement):
                children.append(process_ui_element(child))
            elif isinstance(child, UIConditional):
                children.extend(process_conditional(child))
            elif isinstance(child, UILoop):
                children.extend(process_loop(child))
            else:
                children.append(str(evaluate(child)))

        return create_vnode(tag, props, children)

    # Handle custom components
    else:
        component_def = lookup_component(element.tag)
        props = {attr.name: evaluate(attr.value) for attr in element.attributes}
        return render_component(component_def, props)
```

---

## Phase 5: Code Generation

### 5.1 JavaScript Backend

```python
class JSCodeGenerator:
    """Generate JavaScript from Aria AST"""

    def generate(self, node: AST) -> str:
        match node:
            case FunctionDef(name, params, body, is_async):
                async_kw = "async " if is_async else ""
                params_str = ", ".join(self.gen_param(p) for p in params)
                body_str = self.generate_block(body)
                return f"{async_kw}function {name}({params_str}) {{\n{body_str}\n}}"

            case Lambda(params, body):
                params_str = ", ".join(self.gen_param(p) for p in params)
                if isinstance(body, Block):
                    return f"({params_str}) => {{\n{self.generate_block(body)}\n}}"
                return f"({params_str}) => {self.generate(body)}"

            case ComponentDef(name, props, body):
                # Generate as React-like functional component
                return self.generate_component(name, props, body)

            case UIElement(tag, attrs, children):
                # Generate JSX or createElement calls
                return self.generate_jsx(tag, attrs, children)

            case Comprehension(expr, iterators, conditions):
                # Convert to .map().filter() chains
                return self.generate_comprehension(expr, iterators, conditions)

            # ... more cases

    def generate_component(self, name, props, body):
        """Generate React-compatible component"""
        code = f"function {name}({{{', '.join(p.name for p in props)}}}) {{\n"

        # Process state declarations
        for stmt in body:
            if isinstance(stmt, StateDeclaration):
                code += f"  const [{stmt.name}, set{stmt.name.title()}] = useState({self.generate(stmt.initial)});\n"
            elif isinstance(stmt, ComputedDeclaration):
                code += f"  const {stmt.name} = useMemo(() => {self.generate(stmt.compute_fn)}, [{', '.join(stmt.deps)}]);\n"
            elif isinstance(stmt, EffectDeclaration):
                code += f"  useEffect(() => {{ {self.generate(stmt.effect_fn)} }}, [{', '.join(stmt.deps)}]);\n"

        # Generate return statement with JSX
        return_stmt = find_return(body)
        code += f"  return {self.generate(return_stmt.value)};\n"
        code += "}"

        return code
```

### 5.2 WebAssembly Backend

```python
class WASMCodeGenerator:
    """Generate WebAssembly from Aria AST"""

    def __init__(self):
        self.module = WASMModule()
        self.functions = {}
        self.globals = {}
        self.memory = WASMMemory(initial=1)

    def generate(self, ast: Program) -> bytes:
        # First pass: collect all function/class definitions
        self.collect_definitions(ast)

        # Second pass: generate code
        for stmt in ast.statements:
            self.generate_statement(stmt)

        # Add runtime support functions
        self.add_runtime()

        return self.module.encode()

    def generate_function(self, func: FunctionDef):
        # Determine WASM types for parameters and return
        param_types = [self.to_wasm_type(p.type) for p in func.params]
        return_type = self.to_wasm_type(func.return_type)

        # Create function
        fn = self.module.add_function(func.name, param_types, return_type)

        # Generate body
        for stmt in func.body:
            self.generate_statement(stmt, fn)

        return fn
```

---

## Phase 6: Tooling

### 6.1 Package Manager (`aria`)

```bash
# Initialize a new project
aria init my-project

# Install packages
aria install lodash
aria install numpy --python
aria install react --js

# Run development server
aria dev

# Build for production
aria build --target web
aria build --target node
aria build --target native

# Run tests
aria test

# Format code
aria fmt

# Lint code
aria lint
```

### 6.2 IDE Integration

**Language Server Protocol (LSP) Features:**
- Syntax highlighting (TextMate grammar)
- Auto-completion
- Go to definition
- Find references
- Rename symbol
- Type information on hover
- Error diagnostics
- Code actions (quick fixes)
- Formatting

### 6.3 Debugger

```python
class AriaDebugger:
    """Source-level debugger for Aria"""

    def __init__(self):
        self.breakpoints: set[Location] = set()
        self.call_stack: list[Frame] = []
        self.variables: dict[str, any] = {}

    def set_breakpoint(self, file: str, line: int):
        self.breakpoints.add(Location(file, line))

    def step_over(self):
        """Execute current line, stepping over function calls"""
        pass

    def step_into(self):
        """Step into function calls"""
        pass

    def step_out(self):
        """Run until current function returns"""
        pass

    def evaluate(self, expr: str) -> any:
        """Evaluate expression in current context"""
        ast = parse_expression(expr)
        return evaluate(ast, self.current_frame.context)
```

---

## Implementation Technology Choices

| Component | Recommended Technology | Alternative |
|-----------|----------------------|-------------|
| Lexer/Parser | Rust (tree-sitter) | Python (PLY), TypeScript |
| Type Checker | Rust | TypeScript |
| JS Backend | Rust (SWC-based) | TypeScript |
| WASM Backend | Rust (walrus/wasm-encoder) | LLVM |
| Native Backend | LLVM | Cranelift |
| Runtime | Rust + V8/QuickJS | Deno |
| Package Manager | Rust | Go |
| Language Server | Rust (tower-lsp) | TypeScript |

---

## Testing Strategy

1. **Unit Tests**: Individual language features
2. **Integration Tests**: Multi-file programs
3. **Conformance Tests**: Language specification compliance
4. **Performance Benchmarks**: Comparison with Python/JS
5. **Fuzzing**: Random program generation for parser/compiler

---

## Community & Ecosystem

1. **Documentation Site**: Interactive tutorials and API docs
2. **Playground**: Browser-based code editor
3. **Package Registry**: `aria.dev/packages`
4. **Discord/Forum**: Community support
5. **GitHub**: Open source development

---

## Success Metrics

- Parse 10,000+ lines/second
- Type check 5,000+ lines/second
- Generate JS in <100ms for typical projects
- IDE response time <50ms
- Bundle size competitive with hand-written JS

---

*This roadmap provides a technical foundation for implementing Aria. Each phase can be developed incrementally, with the core language features prioritized first.*
