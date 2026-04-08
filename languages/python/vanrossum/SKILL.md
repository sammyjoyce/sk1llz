---
name: vanrossum-pythonic-style
description: "Design and review Python the way Guido-style library maintainers do: preserve debuggability, keep APIs evolvable, and avoid CPython-only traps. Use when deciding public signatures, dataclass layout, slots, caching, encoding defaults, inheritance boundaries, immutable subclasses, or disputed \"Pythonic\" trade-offs. Triggers: pythonic, Guido, van Rossum, positional-only, keyword-only, dataclass, slots, weakref_slot, __match_args__, cached_property, lru_cache, unsafe_hash, EncodingWarning, __new__, walrus, tail recursion."
tags: python, pythonic, van-rossum, api-design, dataclass, slots, caching, encoding, inheritance
---

# Guido van Rossum Style: Semantics First

Guido-style Python is not "shorter code" or "more magic". It is code that keeps tracebacks useful, leaves room for API evolution, and refuses optimizations that quietly change semantics.

## Use this skill for these disputes

- Public API and signature design.
- Dataclass vs handwritten class vs immutable value object.
- `slots`, `cached_property`, `lru_cache`, weakrefs, and hashability.
- Immutable builtins and value normalization in `__new__`.
- Encoding defaults, cross-platform text I/O, and library portability.
- Inheritance boundaries, subclass APIs, and "is this actually Pythonic?"

## Load only what matches

- Before changing signature shape, read PEP 570 sections on subclass consistency and PEP 399 parity.
- Before changing dataclass layout, read the stdlib `dataclasses` docs for `kw_only`, `__match_args__`, `slots`, `weakref_slot`, and `unsafe_hash`.
- Before adding caching, read the stdlib `functools` docs and the FAQ entry "How do I cache method calls?"
- Before changing file I/O defaults, read PEP 597.
- Before proposing recursion-as-loop tricks, read Guido's "Tail Recursion Elimination" note.
- Do NOT load those sources for format-only edits, import sorting, trivial renames, or pure annotation cleanup.

## Before doing X, ask yourself

1. Before renaming or exposing a parameter name, ask: is the name part of the public contract, or merely today's implementation detail?
2. Before making a constructor "clearer" with `kw_only=True`, ask: am I also changing pattern-matching surface by shrinking `__match_args__`?
3. Before adding `slots=True`, ask: how many live instances exist, and do I need dynamic attributes, weakrefs, `cached_property`, or base-class `__init_subclass__` parameters?
4. Before adding a cache, ask: what lifetime gets pinned, what invalidates the result, and what happens if concurrent callers compute it twice?
5. Before hiding work behind attribute syntax, ask: will callers still be right to assume the access is cheap and side-effect-light?
6. Before using clever recursion or walrus compression, ask: am I improving scanability, or only compressing syntax?

## High-signal procedures

### Signature procedure

- Use positional-only parameters (`/`) when the parameter name should not become public API, when subclasses may need a different name, or when a pure-Python surface must match a C/builtin surface. PEP 570 treats `dict.update`-style ambiguity and PEP 399 parity as first-class reasons, not edge cases.
- Use keyword-only parameters when the name is semantically meaningful and you want call sites to stay self-documenting.
- For public signature changes, check four things in order: existing keyword call sites, subclass overrides with renamed parameters, pure-Python/C parity, and dataclass pattern matches.

### Data-model procedure

- Treat `kw_only=True` as more than a constructor nicety. Dataclass keyword-only fields are excluded from `__match_args__`, so `case MyType(...)` can break even when regular construction looks better.
- Treat `slots=True` as a semantic change, not a style tweak. Dataclasses return a new class object when `slots=True`; code that records class identity early can observe that.
- If `eq=True` and `frozen=False`, dataclasses intentionally make the class unhashable. That is the runtime telling you mutation and hashed containers do not mix.
- Use `field(hash=False, compare=True)` only when equality needs a field but hashing it is too expensive. That is the narrow escape hatch. `unsafe_hash=True` is not the normal answer.
- If subclassing an immutable builtin such as `str`, `int`, `tuple`, or `date`, enforce invariants in `__new__`, not `__init__`.

### Cache procedure

- Use `cached_property` only for instance-local, idempotent values on objects that are effectively immutable. In Python 3.12+, the undocumented once-only lock was removed, so concurrent access may run the getter more than once.
- Use `lru_cache` only when the result is reusable, arguments are hashable, and recent calls predict future calls. Remember the non-obvious parts: keyword order may create distinct cache entries, `typed=True` only distinguishes immediate arguments, and cached methods keep `self` alive until eviction.
- If you cannot describe the invalidation rule in one sentence, do not cache yet.

### Text-I/O procedure

