---
name: hettinger-idiomatic-python
description: >
  Encode Raymond Hettinger-style Python design judgment: iterator algebra,
  container choice, sorting and caching trade-offs, and cooperative
  inheritance. Use when refactoring loops into stdlib primitives, deciding
  between `defaultdict`/`Counter`/`deque`/`ChainMap`, tuning `lru_cache` or
  `cached_property`, designing `super()`-friendly hierarchies, or replacing
  comparison-heavy code with key-based sorts. Trigger keywords: pythonic,
  idiomatic Python, Hettinger, itertools, collections, functools, sort key,
  groupby, tee, defaultdict, OrderedDict, ChainMap, lru_cache,
  cached_property, total_ordering, super, MRO.
---

# Hettinger-Style Idiomatic Python⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​‌​‌​​‍​‌‌​​​‌​‍‌‌​​​​​​‍‌​​​​​‌​‍​​​​‌​‌​‍‌​​‌​​​​⁠‍⁠

## Load Boundaries

- Before transforming real code, READ `references/philosophy.md` for before/after rewrites and pipeline shapes.
- Before running a repo-wide cleanup, READ `scripts/pythonic_check.py` to see what it can and cannot catch; it is intentionally shallow.
- Do NOT load `references/philosophy.md` for cache, sort, container, or MRO design questions; this file is the decision layer.
- Do NOT trust `scripts/pythonic_check.py` as style authority; it will not detect iterator ownership bugs, cache leaks, or non-cooperative inheritance.

## Operating Mindset

Before you rewrite anything, ask:

- **Is the source one-shot?** Generators, file iterators, `map`, `zip`, `groupby`, and cursors are linear resources. Once you split or partially consume them, ownership becomes the real bug.
- **Am I compressing logic or compressing readability?** Hettinger code gets shorter by choosing better primitives, not by golfing. Prefer a named intermediate over a tuple key or nested comprehension that hides invariants.
- **Will laziness help or hurt?** Lazy pipelines shine when data is large and strictly single-pass. The moment you need peeking, replay, cross-branch fanout, or rich error reporting, materialize on purpose.
- **Is the bottleneck dispatch or algorithm shape?** `itemgetter`, `attrgetter`, `Counter`, and `deque` help, but the big wins usually come from deleting quadratic behavior or repeated Python-level comparisons.
- **Am I designing for composition?** When classes may be mixed later, signatures, root methods, and `super()` discipline matter more than today's inheritance tree.

## Iterator and Pipeline Heuristics

- Treat `tee()` as a last resort, not a harmless splitter. It buffers the lag between consumers, can raise `RuntimeError` under simultaneous use, and the docs explicitly say to use `list()` instead when one branch will run far ahead.
- `groupby()` groups runs, not values. It is right only when adjacency is meaningful or when you sort first with the same key function. For arbitrary arrival order, use `defaultdict(list)` or `Counter`.
- `takewhile()` consumes the first failing item. If that boundary record matters later, `takewhile()` is the wrong primitive unless you wrap the stream.
- Fully consuming `islice(it, start, stop)` advances the shared iterator by `max(start, stop)`. This matters when slicing a live cursor or generator other code still expects to read.
- `zip_longest()` against a possibly infinite iterable must be fenced with `islice()` or `takewhile()`, otherwise the finite side stops protecting you.
- Prefer `iter(callable, sentinel)` for block readers and polling loops. It expresses "keep calling until EOF or stop token" without `while True` bookkeeping.

## Containers: Choose By Failure Mode

- `defaultdict` is for mutating reads. `__missing__` only fires on `d[key]`; `d.get(key)` still returns `None`. If you need a non-mutating probe, `defaultdict` is the wrong signal.
- `setdefault()` is seductive because it looks single-step, but `setdefault(k, [])` builds the default object before Python knows whether the key exists. For hot grouping code, that means repeated throwaway allocations. Prefer `defaultdict(list)` unless you truly need plain-`dict` semantics.
- `Counter` arithmetic has different cleanup rules: `a - b` drops zero and negative counts, `subtract()` preserves them, and unary `+c` strips non-positives. Use the operator when you want inventory semantics; use `subtract()` only when debt is meaningful.
- `Counter` math preserves encounter order: first from the left operand, then new keys from the right. That matters when tests or UIs depend on deterministic tie order after arithmetic.
- `deque` is for ends, not middles. Appends and pops on either end are O(1); lookups toward the middle degrade, and `extendleft()` reverses the incoming order.
- `ChainMap` is for layered lookup, not shared mutation. Writes only hit `maps[0]`. If an update must land in an existing inner mapping, use an explicit merge or the docs' `DeepChainMap` pattern.
- `OrderedDict` still earns its keep when you need cheap reordering, `move_to_end(last=False)`, FIFO `popitem(last=False)`, or order-sensitive equality. Do not keep it out of habit now that plain `dict` preserves insertion order.
- Do not compare record types with `sys.getsizeof()` alone. A regular dataclass often looks smaller than a `namedtuple` until you count the per-instance `__dict__`. If memory matters, measure the object and its attached storage, or use slots.

## Sorting and Key-Function Rules

