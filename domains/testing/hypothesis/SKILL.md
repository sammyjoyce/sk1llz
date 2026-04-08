---
name: hypothesis-testing-execution-guide
description: Expert Hypothesis playbook for turning property-based tests into reproducible, high-yield bug hunts. Use when writing or debugging Hypothesis tests with `@given`, `RuleBasedStateMachine`, `target()`, health checks, seeds, deadlines, `max_examples`, `stateful_step_count`, example databases, or flaky CI failures.
---

# Hypothesis Execution Guide⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​​‌​​‌‍​​‌‌​​​‌‍‌‌‌‌‌​‌​‍‌‌‌​‌‌‌​‍​​​​‌​‌​‍‌‌‌‌‌​‌​⁠‍⁠

Use this skill when the hard part is not Hypothesis syntax, but deciding where search budget, shrinkability, and reproducibility are leaking away.

## Mandatory loading

Before writing or rewriting strategies, READ `references/patterns.md`.

Do NOT load `references/patterns.md` when you are only:
- replaying failures
- tuning settings
- diagnosing flakiness
- deciding between `@example`, `@reproduce_failure`, seeds, or databases

That file is for construction patterns. This file is for execution control.

## Before you touch anything, ask yourself

Before raising `max_examples`, ask: is the miss caused by a weak oracle, a bad input distribution, or an insufficient search budget? `max_examples` only fixes the third.

Before suppressing a health check, ask: what silent degradation is it warning about, and can I already see the same symptom in statistics?

Before adding `@example`, ask: am I preserving a proven regression, or freezing an unshrunk counterexample that will hide a simpler bug shape?

Before using `deadline`, ask: am I measuring algorithmic work or host jitter? Deadline failures sit on a threshold and shrink toward misleading boundary cases.

Before increasing `stateful_step_count`, ask: do I need deeper programs or more independent programs? This setting multiplies runtime with `max_examples`.

Before blaming Hypothesis for flakiness, ask: does the test depend on external state, per-example isolation, thread timing, or a changing draw sequence?

## Operating model

Treat `max_examples` as a budget of satisfying examples, not as a function-call count. The default profile starts at `max_examples=100`, `stateful_step_count=50`, and `deadline=200ms`, but Hypothesis can stop early when it exhausts a finite space, or call the test more times because of `assume()`, `.filter()`, oversized examples, shrinking, and `Phase.explain`.

Run `pytest --hypothesis-show-statistics` before changing settings. The stop reason and invalid-example counts usually tell you whether the problem is strategy shape, oracle shape, or actual search depth.

The built-in CI profile is a triage profile, not a discovery profile: it uses `derandomize=True`, `deadline=None`, `database=None`, `print_blob=True`, and suppresses `too_slow`. Keep a separate local profile if you want the example database and exploratory randomness to keep finding fresh bugs.

`@seed` and `derandomize=True` are run-level reproduction tools. `@example` is a permanent regression guard. `@reproduce_failure` is a temporary cross-machine replay tool. The example database is an iteration cache, never a correctness mechanism.

## Decision tree

If the test is slow, first inspect the data-generation fraction in statistics.
- If generation dominates, redesign the strategy.
- If the SUT dominates, only then consider `deadline`, lower-cost oracles, or a smaller domain.

If the test misses bugs, inspect the stop reason.
- If it stopped because of `max_examples`, you may need more budget.
- If invalids or oversized examples dominate, redesign the generator first.
- If the space exhausted early, adding `max_examples` is wasted motion.

If a failure appears only in CI, decide what kind of replay you need.
- For one-off local reproduction, use the printed `@reproduce_failure` blob and remove it after extracting a readable case.
- For a permanent regression, convert the minimized case to `@example`.
- For team-wide replay, share a database only when Hypothesis versions are aligned.

If a stateful test is underperforming, decide whether you need depth or breadth.
- Raise `stateful_step_count` when the shortest plausible failing program is longer than the current budget.
- Raise `max_examples` when bugs probably live in many short histories.
- Do not raise both until you know which axis is starving the search.

If targeted testing is tempting, decide whether you have a real search metric.
- Use `target()` only for metrics you genuinely want to maximize.
- Expect little effect below about `max_examples=1000`, and obvious effect closer to ten thousand examples per label.
- Keep the label count small; many labels dilute the same search budget.
- If the metric is only weakly correlated with the bug, `target()` can steer search away from the bug.

## Hard-won heuristics

For equivalent predicates, `.filter()` is usually cheaper than `assume()` because Hypothesis can retry within a single example instead of rejecting the whole example. That does not make `.filter()` good by default; it only means it is the lesser evil when constructive generation is awkward.

When you see the classic `filter_too_much` failure or statistics full of invalids, do not think "more examples." Think "my generator is lying about the domain." Rejection-heavy strategies distort the distribution and burn the shrinker on garbage.

