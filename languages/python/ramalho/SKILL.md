---
name: ramalho-fluent-python
description: "Design Python objects that cooperate with the interpreter through protocols, descriptors, dataclasses, weak references, and zero-copy buffers. Use when writing or refactoring custom collections, value objects, operator overloads, `Protocol` types, `__slots__`, descriptors, `memoryview` or `struct` code, or advanced dataclass APIs. Trigger on: dunder methods, `NotImplemented`, `Protocol`, `runtime_checkable`, `__slots__`, `descriptor`, `weakref`, `cached_property`, `dataclass`, `memoryview`, `struct`."
tags: protocols, dataclasses, descriptors, slots, weakref, memoryview, typing, data-model
---

# Ramalho: Cooperate With the Interpreter⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​‌​‌​‌‍‌​​​‌​​‌‍‌​‌​‌‌‌​‍​​‌​​‌​‌‍​​​​‌​​‌‍‌‌​​‌​​‌⁠‍⁠

Ramalho-style Python starts from this question: what protocol is this object promising to the interpreter, not what methods can I bolt onto a class. The win is getting built-ins, operators, pattern matching, and type checkers to do the work once the object advertises the right hooks.

This skill is intentionally self-contained. Do NOT load generic Python style guides, beginner OOP notes, or cookbook-level dunder tutorials for this task; they dilute the data-model decisions that matter here.

## Route First

Pick one branch and ignore the rest:

- `__slots__`, dataclass layout, `cached_property`, or weakrefs: read the slot and dataclass rules below; skip Binary and Buffer.
- `Protocol`, `typing`, `runtime_checkable`, or operator overloading: read the typing and `NotImplemented` rules; skip descriptor internals unless attribute access is involved.
- Properties, descriptors, `__getattr__`, `__getattribute__`, or alternate constructors: read the descriptor branch; skip binary parsing.
- Raw bytes, buffers, C structs, or wire formats: read Binary and Buffer first; skip collection mixin advice unless parsed objects escape that layer.

## Before You Touch a Class

Before adding or editing a dunder, ask yourself:

- Is this object a value, a mutable service, or a facade over external state? Value objects can justify comparison and hashing; services usually should not.
- Which protocol is actually true: `Sequence`, `Mapping`, `Iterable`, descriptor, numeric type, context manager, or binary buffer? Only implement hooks that match the real semantics.
- Will users depend on positional construction or pattern matching? `dataclass(kw_only=True)` is often the cheapest way to keep an API evolvable because keyword-only fields are excluded from `__match_args__`.
- Is the code in a hot path, at a memory wall, or at an I/O boundary? Be strict with `__slots__`, `runtime_checkable`, and zero-copy buffers only when the economics justify the trade-off.
- Does the task require runtime validation, or only static promises? `Protocol` is primarily for type checkers; runtime checks are a separate design choice.

## Choose The Smallest Honest Surface

| If the object is... | Prefer | Because |
| --- | --- | --- |
| Random-access, repeatable, finite | `collections.abc.Sequence` semantics | The mixins assume repeated `__getitem__` access and derive `__contains__`, `__iter__`, `index`, and `count` from it. |
| Lazy, one-pass, or external-state-backed | Explicit `__iter__`, often no `__len__` | Letting sequence fallbacks fake iteration gives the wrong complexity and the wrong user expectations. |
| Logically immutable public data | `@dataclass(frozen=True, kw_only=True)` or named tuple | You get value semantics without hand-written comparison bugs, and `kw_only` keeps constructor and match APIs from freezing accidentally. |
| Attribute-bound validation or computed access | Descriptor or property | The invariant belongs at attribute access time, not buried in ad hoc setter calls. |
| Many homogeneous tiny instances | `__slots__` or `dataclass(slots=True)` after measurement | CPython key-sharing dicts already reclaim 10-20% memory for OO programs, so slots are no longer a default first move. |
| Runtime structural checks at a boundary | ABC or targeted capability probe | `runtime_checkable` checks presence, not signatures, and can be surprisingly slow. |
| Binary input you do not control | Existing library first; otherwise `struct.iter_unpack` plus `memoryview` | Raw binary formats are brittle, padding-heavy, and endianness-sensitive. |

## High-Signal Heuristics

- Return `NotImplemented` from rich comparisons and numeric ops when the other operand is not truly supported. Returning `False` feels tidy, but it blocks reflected dispatch and creates asymmetric cross-type behavior. Since Python 3.14, `bool(NotImplemented)` raises `TypeError`, so never use it as a truthy sentinel.
- Treat `__contains__`, `__reversed__`, and `__bool__` as performance hooks, not decorative completeness. If you omit them, Python falls back to iteration or `__len__`; that is only acceptable when the fallback has the same semantics and cost profile.
- Prefer ABC mixins when the derived complexity is acceptable, and implement the hot hooks yourself when it is not. `Sequence` mixins repeatedly call `__getitem__`; that is excellent for true sequences and terrible for remote, paged, or side-effectful data.
- Use descriptors when the invariant belongs to the attribute, not the call site. If the attribute must be truly read-only, define `__set__` that raises `AttributeError`; that makes it a data descriptor so instance `__dict__` entries cannot shadow it.
- If you override `__getattribute__`, assume you are opting out of Python's automatic descriptor machinery until proven otherwise. Most meta-access problems are better solved with `__getattr__`, descriptors, or `__set_name__`.
- Use weak references for registries, caches, and side tables owned by long-lived classes. A plain `set` or `dict` is seductive because it is simple, but it quietly turns "remember instances" into "keep instances alive forever."
- Reach for `memoryview` before copying bytes. In Ramalho-style code, bytes are copied at trust boundaries or display boundaries; slices of a `memoryview` stay zero-copy.

