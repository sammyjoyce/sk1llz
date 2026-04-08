---
name: maciver-hypothesis-testing
description: >-
  Expert patterns for Python's Hypothesis property-based testing library.
  Covers strategy performance traps, shrinking internals, stateful testing
  with Bundles, health check tuning, CI database management, and target()
  coverage guidance. Use when writing @given tests, debugging flaky
  Hypothesis failures, designing composite strategies, testing stateful
  APIs with RuleBasedStateMachine, or tuning Hypothesis settings/profiles.
  Keywords: hypothesis, property-based testing, @given, strategies,
  shrinking, stateful testing, st.composite, health checks, PBT, fuzzing.
---

# Hypothesis — Expert Practitioner Guide

## Thinking Framework

Before writing any Hypothesis test, ask:

1. **What property actually holds universally?** Most bugs come from asserting
   something that's only *usually* true. Round-trips, idempotency, invariant
   preservation, and "does not crash" are safe starting points.
2. **Can I generate valid inputs directly, or am I filtering?** If >5% of
   generated values get rejected, your strategy is wrong — restructure with
   `@st.composite` and constrained ranges, not `filter()`/`assume()`.
3. **What's my oracle?** The best PBT tests compare against a simpler reference
   implementation, not just assert structural properties.
4. **Will the shrunken example be meaningful?** If your test uses `.map()` with
   a lossy transform, the shrunken output may not help diagnosis.

## Strategy Performance — The Non-Obvious Traps

| Trap | Why It's Seductive | Consequence | Fix |
|------|-------------------|-------------|-----|
| `st.text()` inside `st.recursive()` without `max_size` | Seems natural | Generation time explodes; triggers `too_slow` health check | Always `st.text(max_size=N)` in recursive contexts |
| `.filter()` with >50% rejection | Looks clean | Silent performance death; 10× slower, worse shrinking | Restructure as `@st.composite` with constrained draws |
| `st.from_type()` on complex classes | Avoids writing strategies | Generates nonsensical instances; `__init__` side effects | Write explicit `@st.composite` or `st.builds()` with constrained args |
| `st.floats()` without `allow_nan=False` | Default seems fine | NaN breaks almost every comparison; test "passes" with wrong logic | Always decide: `allow_nan=False, allow_infinity=False` unless testing IEEE behavior |
| Unbounded `st.lists(st.lists(...))` | Nested collections seem harmless | Quadratic+ generation/shrinking time | Set `max_size` at every nesting level |

### `assume()` vs `.filter()` — The Real Difference

`assume()` discards the **entire test case** and starts over. `.filter()` on a
strategy discards only that one draw and retries internally (up to a limit).
This means `.filter()` on an element strategy inside `st.lists()` is
exponentially more efficient than `assume()` on the whole list — a list of 5
filtered booleans has ~60% success rate vs ~3% with `assume(all(...))`. **Use
`.filter()` at the narrowest possible scope, `assume()` only for cross-argument
constraints you can't express in strategies.**

### `@st.composite` vs `.flatmap()` — No Performance Difference

They are exactly equivalent — `@st.composite` is syntactic sugar over
`.flatmap()`. The Hypothesis numpy `arrays()` strategy uses `.flatmap()`
internally only to avoid redundant argument validation on recursion, not for
performance. Choose `@st.composite` for readability in all application code.

## Shrinking — What Experts Know

Hypothesis does NOT shrink by type (unlike QuickCheck). It shrinks the
underlying **byte stream** ("choice sequence"). This means:

- **`.map()` preserves shrinking quality** — the byte stream shrinks, then gets
  mapped. Use `.map()` freely.
- **`.filter()` degrades shrinking** — rejected candidates during shrinking
  force backtracking. Heavy filtering makes shrinking slow and produces
  suboptimal minimal examples.
- **`@st.composite` with many `draw()` calls** — each draw adds a segment to
  the byte stream. More draws = more shrinking dimensions = better results.
  Don't fear many small draws.
- **Shrinking targets human readability** — integers shrink toward 0, strings
  toward shorter/simpler, booleans toward False. Design strategies so the
  shrink direction aligns with "simpler test case."

## Stateful Testing — Where the Real Bugs Hide

NEVER skip stateful testing for anything with mutable state. `RuleBasedStateMachine`
finds bugs that unit tests structurally cannot: state-dependent interactions,
ordering bugs, resource leaks across operations.

**Critical gotchas:**

- **`@initialize` runs after `__init__` but before any `@rule`**. Use it to
  seed Bundles so rules that `consumes()` items don't starve on the first step.
- **`consumes(bundle)` removes the item** — this models deletion correctly but
  means subsequent rules can't reference it. Use plain `bundle` for read-only
  references, `consumes(bundle)` for destructive operations.
- **`@precondition` without `@rule`/`@invariant` silently does nothing** (since
  v6.136.2 this raises an error, but older versions just ignored it).
- **`@invariant()` runs after every single rule** — make it cheap. An expensive
  invariant multiplied by hundreds of steps kills performance.
- **Step count defaults to ~50 steps**. Increase `stateful_step_count` in
  settings if you need deeper sequences (but expect slower runs).