- Favor key functions over comparison functions. `cmp_to_key()` is mainly a migration bridge and locale escape hatch; comparison logic reintroduces Python-level work on every pairwise comparison.
- Use stable multi-pass sorts when directions differ or when a single tuple key would bury business rules. Timsort exploits existing order, so "minor key first, major key second" is often clearer and still fast.
- `itemgetter` and `attrgetter` are primarily clarity tools on modern CPython. They may be a bit faster, but if you need a benchmark to justify them, your bigger problem is elsewhere.
- For locale-aware sorts, use `locale.strxfrm()` as the key when possible; reach for `strcoll` plus `cmp_to_key` only when you truly need comparator semantics.
- Sorting calls the key once per element. Spend effort on making the key canonical and cheap, not on micro-optimizing rich comparison fallbacks.

## Caching and Memoization Rules

- Start with `lru_cache(maxsize=128)` only because that is the stdlib default, not because 128 is magic. Check `cache_info()` under realistic traffic; if misses stay high, the cache is probably noise.
- `lru_cache` is thread-safe for the cache structure, not for exactly-once execution. Concurrent misses may compute the same value twice before the first result lands.
- Keyword order can fragment the cache: `f(a=1, b=2)` and `f(b=2, a=1)` may occupy separate entries. Canonicalize calling conventions when cache density matters.
- `typed=True` only separates immediate argument types. It will distinguish `Decimal(42)` from `Fraction(42)` as direct args, but not the same values nested inside equal tuples.
- `lru_cache` keeps references to arguments and return values until eviction or `cache_clear()`. On methods, that includes `self`, so cache growth can accidentally pin whole object graphs.
- `cached_property` is right for per-instance values on normal objects, but it writes into each instance `__dict__`, interferes with PEP 412 key-sharing savings, and since Python 3.12 the getter may run more than once under races. If the class uses `__slots__` or memory density matters, stack `property()` over `lru_cache()` or memoize explicitly.
- Never cache generators, coroutines, open files, or mutable containers you expect callers to mutate. You are caching identity, not just computation.

## Cooperative Inheritance Rules

- Every override in a cooperative chain must call `super()`, even if today's parent is `object`. Future mixins change the MRO; hardwiring a parent name freezes composition.
- When the callee is not guaranteed to exist on `object`, provide a root class that terminates the chain and assert that no later class also implements the method. This turns a silent MRO bug into an immediate failure.
- Flexible `**kwargs` signatures are not style fluff here; they are how unrelated mixins strip what they need and forward the rest.
- If you adapt a non-cooperative third-party class, write an adapter that owns the impedance mismatch. Do not contaminate the cooperative chain with one-off positional signatures.
- When subclassing builtins, audit the whole method family. Overriding `__setitem__` on `dict` does not automatically affect `update()`, `setdefault()`, or constructor behavior.

## NEVER Do These

- NEVER use `tee()` because "I might need two passes." That is seductive because it preserves laziness, but the hidden buffer grows with branch skew and simultaneous use can explode at runtime. Instead snapshot with `list()` or redesign as a single-pass pipeline.
- NEVER use `defaultdict` and then read it with `.get()` expecting auto-vivification, because `default_factory` is ignored outside `__getitem__`. Instead use `d[key]` when creation is intended, or use a plain `dict` for non-mutating probes.
- NEVER reach for `setdefault(k, [])` in hot grouping code because it looks concise. It eagerly constructs throwaway defaults on hits. Instead use `defaultdict(list)` and make mutation explicit.
- NEVER use `cmp_to_key()` for ordinary business sorting because comparison functions feel "general." The concrete consequence is Python-level work on every comparison and harder-to-read ranking rules. Instead compute a canonical key or use stable multi-pass sorts.
- NEVER treat `lru_cache` as a lock or dedup barrier because the cache is coherent, not single-flight. The concrete consequence is duplicate expensive work under concurrent misses. Instead add your own synchronization if duplicate execution is harmful.
- NEVER put `lru_cache` on instance methods with many live objects because it is easy and benchmarks well in isolation. The consequence is that `self` stays strongly referenced until eviction, which can look like a memory leak. Instead use `cached_property`, a module-level helper, or explicit instance memo fields.
- NEVER use `cached_property` on space-sensitive fleets of objects because it silently breaks key-sharing dict savings and cannot work on slot-only classes without `__dict__`. Instead use `property` over `lru_cache()` or a dedicated slot-backed cache field.
- NEVER override one builtin method and assume the type is now instrumented, because sibling methods may bypass your hook. Instead prefer composition or `UserDict`, or audit and override the whole mutating surface.
- NEVER use `total_ordering` in a comparison hotspot because it is convenient. The consequence is extra Python call frames and slower rich comparison dispatch. Instead hand-write the full ordering set when profiling says comparisons matter.
- NEVER use `namedtuple(rename=True)` on external schemas because it feels forgiving. The consequence is silent field renames like `_1` and `_3`, which hide upstream data-contract breaks. Instead validate field names and fail fast.

## When Stuck

- If the refactor makes iterator ownership ambiguous, materialize once and document the boundary.
- If cache tuning is guesswork, instrument with `cache_info()` before changing `maxsize`.
- If sorting rules read like a mini-language, split them into stable passes or named helper keys.
- If the MRO story takes longer than a comment to explain, prefer composition.
