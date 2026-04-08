---
name: vanrossum-pythonic-style
description: Write idiomatic Python the way Guido van Rossum designed it, applying BDFL-era reasoning about trade-offs most guides get wrong. Use when writing, reviewing, or refactoring Python code that must be genuinely Pythonic (not just "works" — reads the way a core developer would write it), when deciding between dataclass/NamedTuple/TypedDict/pydantic, when evaluating whether __slots__ is still worth it on modern CPython, when debating walrus operator style, when a reviewer claims something "isn't Pythonic," or when you need the actual reasoning behind rejected proposals like tail-call elimination and multi-line lambdas. Triggers — "Pythonic", "idiomatic Python", "Zen of Python", "PEP 8", "PEP 20", "van Rossum", "Guido", "BDFL", "refactor Python", "Python style", "dataclass vs NamedTuple", "mutable default argument", "walrus operator", "PEP 572", "tail recursion Python", "should I use slots", "EAFP vs LBYL", "lru_cache", "open encoding", "dict ordering".
tags: python, pythonic, van-rossum, bdfl, zen, pep8, pep20, style, idioms, dataclass, slots, walrus, eafp
---

# Guido van Rossum — Pythonic Style

Guido's design rules were NOT the Zen of Python (that was Tim Peters codifying retroactively). The actual rules from Guido's own history notes were about **pragmatism under time pressure**:

> "Don't fret about performance — optimize later. Don't try for perfection; 'good enough' is often just that. It's okay to cut corners sometimes, especially if you can do it right later. Don't fight the environment and go with the flow."

> "A bug in the user's Python code should not cause undefined interpreter behavior; a core dump is never the user's fault."

Every controversial Python decision (explicit `self`, no TCO, no multi-line lambda, significant whitespace, GIL) flows from one or both of these: **pragmatism over purity**, and **user safety over user cleverness**. Read the second quote again — it is the reason Python rejects features that would let user code crash the interpreter. When purity and pragmatism conflict, pragmatism wins.

## The Core Heuristic — four questions before writing any Python line

1. **"Can this line be mistaken for something else?"** If yes, be explicit. `if value is None` not `if not value` whenever `0`, `""`, `[]` are valid values. `raise ValueError("bad port: %r" % port) from exc` not bare `raise`. Implicitness is a bug multiplier.
2. **"What does the traceback look like when this fails?"** Guido rejected tail-call elimination specifically because it destroys tracebacks. Your job is to write code that crashes at the frame containing the bug, not three frames away. If a helper hides the real call site, inline it or use `raise ... from`.
3. **"Is this the ONE obvious way, or am I inventing a second way?"** Python explicitly rejects Perl's TMTOWTDI. If a reviewer can write a different-looking version with the same semantics, at least one of you is wrong.
4. **"Am I fighting the language?"** Metaclasses to auto-register, decorators to bolt on tail calls, AST rewrites to hide `self` — if you're reaching for these, you are writing a dialect of Python that every future maintainer will hate. The Pythonic answer is usually to accept what the language gives you.

## Expert Trade-offs (the things textbooks get wrong)

**`__slots__` barely matters on Python 3.11+.** The memory savings collapsed from ~80–216 bytes/instance to ~40–64 bytes, and the attribute-lookup speedup dropped from 62% to 3–13% (essentially 0% on optimized builds). Most `__slots__` advice is from 3.9-era benchmarks. Add it only when creating >100k instances in a hot loop, or when you want to *forbid* attribute creation as a correctness constraint. See `references/expert-traps.md` §1 for the version table.

**`@dataclass(frozen=True, slots=True)` is the modern default container** (Python 3.10+). It's smaller AND faster than `NamedTuple` at attribute access (48 B vs 56 B, ~23 ns vs ~29 ns on 2-field structs). Use `NamedTuple` only when you need tuple unpacking at call sites, positional compatibility with tuple-expecting APIs (TensorFlow `tf.function`), or Python <3.10 support. Use `pydantic` only for validated data at system boundaries — it pays ~10x creation cost for that validation, so don't use it for internal state.

**`NamedTuple` equality is a nominal-vs-structural footgun.** `Point(1,2) == Color(1,2)` is `True` because both are tuples. If your type means "a Point," not "a pair of ints," use a frozen dataclass.

**`s += x` in a loop is O(n) on CPython and O(n²) everywhere else.** CPython since 2.4 has an in-place optimization when the string's refcount is 1. Your code works, your CI passes, you deploy on PyPy, and prod melts. **Always use `"".join(parts)`**. The optimization is a CPython implementation detail, not a language guarantee.

**Dict insertion order became a guarantee by accident.** 3.6 implementation detail (side effect of a Raymond Hettinger memory optimization), 3.7 language feature. If you maintain pre-3.7 code that relied on order, it was technically undefined. For new code, the guarantee is real — but still reach for `collections.OrderedDict` when order is *semantically* meaningful (it has `move_to_end()` and order-sensitive equality; plain dicts do not).

**`open()` without `encoding=` is a cross-platform bug.** Default is platform locale: UTF-8 on macOS/Linux, cp1252/cp936 on Windows. PEP 597 exists for this reason. **Always** pass `encoding="utf-8"` explicitly, and enable `PYTHONWARNDEFAULTENCODING=1` in CI.