**MANDATORY — READ [`references/patterns.md`](references/patterns.md)** before
writing stateful tests. It contains the Bundle/initialize/consumes patterns.

## Health Checks & Settings — The Expert Tuning Guide

NEVER blanket-suppress health checks with `suppress_health_check=list(HealthCheck)`.
This hides real problems. Suppress individually and understand why:

| Health Check | When to Suppress | When It's a Real Problem |
|---|---|---|
| `too_slow` | Test does I/O or setup per example | Strategy generates data too slowly — restructure it |
| `filter_too_much` | Never suppress — fix your strategy | Filter rejection >50% means your strategy is wrong |
| `large_base_example` | Intentionally testing large data | Unintentional — constrain `max_size` |
| `differing_executors` | Multi-threaded test runners | Usually a real configuration bug |

### Deadline: The Most Misunderstood Setting

`deadline=200` (ms) is the default. It measures **per-example wall clock time**.
NEVER set `deadline=None` globally because it hides performance regressions.
Instead:
- CI with variable machine speed: `deadline=None` in CI profile only
- Tests with I/O: `deadline=timedelta(milliseconds=1000)` per-test
- Threaded tests: deadline is automatically disabled (since v6.136.3) because
  thread scheduling adds unpredictable latency

### `derandomize=True` — CI Trap

`derandomize=True` makes tests deterministic by seeding from the test source.
This means **changing the test source changes the seed**, so a "passing" test
may start failing after an unrelated refactor. Use it for CI reproducibility,
but don't rely on it for correctness — pair with committed `.hypothesis/`
database.

### `max_examples` Diminishing Returns

- **100** (default): Catches most shallow bugs. Fine for development.
- **1,000**: Catches interaction bugs. Good for CI.
- **10,000+**: Diminishing returns on random generation. Only useful if
  combined with `target()` for coverage guidance.
- Going from 100→1000 finds ~10× more bugs. Going from 1000→10000 finds ~2×.

## The Example Database — CI Pitfalls

- `.hypothesis/examples/` stores **choice sequences**, not generated values.
  They're compact but version-sensitive.
- **Commit `.hypothesis/` to VCS** — this is the #1 thing teams skip. Without
  it, CI rediscovers the same bugs from scratch every run.
- **CI caching the database across runs** works but beware: a schema change in
  your code can make old database entries produce `InvalidArgument` on replay.
  If CI starts failing with deserialization errors after a refactor, clear the
  cache.
- `@reproduce_failure` blobs are **version-pinned** — they break across
  Hypothesis upgrades. Use `@example()` for permanent regression tests instead.

## NEVER List

NEVER suppress `filter_too_much` — it means your strategy is architecturally
wrong, not that the check is too strict. A >50% rejection rate makes shrinking
nearly useless and generation 10× slower. Restructure with constrained ranges.

NEVER use `random.random()` inside a Hypothesis test — it introduces
non-determinism that breaks shrinking and reproducibility. All randomness must
flow through Hypothesis strategies.

NEVER call `.example()` inside a test — it bypasses the Hypothesis engine
entirely (no shrinking, no database, no reproducibility). It exists for REPL
exploration only.

NEVER set `max_examples` below 20 for "fast" local runs. Below ~30 examples,
Hypothesis can't effectively explore the input space OR shrink failures. Use
profiles: `settings.register_profile("dev", max_examples=100)` with
`HYPOTHESIS_PROFILE=dev`.

NEVER write `@given(st.integers().filter(lambda x: x > 0 and x < 10))` when
you can write `@given(st.integers(min_value=1, max_value=9))`. The filter
version rejects ~99.99% of integers. Hypothesis has smart filter rewriting for
some cases, but don't rely on it.

NEVER put `@reproduce_failure` in committed code — it's pinned to a specific
Hypothesis version and will break on upgrade. Convert to `@example()` instead.

## Decision Tree: Which Property Pattern?

```
Input → Output with known inverse?
  YES → Round-trip test (encode/decode, serialize/deserialize)
  NO  → Is there a simpler reference implementation?
    YES → Oracle test (compare fast impl vs slow-but-correct impl)
    NO  → Can you state an invariant over all outputs?
      YES → Invariant test (sorted output is same length, sum preserved, etc)
      NO  → "Does not crash" test (still valuable! catches ~40% of real bugs)
```

## When Primary Approach Fails

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| `HealthCheck.too_slow` | Strategy too complex or test does I/O | Profile the strategy with `--hypothesis-show-statistics`; move I/O to fixture |
| Flaky failures | Non-determinism outside Hypothesis | Check for global state, time-dependent logic, dict ordering |
| Shrunk example is unhelpful | Heavy `.filter()` or lossy `.map()` | Restructure strategy; ensure `.map()` transform is invertible or at least monotonic |
| `Unsatisfied assume()` | Precondition rejects >90% | Move constraint into strategy construction |
| Database replay crashes after refactor | Choice sequences reference old structure | Delete `.hypothesis/` cache and re-run |
| Stateful test never exercises some rules | Preconditions too restrictive or Bundle empty | Add `@initialize` to seed Bundles; loosen preconditions |

**Do NOT load `references/patterns.md`** for simple `@given` property tests —
it's only needed for stateful testing, recursive strategies, or CI profile setup.
