---
name: hettinger-idiomatic-python
description: >
  Write Python code in the style of Raymond Hettinger, Python core developer
  and author of collections, itertools, and functools. Emphasizes idiomatic
  transformations, iterator algebra, and leveraging the stdlib over hand-rolled
  logic. Use when refactoring Python to be more Pythonic, choosing between
  collections/itertools/functools abstractions, designing cooperative
  inheritance with super(), or transforming imperative loops into composable
  generator pipelines. Trigger keywords: Pythonic, idiomatic, itertools,
  collections, functools, lru_cache, defaultdict, Counter, namedtuple,
  generator pipeline, refactor Python, "there must be a better way".
---

# Hettinger-Style Idiomatic Python

## Thinking Framework

Before writing or refactoring any code, ask in order:

1. **Is there a stdlib tool for this?** Check `collections`, `itertools`, `functools` — Hettinger built them to eliminate hand-rolled logic. If you're writing a loop that counts, groups, caches, chains, or combines, the tool already exists.
2. **Am I working at the right abstraction level?** One `Counter` arithmetic expression replaces 15 lines of loop-and-dict code. Prefer the declarative operation.
3. **Can this be lazy?** If the consumer only needs one item at a time, yield — don't build a list. But if you need `len()`, indexing, or multiple passes, a list is correct.
4. **Would a 6-month-from-now reader understand this?** Hettinger's "Beyond PEP 8" point: a clever one-liner that takes 30 seconds to parse is worse than three clear lines. Chunk complex expressions into named sub-expressions.

## Expert Decision Trees

### Choosing a collections type

| Need | Use | NOT |
|---|---|---|
| Count occurrences | `Counter` | `dict` + manual increment |
| Group items by key | `defaultdict(list)` | `dict.setdefault()` — it creates a new list *every call* even on hits, just to discard it |
| Ordered keys (insert order + reorder) | `OrderedDict` — still needed for `move_to_end()` and equality semantics (order-sensitive `==`) | Plain `dict` if you need reordering or order-aware equality |
| Lightweight immutable record | `namedtuple` — 64 bytes/instance, `__slots__` automatic, picklable | `dataclass` when you need mutability or default values; `namedtuple` when you need tuple protocol (unpacking, hashing, use as dict keys) |
| FIFO/LIFO with O(1) ends | `deque` — `appendleft`/`popleft` are O(1) | `list` — `insert(0, x)` is O(n), silently quadratic on large data |
| Scope chain / layered config | `ChainMap` — lookups fall through parents; mutations only hit the first map | Merging dicts with `{**a, **b}` loses the ability to track which layer owns a key |

### Choosing between caching strategies

| Situation | Use | Why |
|---|---|---|
| Pure function, hashable args | `@lru_cache(maxsize=128)` or `@cache` | `@cache` is `lru_cache(maxsize=None)` — unbounded, faster, but leaks memory if args are diverse |
| Method on mutable instance | `@cached_property` | `lru_cache` on a method pins `self` in cache, preventing GC of the instance — silent memory leak |
| Class with `__slots__` | Stack `@property` over `@lru_cache` | `cached_property` writes to `__dict__` which doesn't exist on `__slots__` classes — raises `TypeError` |
| Function that returns generators | DO NOT cache | `lru_cache` returns the *same exhausted* generator object on subsequent calls — returns empty |
| Need TTL expiry | Roll your own with `time.monotonic()` | `lru_cache` has no expiry; stale data persists until `cache_clear()` |

### Generator vs List decision

- **Use generator when**: single pass, pipeline stage, data larger than memory, feeding `sum()`/`any()`/`all()`/`min()`/`max()`
- **Use list when**: need `len()`, indexing, multiple iteration, or the data will be consumed in a non-linear order
- **Trap**: a generator assigned to a variable and iterated twice silently yields nothing the second time — no error, just empty. If there's any chance of re-iteration, materialize with `list()`.

## NEVER (with reasons)

**NEVER use `@lru_cache` on a method** without understanding it pins `self`. The cache holds strong references to every unique `self` that calls it, so instances never get garbage-collected. For 10K objects each calling a cached method, that's 10K entries leaking. Use `@cached_property` for instance-level caching, or use a module-level function that takes the relevant data (not `self`) as args.

**NEVER use `groupby` on unsorted data** — it only groups *consecutive* identical keys, so `[A,A,B,A]` gives three groups (`A,A`, `B`, `A`), not two. Always `sorted(data, key=keyfunc)` first, or use `defaultdict(list)` if data arrives in arbitrary order.

**NEVER use `Counter.subtract()` and assume non-negative counts.** Unlike `-` operator which drops zero/negative, `subtract()` happily goes negative: `Counter(a=1).subtract(Counter(a=3))` → `Counter(a=-2)`. If you need floor-at-zero, use `counter1 - counter2` (the operator), or `+counter` to strip non-positives.

**NEVER pass `maxsize=None` to `lru_cache` on functions with unbounded argument domains** (e.g., string inputs from users). The cache grows without limit. Use bounded `maxsize` or `@cache` only when you know the domain is finite.

**NEVER build a `list` just to call `len()` on a generator.** Use `sum(1 for _ in gen)` for counting, or better, restructure to track count during generation.

**NEVER use `functools.total_ordering` in performance-hot paths.** It synthesizes comparison methods with two indirections per call (~2x slower than hand-written comparisons). Write all six comparison methods when objects are sorted in tight loops.

## Cooperative Inheritance (Hettinger's super() Protocol)

Before designing a class hierarchy, ask: "Will anyone compose this via multiple inheritance?" If yes, follow these three rules from Hettinger's "super() considered super":

1. **Every overriding method must call `super()`** — even if your immediate base is `object`. Breaking the chain silently drops all downstream mixins.
2. **Use `**kwargs` to forward unknown arguments** — the MRO can insert classes you didn't anticipate. Rigid positional signatures break when composed.
3. **Create a root class that stops the chain** — a `Root.draw()` that doesn't call `super().draw()` prevents `AttributeError` when `object` is reached.

To incorporate a non-cooperative third-party class, write an adapter that translates between the cooperative `**kwargs` protocol and the rigid class.

## Hettinger's Refactoring Heuristic

From "The Mental Game of Python":
- **Don't write `def` first.** Write the operation inline 2-3 times. Let the pattern emerge, *then* extract a function. Premature abstraction creates the wrong abstraction.
- **Chunk aggressively.** If a line has >7 symbols, alias sub-expressions into named variables. `uniform(50, 250)` beats `50 + random() * 200` — same result, half the cognitive load.
- **Let inheritance discover itself.** Build two concrete classes independently. The shared base class becomes obvious only after you see the duplication. Planning the hierarchy upfront yields unused methods and missing ones.

## Reference Loading

**For code examples and patterns**: READ `references/philosophy.md` when you need specific before/after code transformations or stdlib usage examples.

**Do NOT load** `references/philosophy.md` if you only need the decision trees above — they are self-contained.