**`functools.lru_cache(maxsize=None)` is a memory leak by design.** Use `functools.cache` (3.9+) when you *mean* unbounded — the name documents intent. `lru_cache` on an instance method holds a reference to every `self` it's ever seen; use `@cached_property` or make the method a module-level function instead.

**`sys.setrecursionlimit(10000)` risks a segfault.** The Python-level limit defaults to 1000, but the real constraint is the C stack (~8 KB frames on CPython). Raising it past ~10k means "a core dump is now your fault" — exactly the rule Guido refused to break when he rejected TCO. If you need deep recursion, you need iteration.

**EAFP beats LBYL for correctness, not just style.** `if os.path.exists(p): open(p)` has a TOCTOU race: any other process can delete the file between check and open. `try: open(p) except FileNotFoundError:` is atomic. LBYL is acceptable only when the check is O(1) and atomic with the use (e.g., `if key in local_dict`), and even then it breaks under free-threading.

## NEVER (with non-obvious consequences)

- **NEVER use mutable default arguments** (`def f(x=[])`). The default is evaluated at `def` time and stored on `function.__defaults__` — every call that omits `x` mutates the same list. This is not a bug; it's consistent with how class bodies and decorators also evaluate at `def` time. Seductive because it reads like C++ default parameters; the consequence is silent state leakage between unrelated callers. Fix: `def f(x=None): if x is None: x = []`.

- **NEVER use bare `except:`**. It catches `KeyboardInterrupt` and `SystemExit`, making Ctrl-C impossible and `sys.exit()` silent. `except Exception:` is what you meant. Consequence of bare `except:`: unkillable processes and silent test-runner failures.

- **NEVER nest list comprehensions more than two levels deep.** This is a **correctness** rule, not style. The variable-binding order in `[x for row in matrix for x in row if x > t]` is left-to-right, opposite of the "prose reading" intuition. A three-level comprehension is ambiguous to every reader including future-you. Fix: extract the inner loops to a generator function with a descriptive name.

- **NEVER use the walrus operator outside PEP 572's two canonical patterns** — (1) `while chunk := stream.read(8192):` and (2) expensive-predicate filters `[y for x in data if (y := f(x)) is not None]`. Anything else is the style that cost Guido his BDFL title. If `:=` "saves a line," you are saving the wrong resource.

- **NEVER use `typing.cast` to silence a type error.** Unlike TypeScript `as`, `cast` does nothing at runtime — it lies to the type checker AND to humans. If you have to cast, your types are wrong. Fix: `isinstance` check (the checker narrows automatically), or redesign.

- **NEVER subclass `str`, `int`, `tuple`, or `bytes` casually.** Methods that return new instances (`upper()`, `__add__`) return the *base* class, not your subclass, silently. `StrEnum` members compare `==` to raw strings — almost always a footgun in API boundaries.

- **NEVER use `is` for anything except `None`, `True`, `False`.** Small-int caching (CPython caches -5..256) and string interning make `x is 100` work sometimes, fail sometimes. These are CPython implementation details. Modern Python raises `SyntaxWarning` on literal `is`; treat it as an error.

- **NEVER catch exceptions with `except: pass` without a logged reason.** "Errors should never pass silently" is one of Guido's explicit original rules, not stylistic advice. Minimum acceptable: `except SpecificError as e: log.debug("ignoring: %s", e)`.

- **NEVER write multi-line lambdas via tuple-of-statements, `exec`, or `and`/`or` chains.** Guido rejected multi-line lambdas because any syntax introduces a "where does the colon go" paradox. If you need multiple statements, you need a name; if you have a name, you can also have a docstring. Fix: `def`.

## Procedure — "Is this Pythonic?" review

Walk this list in order, stopping at the first failure:

1. **Would a stack trace from the failing line blame the actual bug?** If a helper wraps the real failure, inline it or add `raise ... from`.
2. **Does reading this require knowing `def`-time vs call-time evaluation, descriptor protocol, MRO, or GIL semantics?** If yes, add a comment pointing at the relevant mental model, or refactor until it doesn't.
3. **Is there a stdlib tool that would replace this?** `collections.Counter`, `itertools.groupby`/`pairwise`/`batched`, `functools.cache`/`reduce`, `pathlib.Path`, `dataclasses`, `contextlib.contextmanager`/`suppress`, `enum`, `typing.Protocol` for structural subtyping.
4. **Can the happy path be read without reading the error handling?** `try:` blocks should wrap ONE operation that can fail, followed by `else:` for the continuation. Narrow `try:`, never wide. See `references/expert-traps.md` §13.
5. **Is "the one obvious way" actually obvious?** If there are two equally plausible implementations, the less clever one wins.

## Loading rules for references

- **Before recommending a container type (dataclass / NamedTuple / TypedDict / pydantic), READ `references/expert-traps.md` §2** for the empirical size/speed numbers on 3.12.
- **Before justifying (or rejecting) `__slots__`, READ `references/expert-traps.md` §1** for the version-specific memory savings table.
- **Before arguing about tail recursion, multi-line lambdas, or "why doesn't Python have X", READ `references/expert-traps.md` §3 and §14** for Guido's own reasoning.
- **Do NOT load `references/expert-traps.md`** for simple tasks: renaming a variable to `snake_case`, adding type hints to an obvious signature, converting `%` formatting to f-strings, replacing `range(len(x))` with `enumerate(x)`. Claude already knows these; loading the reference wastes context.