- For new APIs, prefer `encoding="utf-8"` unless locale coupling is the contract.
- If locale behavior is intentional, say `encoding="locale"` explicitly instead of relying on omission.
- In CI, turn on `-X warn_default_encoding` or `PYTHONWARNDEFAULTENCODING=1` so implicit-locale bugs show up before users on other platforms hit them.

### Inheritance procedure

- Decide explicitly whether an attribute is public API, subclass API, or internal. PEP 8's rule matters here: if in doubt, start non-public, because Python makes it easy to expose later and painful to retract later.
- Use double-leading-underscore names only to avoid subclass name collisions in extensible bases. It is collision control, not real privacy.
- Expose simple public data as a plain attribute first. `property` is the compatibility escape hatch when behavior must grow later, not an excuse to hide expensive work behind dot syntax.

## Numbers and thresholds that actually matter

- Guido's tail-recursion note assumes a typical recursion limit around `1000`: enough for ordinary tree traversal, not enough for "recursion as loop over a large list".
- `lru_cache` defaults to `maxsize=128`; that is a convenience default, not evidence that `128` fits your workload. `maxsize=None` is explicitly unbounded.
- The official descriptor HOWTO's sample measurement shows why `__slots__` is a scale tool, not a style badge: a two-attribute instance used `48` bytes with `__slots__` and `152` without, with attribute reads about `35%` faster in that measurement. If you do not expect large live populations, the complexity usually does not earn its keep.

## NEVER do these

- NEVER add `/` just because builtin-style signatures look advanced, because the seductive part is the tiny syntax change. The concrete consequence is that you erase useful keyword readability without gaining evolution headroom. Instead use `/` only when the name must stay non-contractual or subclass/C-parity pressure is real.
- NEVER flip `kw_only=True` on a dataclass because constructor calls "read nicer". The seductive part is cleaner call sites; the concrete consequence is that keyword-only fields disappear from `__match_args__`, so positional class-pattern matches can stop working. Instead search both constructor call sites and `case TypeName(...)` matches first.
- NEVER treat `slots=True` as a free memory win, because the seductive part is benchmark folklore. The concrete consequence is broken weakrefs, no instance `__dict__`, `cached_property` failures, `__init_subclass__` surprises, and a new class object in dataclasses. Instead require an instance-count argument and a compatibility checklist before slotting.
- NEVER use `unsafe_hash=True` to quiet "unhashable type" errors, because the seductive part is that sets and dict keys start working immediately. The concrete consequence is mutated keys that drift out of hashed collections or violate equality assumptions. Instead freeze the real identity or hash only stable fields.
- NEVER put `lru_cache` on an instance method whose result depends on mutable instance state, because the seductive part is a one-line speedup. The concrete consequence is stale answers plus object retention through cached `self` references; keyword reordering can also duplicate entries. Instead externalize the pure function or use `cached_property` with an invalidation story.
- NEVER assume `cached_property` is a harmless drop-in, because the seductive part is attribute syntax with memoization. The concrete consequence is extra per-instance dict space, failure on slotted classes without `__dict__`, and multi-threaded double computation on 3.12+. Instead choose it only for effectively immutable instances or use `property()` over an explicit cache function.
- NEVER omit `encoding` in library code because your machine uses UTF-8. The seductive part is that omission works locally. The concrete consequence is Windows and locale-specific breakage plus `EncodingWarning` once projects enable it. Instead spell `utf-8` or `locale` deliberately.
- NEVER hide I/O, network calls, or O(n) work behind a plain `property`, because the seductive part is a tidy attribute API. The concrete consequence is callers accidentally multiplying expensive work because attribute syntax advertises cheap access. Instead use a method or a cached attribute with explicit invalidation semantics.
- NEVER use walrus in call arguments, defaults, or dense comprehensions just because it shortens the line. The seductive part is fewer temporary names; the concrete consequence is readers missing the binding site or stumbling over scope and precedence. Instead reserve `:=` for the two strong patterns: filtered binding (`if match := ...`) and loop-and-read (`while chunk := f.read(...)`).
- NEVER pitch tail-recursion elimination or recursion-as-loop decorators as a Pythonic optimization, because the seductive part is functional elegance. The concrete consequence is worse tracebacks, portability problems across implementations, and code that fails once depth grows past the typical recursion limit. Instead write the loop or iterator directly.

## Fallback strategies

- If you need slot-like memory savings but also need cached attributes, add `__dict__` intentionally or replace the design with explicit cached functions; do not discover the incompatibility in production.
- If you need constructor clarity but cannot afford pattern-matching breakage, keep only truly optional fields keyword-only and leave structural fields positional.
- If you need hashing but one field is mutable or expensive, keep equality on that field and exclude it from hashing rather than pretending the whole object is safely immutable.
- If recursion expresses the algorithm best but depth is unbounded, keep the recursive helper for the small case and expose an iterative driver for real workloads.
