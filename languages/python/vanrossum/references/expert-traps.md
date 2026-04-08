# Expert Traps & Empirical Data⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​‌​‌‌​‍‌‌‌​​​‌‌‍‌​‌‌‌​‌‌‍‌​‌‌​​​​‍​​​​‌​‌​‍‌‌‌‌‌‌​​⁠‍⁠

Concrete examples, version-specific numbers, and the original BDFL reasoning behind Python's controversial design choices. Load this file when you need to justify a refactor, pick between container types, or explain a non-obvious gotcha in review.

---

## 1. `__slots__` — measured benefit has collapsed

Most tutorials still cite Python 3.9–3.10 numbers. CPython 3.11 rewrote object layout (PEP 684-adjacent work on per-interpreter state), which absorbed most of what `__slots__` used to win.

| Python | Memory saved per instance | Attribute-lookup speedup (standard build) | Speedup (optimized `--enable-optimizations --with-lto` build) |
|--------|---------------------------|-------------------------------------------|---------------------------------------------------------------|
| 3.9    | 80–216 bytes              | 62%                                       | 4%                                                            |
| 3.10   | 80–216 bytes              | 52%                                       | 8%                                                            |
| 3.11   | 40–64 bytes               | 13%                                       | 4%                                                            |
| 3.12   | 32–56 bytes               | 15%                                       | 0%                                                            |
| 3.13   | 40–64 bytes               | 3%                                        | 3%                                                            |

Source: cpython#136016, measured against 16-attribute classes.

**Decision rule**: On 3.11+, add `__slots__` only when (a) creating >100k instances in a hot loop, (b) you want to *forbid* attribute creation as a correctness constraint, or (c) you're writing a library meant to work on 3.9–3.10. Otherwise `__slots__` is premature optimization and it breaks `weakref`, multiple inheritance from slotted bases, and dict-based introspection patterns.

**`dataclass(slots=True)` gotcha**: `slots=True` creates a NEW class, which breaks `super()` with no arguments (`TypeError: super(type, obj): obj must be an instance or subtype of type`). Use two-arg `super(MyClass, self)` as the workaround (gh-90562).

---

## 2. Container choice — empirical results on 3.12

Measured with `sys.getsizeof` and `timeit` on a struct with 2 int fields (small) and 100 int fields (large).

| Container                              | Size (2 attrs) | Size (100 attrs) | Attr access | Notes |
|----------------------------------------|---------------:|-----------------:|------------:|-------|
| `NamedTuple`                           | 56 B           | 840 B            | ~29 ns      | Immutable, unpackable, tuple-compatible |
| `@dataclass` (default)                 | 344 B          | 3376 B           | ~23 ns      | Mutable, dict-backed, fastest attr access |
| `@dataclass(frozen=True, slots=True)`  | 48 B           | 832 B            | ~23 ns      | **Smallest AND fastest on 3.10+** |
| `dict`                                 | ~232 B         | large            | ~30 ns      | No type info, no `__repr__` help |

**Decision rule for containers**:

```
Do you need tuple-compatibility (unpacking, APIs that expect a tuple, tf.function)?
├─ YES → typing.NamedTuple
└─ NO → Does it represent validated input from an external boundary (HTTP, JSON, DB)?
         ├─ YES → pydantic.BaseModel (accept the ~10x creation cost for validation)
         └─ NO → Does it have methods, inheritance, or mutation?
                  ├─ YES → @dataclass (add slots=True if >10k instances)
                  └─ NO  → @dataclass(frozen=True, slots=True)  # the modern default
```

**`typing.TypedDict` is not a container** — it's a type annotation for plain dicts. It disappears at runtime and provides zero validation. Use it only when the data genuinely IS a dict (JSON from an API) and you want mypy to check key access.

**NamedTuple equality footgun**:

```python
from typing import NamedTuple
class Point(NamedTuple): x: int; y: int
class Color(NamedTuple): r: int; g: int
Point(1, 2) == Color(1, 2)  # True — because both are tuples (1, 2)
```

Frozen dataclasses have nominal equality (class identity matters). If you're using NamedTuple as a domain type, you're inviting this bug.

---

## 3. Why Guido rejected tail-call elimination (the full argument)

From neopythonic.blogspot.com/2009/04/tail-recursion-elimination.html:

1. **It's a language feature, not an optimization.** If CPython does TCO and Jython/IronPython/MicroPython don't, your code runs on one interpreter. Guido refuses to ship features that fragment the language across implementations. This is the same reason he resisted merging Stackless Python: it would be CPython-only.

2. **It destroys tracebacks in the most useful case.** "When a stack trace reports the 5 most recent frames, and the bug is in the 6th because of tail-recursion, you're going to have a bad day." Python's debugging story depends on stack frames matching call history.

3. **Tail calls inside `try:` blocks CANNOT be eliminated** — the exception handler requires a live frame. Naive decorator-based TCO libraries silently skip these, breaking exception semantics.