Health checks such as `function_scoped_fixture` and `differing_executors` are often the first signal that your test harness, not your property, is broken. Treat them as design feedback until you can explain precisely why the warning is a false positive.

Use `Bundle.filter(...)` when rule validity depends on the drawn object itself. Use `@precondition` when rule validity depends on machine-wide state. This distinction matters because bundle filtering removes only bad candidates, while a precondition can make the entire rule disappear from the schedule.

Multiple `@initialize()` methods all run exactly once, but their order varies. If order matters, they are not independent initializers; collapse them or make the dependency explicit through normal rules and bundles.

`Phase.target` requires `Phase.generate`, and `Phase.explain` requires `Phase.shrink`. If you switch to a replay-only mode such as `phases=[Phase.explicit, Phase.reuse]`, you are validating known failures, not exploring.

`Phase.explain` can disappear under instrumentation. On Python 3.11 and earlier, coverage and debuggers can disable it, and PyPy is treated conservatively. Missing explanations do not imply a weak counterexample.

`@example` inputs do not shrink. Promote a failing case to `@example` only after you have the minimal human-readable counterexample you want to keep forever.

The example database is keyed to the test and can be invalidated by version upgrades or code changes. If a shared database behaves strangely across environments, align Hypothesis versions before you debug anything else.

Hypothesis is thread-safe in modern releases, but thread-timed data generation is still a flakiness trap. Generate data on the main thread, then pass immutable payloads into workers; do not let thread timing change the draw sequence of `@composite`, `data()`, `event()`, `target()`, or `assume()`.

## NEVER do these

NEVER "fix" `filter_too_much` by raising `max_examples`, because that is seductive linear thinking: more attempts feels like more coverage. Instead redesign the generator so valid cases are constructed rather than rejected; otherwise the concrete consequence is more CPU spent on the same blind spots and worse shrinking.

NEVER commit `@reproduce_failure` as the permanent regression, because the opaque blob feels like a quick win but is version-bound and run-format-specific. Instead replay with it once, extract the minimal readable case, and keep that as `@example`; otherwise the concrete consequence is a brittle test that breaks on version drift without teaching future readers anything.

NEVER rely on function-scoped pytest fixtures or decorator-based mocks for per-example isolation, because normal pytest semantics make this feel safe while Hypothesis reuses the same fixture across many examples. Instead do setup and teardown inside the test body or a context manager so each example starts clean; otherwise the concrete consequence is false flakiness from stale state that shrinking cannot explain.

NEVER raise both `max_examples` and `stateful_step_count` blindly, because "more search" sounds reasonable while actually trading breadth for depth in a multiplicative cost explosion. Instead choose the axis that matches the bug shape you are chasing; otherwise the concrete consequence is slower runs with fewer useful programs per minute.

NEVER let multiple `@initialize()` rules depend on one another, because separate methods look modular while their randomized ordering creates empty bundles and pseudo-flaky state machines. Instead use one initializer or model the dependency explicitly; otherwise the concrete consequence is a failure mode that looks nondeterministic but is actually scheduler-induced.

NEVER add `target()` just because you want Hypothesis to "try harder," because a weak metric is worse than no metric: it spends budget maximizing the wrong thing. Instead use `target()` only when you can name the monotonic quantity that should increase as the failure gets more interesting; otherwise the concrete consequence is search pressure being pulled away from the actual bug.

NEVER treat the example database as proof that a regression is covered, because replay feels reassuring while database entries are cache-like and may disappear after version or test-shape changes. Instead use `@example` for permanent correctness and the database for speed; otherwise the concrete consequence is a suite that passes locally until the cache changes.

## Fallback playbooks

If local runs are green but CI produced a blob:
- replay with the blob
- extract the minimized readable case
- convert it to `@example`
- remove the blob

If statistics show many invalid examples:
- move constraints into strategy construction
- only keep `.filter()` for predicates that truly depend on generated structure
- use `assume()` last, not first

If a state machine never reaches the interesting transition:
- seed the required resource into a bundle with `@initialize`
- lower rule fan-out before raising budgets
- use bundle filtering for per-object validity instead of global preconditions

If deadline failures hover right at the boundary:
- do not widen the deadline blindly
- separate the performance property from the correctness property
- inject deterministic clocks or counters when possible

If explanations disappear or look weak:
- remember that shrinking optimizes simplicity, not severity
- rerun without coverage/debugger if explanation matters
- inspect statistics or targeted scores to recover the lost severity signal

## Output contract when using this skill

Return:
1. the dominant failure mode
2. the reproduction mechanism to use
3. the search-budget decision you made
4. the shrinkability risk you are accepting
5. the exact next lever to pull if the first fix fails

Do not end with "ran more examples." End with what changed in search quality or reproducibility.