## NEVER Do This

- NEVER make a mutable object hashable because the object "usually doesn't change." The seduction is set and dict compatibility; the consequence is silent bucket corruption when fields drift. Instead make the value immutable, or set `__hash__ = None`.
- NEVER use `unsafe_hash=True` on a dataclass as a convenience toggle. It exists for specialized logically immutable cases; on ordinary mutable objects it gives you hash semantics that lie. Instead pair hashing with `frozen=True` or write the contract explicitly.
- NEVER add `__slots__` as a default optimization because old Python advice said it is always leaner and faster. The consequence is broken `cached_property`, lost dynamic attributes, disabled weakrefs unless you add `__weakref__`, awkward inheritance, and zero benefit when you subclass an unslotted base. Instead measure first and slot only memory-critical, high-cardinality types.
- NEVER assume `dataclass(slots=True)` is drop-in. The decorator returns a new class, warns about `__init_subclass__` parameters, no-arg `super()` can bite, and `__slots__` is not a reliable source of field names. Instead use `dataclasses.fields()` for introspection and audit inheritance before opting in.
- NEVER use `@runtime_checkable Protocol` as a validator because it only checks attribute presence, not signatures or types. It is seductive because it reads like an interface check; the consequence is false positives and slow `isinstance` checks in hot paths. Instead keep protocols for static typing, or write explicit runtime capability checks.
- NEVER implement only `__getitem__` for a non-sequence because iteration and membership will "work anyway." The consequence is hidden old-style sequence fallback until `IndexError`, surprising repeated lookups, and impossible one-shot semantics. Instead implement `__iter__` for streams and `__contains__` for fast membership when needed.
- NEVER override `__getattribute__` to power lightweight computed attributes because it centralizes too much and bypasses normal descriptor behavior. The consequence is recursion bugs, broken properties, and invisible metaprogramming. Instead use descriptors, properties, or `__getattr__` for misses only.
- NEVER persist or compare set or dict order across processes via hashes. Hash randomization exists to defend against worst-case `O(n^2)` collision attacks, so order can drift across runs and builds. Instead sort explicitly when order matters.
- NEVER hand-roll `__class_getitem__` for cute runtime APIs. Custom uses outside typing are discouraged and third-party type checkers may not understand them. Instead inherit from `typing.Generic` or a stdlib generic base.

## Numbers And Thresholds That Matter

- PEP 412 key-sharing dictionaries made instance `__dict__` storage split-table on CPython; the PEP reported object-oriented program memory reductions around 10-20%, with shared-key dicts typically about half the old size. That is why `__slots__` is now a later optimization, not a reflex.
- The descriptor guide's concrete example measured a two-attribute instance at 48 bytes with `__slots__` versus 152 bytes without on 64-bit Linux, and about 35% faster attribute reads in one benchmark. Treat those as "large-instance-count only" numbers, not blanket justification.
- `runtime_checkable` is opt-in and documented as surprisingly slow. Use it at plugin or API boundaries, not inside per-item loops.
- In CPython, `__len__` must fit within `sys.maxsize`; if a conceptual size can exceed that, define `__bool__` so truth testing does not accidentally trip `OverflowError`.

## Binary And Buffer Branch

Before touching binary formats, ask:

- Is there a maintained parser already? Use that first.
- Is the format external or cross-language? Prefer JSON, MessagePack, Protocol Buffers, or another explicit schema over home-grown `struct` layouts.
- Are you forced to parse raw records? Then specify endianness in every format string, use `struct.iter_unpack` for streams of homogeneous records, and keep payloads as `memoryview` until the API boundary.
- Are fields fixed-width C strings? Expect null termination plus garbage or padding after the first `\0`; clean that explicitly after unpacking.

## Slot And Descriptor Edge Cases

- If a class with `__slots__` needs weak references, add `'__weakref__'` or use `dataclass(slots=True, weakref_slot=True)`.
- If a subclass inherits from a base without `__slots__`, the subclass still has an instance `__dict__`; slotting it for memory is mostly theater.
- If multiple parents define non-empty slot layouts, expect `TypeError`; only one slotted parent may contribute actual storage.
- If a slot name is repeated from a base class, the base slot becomes inaccessible through normal lookup; treat that layout as broken.
- If a descriptor must not be shadowed by instance state, make it a data descriptor. Non-data descriptors intentionally yield to instance dictionaries.

## Fallback Strategy

- If slots break a needed tool such as `cached_property`, restore `__dict__` or stop slotting that type.
- If a descriptor becomes hard to reason about, write the precedence chain down: data descriptor -> instance dict -> non-data descriptor -> class var -> `__getattr__`.
- If typing design and runtime design diverge, privilege runtime truth first, then express it to the type checker with `Protocol`, ABCs, or helper functions.
- If a custom collection gets weird, reduce it to the smallest honest protocol. Most bugs come from claiming `Sequence` or hashability too early.

## Done Right

A Ramalho-style refactor usually deletes code. The goal is not cleverness; it is getting the interpreter, built-ins, and typing tools to cooperate because the object advertises the right protocol and nothing more.