4. **Recursion that needs TCO is recursion that could have been a loop.** Guido's position: in an imperative language with mutable state, the demand for TCO comes from programmers bringing Scheme habits to Python. Rewrite as `while`. For non-tail recursion (tree walks), TCO doesn't help anyway.

**Practical consequence**: `sys.setrecursionlimit(10000)` is a code smell. The default is 1000 Python frames, and each frame consumes ~500 bytes of *C* stack on CPython. Raising it invites segfaults, which violate Guido's rule: "a core dump is never the user's fault."

---

## 4. Mutable default arguments — why this is not a bug

```python
def append_to(elem, lst=[]):       # BAD
    lst.append(elem)
    return lst

append_to(1)  # [1]
append_to(2)  # [1, 2]   ← the "default" is the same list object
```

This is **not** a bug. It's how `def` works: the default expression is evaluated *once*, at `def` time, and stored on `function.__defaults__`. If the default is `[]`, every call that doesn't pass `lst` shares that exact list object.

This is consistent with every other Python construct that evaluates at `def` time: decorators, class body statements, base class expressions, type annotations (pre-PEP 563). Guido refused to special-case default arguments because that would require either lazy evaluation (and a new concept) or copying (and a performance cost that most callers don't want).

**Idiom**:

```python
def append_to(elem, lst=None):
    if lst is None:
        lst = []
    lst.append(elem)
    return lst
```

**Uncommon correct use**: mutable defaults as a per-function cache (`def memo(x, _cache={}):`). Use sparingly and comment, because readers will assume it's the bug.

---

## 5. String concatenation — the CPython trap

```python
result = ""
for item in items:
    result += item        # LOOKS O(n²), RUNS O(n) on CPython, BREAKS on PyPy
```

Since CPython 2.4, there's an optimization: if `result`'s reference count is exactly 1 (no one else holds it), `+=` mutates the existing buffer in place instead of allocating a new string. This makes the naive loop O(n) — on CPython, in the current function, when the compiler can prove no aliasing.

The moment any of these becomes false — PyPy (doesn't have this opt), Jython (JVM strings are immutable), a debugger holding a reference, a closure capturing `result`, or a thread context switch — it silently becomes O(n²).

**Always**:
```python
result = "".join(items)           # for lists of str
result = "".join(str(x) for x in items)  # for mixed types
```

`io.StringIO` is the right choice when you're building incrementally and need to write via a file-like interface.

---

## 6. Dict ordering — the accidental guarantee

- **Python <3.6**: Dict ordering is officially undefined. `CPython 2.7`, `PyPy`, and `Jython` gave different iteration orders. Code that relied on insertion order was *broken*, it just didn't fail yet.
- **Python 3.6**: Dicts preserve insertion order as an **implementation detail**. This was an accidental consequence of Raymond Hettinger's compact-dict memory optimization. Documentation explicitly said: "do not rely on this."
- **Python 3.7+**: Insertion order became a **language guarantee** (by fiat). Every Python implementation must preserve it.

**Consequences a 10-year Python dev knows**:
- Code written for 3.6 that relied on order was technically undefined behavior — it happened to work.
- Guaranteeing order locked out future memory optimizations. Some core devs consider this a mistake.
- `collections.OrderedDict` still exists and still has value: it has `move_to_end()` and its `__eq__` is order-sensitive (`{1:'a',2:'b'} == {2:'b',1:'a'}` is True; `OrderedDict` of same is False). Use it when order is semantically meaningful to readers, not just an implementation detail of your loop.

---

## 7. `open()` encoding — the PEP 597 time bomb

```python
with open("data.txt") as f:     # BAD — uses locale encoding
    data = f.read()
```

On macOS/Linux this works because the locale is UTF-8. On Windows it's cp1252, Shift-JIS, or cp936 depending on system locale. Your code reads the same file and gets *different strings* or raises `UnicodeDecodeError`. PEP 597 exists because this is the single most common cross-platform Python bug.

**Always pass `encoding=`**:
```python
with open("data.txt", encoding="utf-8") as f:
    data = f.read()
```

Enable `-X warn_default_encoding` or `PYTHONWARNDEFAULTENCODING=1` in CI to catch every unannotated `open()`. Python 3.15 will make UTF-8 the default, but until then, be explicit.

**Bytes mode doesn't need encoding**: `open(p, "rb")` is fine — no decoding happens.

---

## 8. `functools.lru_cache` — three traps

```python
@functools.lru_cache(maxsize=None)   # TRAP 1: unbounded = memory leak
def expensive(x, y): ...

@functools.lru_cache                 # TRAP 2: default maxsize=128, almost always wrong
def expensive(x, y): ...

class Thing:
    @functools.lru_cache(maxsize=1024)  # TRAP 3: caches `self`, prevents GC
    def method(self, x): ...
```

**Fixes**:
1. If you want unbounded, say so: `@functools.cache` (3.9+). The name makes the lifetime choice visible in review.
2. Pick a maxsize that matches your real key distribution. 128 is a default picked for the doc example, not your workload.
3. Instance methods with `lru_cache` are a memory leak in disguise — the cache holds a reference to every `self` it's ever seen. Use `@cached_property` for per-instance caching, or make the method a module-level function keyed on a hashable value object.

---

## 9. `is` vs `==` — the only safe uses of `is`

```python
if x is None: ...     # CORRECT
if x is True: ...     # CORRECT
if x is False: ...    # CORRECT
if x is 100: ...      # WRONG — works on CPython by luck (small int cache), fails on PyPy
if x is "hello": ...  # WRONG — works on CPython if the string is interned, fails otherwise
```

CPython caches integers from -5 to 256 as singleton objects. It also interns string literals that look like identifiers. **These are implementation details.** The language guarantees `is` only for `None`, `True`, `False`, and the handful of documented singletons (`NotImplemented`, `Ellipsis`).

Modern Python (3.8+) raises `SyntaxWarning` on `is` with a literal. Treat it as an error.

---

## 10. The walrus operator — PEP 572's two canonical uses

Guido stepped down as BDFL over PEP 572, not because he opposed it (he co-authored it with Tim Peters), but because the *style debate* around it was exhausting. Use it ONLY in these two patterns:

```python
# Pattern 1: while-chunk loop
while chunk := stream.read(8192):
    process(chunk)

# Pattern 2: expensive-predicate filter
results = [y for x in data if (y := expensive(x)) is not None]
```

**Everything else is wrong style.** In particular:
- Using `:=` in comprehensions to "save a line" — noise.
- Using `:=` at top level of a statement — just use `=`.
- Using `:=` to bind multiple names in one expression — unreadable.

The PEP itself documents these restrictions. Reviewers should flag any `:=` that doesn't match the two canonical patterns.

---

## 11. EAFP vs LBYL — not just style, a race-condition fix

```python
# LBYL — has a TOCTOU race
if os.path.exists(path):
    with open(path) as f:    # file may have been deleted between the two lines
        data = f.read()

# EAFP — atomic
try:
    with open(path) as f:
        data = f.read()
except FileNotFoundError:
    data = default
```

This isn't aesthetic preference. It's a **correctness fix** for any check-then-act sequence on a resource that other processes can modify (files, network sockets, database rows, environment variables, subprocess state).

**LBYL is acceptable** when the check is O(1) and atomic with the use — e.g., `if key in local_dict: local_dict[key]`, because no other thread can modify `local_dict` between those two operations in CPython (due to the GIL's per-bytecode atomicity for dict operations). For threaded code on any non-CPython interpreter, even this breaks.

---

## 12. f-string debug specifier — underused

```python
x = 42
name = "Alice"
print(f"{x=}")            # x=42
print(f"{name=}")         # name='Alice'
print(f"{x + 1 = :>5}")   # x + 1 =    43
```

The `=` specifier (3.8+) prints `expression=repr(value)`. Expert use: temporary debug prints that document themselves when copy-pasted into issues. Strictly better than `print("x =", x)` — it's shorter AND records the expression.

Do NOT leave these in production code. They're debug tools.

---

## 13. Exception handling — the scope rule

```python
# BAD — try block too wide
try:
    response = requests.get(url)
    parsed = response.json()
    user = User(parsed["name"], parsed["email"])
    save(user)
except Exception as e:
    log.error("failed: %s", e)

# GOOD — narrow try, explicit else
try:
    response = requests.get(url)
except requests.RequestException as e:
    log.error("network: %s", e)
    return
else:
    parsed = response.json()  # only runs if no exception above

user = User(parsed["name"], parsed["email"])  # bugs here should NOT be caught
save(user)
```

The wide `try` catches bugs in `User()` construction or `save()` and logs them as "network errors," hiding real failures for months. The narrow `try/except/else` pattern:
1. Puts only the line that can fail in `try:`.
2. Uses `else:` to continue the happy path inside the no-exception scope.
3. Lets unrelated bugs crash loudly, which is Guido's explicit rule ("errors should never pass silently").

---

## 14. The BDFL "hints" — Guido's actual unstated rules

From python-history.blogspot.com/2009/01/pythons-design-philosophy.html (Guido's own words, not Tim Peters' Zen):

- "Borrow ideas from elsewhere whenever it makes sense."
- "Things should be as simple as possible, but no simpler." (Einstein)
- "Do one thing well" (Unix philosophy)
- "Don't fret too much about performance — plan to optimize later when needed."
- "Don't fight the environment and go with the flow."
- "Don't try for perfection because 'good enough' is often just that."
- "It's okay to cut corners sometimes, especially if you can do it right later."
- "Errors should not be fatal" (recoverable exceptions).
- "Errors should not pass silently" (never `except: pass`).
- "A bug in the user's Python code should not cause undefined interpreter behavior; a core dump is never the user's fault."
- "Punctuation characters should be used conservatively, in line with their common use in written English or high-school algebra."

These are the rules Guido actually used while designing the language — the Zen of Python was a retroactive codification by Tim Peters. When the Zen and these hints conflict (they sometimes do on "purity vs pragmatism"), Guido's pragmatism rules win.
