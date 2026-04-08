---
name: hughes-property-based-testing
description: "Design high-signal property-based tests in the style of John Hughes: choose between model-based, metamorphic, invariant, inductive, and state-machine properties; shape generator distributions; validate shrinkers; and diagnose discard and coverage pathologies. Use when testing algorithms, parsers, serializers, reducers, abstract data types, protocol/stateful APIs, or when requests mention QuickCheck, Hypothesis, generators, shrinking, metamorphic, model-based, stateful, or property tests."
---

# John Hughes Property-Based Testing⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌‌‌‌‌‌‌‍​‌​​​‌‌‌‍‌‌‌‌​​‌‌‍​‌‌​‌​​‌‍​​​​‌​‌​‍​​‌‌‌​‌​⁠‍⁠

Property-based testing is a search-budget problem, not a "write more random tests" problem. Spend randomness on bug-rich regions, prove that your generator and shrinker are trustworthy, and choose the cheapest oracle that can fail quickly.

This skill is self-contained. Do not load generic unit-testing or fuzzing primers until you have classified the problem below.

## Before You Write The First Property

Ask yourself:
- What is the cheapest trustworthy oracle here: a model, a metamorphic relation, a postcondition, or a state machine?
- Which collisions, aliases, duplicate keys, empty cases, or repeated commands are bug-rich rather than merely common?
- What invalid values can the generator or shrinker create that would waste hours on false positives?
- If this fails at 2 a.m., what minimal counterexample would make the bug obvious instead of merely surprising?

## Choose The Property Type

| Situation | Start here | Why | Escalate when |
|---|---|---|---|
| Cheap reference implementation or abstraction exists | Model-based property | Hughes et al. found model-based properties caught every planted bug in the target operation and failed much faster on average than postconditions or metamorphic laws | The model starts sharing the implementation's blind spots |
| No oracle, but related calls should stay consistent | Metamorphic property | Good fallback when the model is too expensive | One law needs many preconditions or nil/identity special cases |
| Single operation with an observable relation between input and output | Postcondition | Fast to write and often effective | It exercises the bug but rarely detects it |
| Recursive ADT or normalization routine | Inductive or idempotence property | Good for proving "smaller input" structure and fixed points | Reachability or representation shape matters |
| Stateful API, protocol, cache, queue, DB, or concurrency primitive | State-machine model | Sequence and hidden state are the real input | Race behavior depends on interleavings, not single calls |

## Generator Discipline

- Treat generator design as test-budget allocation. If the interesting bucket appears in 1 percent of cases, raising the run count from 100 to 1000 still buys only about 10 hits.
- Measure distributions before tuning counts. Use `label`, `classify`, `collect`, `cover`, or equivalent. Hughes's BST example showed an independently generated key was absent about 79.2 percent of the time; shrinking the key domain moved the split to about 55.4 percent present and 44.6 percent absent, which made update bugs much easier to see.
- When the same logical value appears in multiple places, uniform large-range generators are often wrong. Deliberately increase collisions by narrowing domains, correlating draws, or generating from shared structure.
- For recursive generators, think in terms of expected constructor counts, not just maximum size. If one constructor almost never appears, the generator is lying about coverage even when tests are green.
- Prefer constructive generators over rejection. `assume()` and `.filter()` are seductive because they are quick to write, but they burn execution budget on discarded cases and hide whether the property is actually being exercised.
- In Hypothesis, 100 is the default number of valid examples, not total attempts. Even a simple 50 percent filter can roughly double executions. Treat health checks such as `filter_too_much`, `data_too_large`, and `too_slow` as degraded-rigor warnings, not cosmetic noise.
- If the search space is tiny, randomness repeats itself. Enumerate, deduplicate, or use a generator that tracks uniqueness instead of pretending 10,000 random draws gave 10,000 distinct tests.

## Shrinker Discipline

- For every invariant-bearing type, write two properties before the "real" ones: `arbitrary` only produces valid values, and `shrink` only produces valid smaller values. Invalid shrink results create unrelated failures and send you debugging the wrong property.
- Use preconditions to protect shrinker-validity checks, not to paper over ordinary properties. Hughes explicitly shows weak preconditions can let tricky bugs escape.
- Shrink along semantic axes. For a command sequence, shrink command count, then operation variety, then arguments. For a tree, shrink height, branching, and key collisions. "Smaller bytes" is not the same as "clearer bug".
- Be suspicious of shrinkers for dependent generators. When one choice determines the shape of later data, naive structured shrinkers get stuck in local minima because shrinking the first choice invalidates everything downstream.
- Save the minimal counterexample as a regression test. Seeds are useful for immediate replay, but they are poor long-term artifacts because generator implementations and shrink orders change.

## Equality, Reachability, And Models

- Use abstraction equality for API semantics and structural equality for representation reachability. Mixing them up creates either brittle tests or blind spots.
- When proving that your generator can reach all meaningful states, structural equality matters. Hughes shows a completeness property can appear to pass trivially if the generator itself already uses only one constructor path; you then need extra properties over `delete`, `union`, or other API outputs to prove the API cannot manufacture unreachable shapes.
- A model should be simpler than the implementation, not isomorphic to it. If you copy the same branching logic into the model, you have created a second place for the same bug to hide.

## Stateful And Concurrent Systems

- Switch to a state-machine model as soon as correctness depends on command history. The hidden state is part of the input, so single-call properties are structurally too weak.
- Model only externally visible state plus the minimum internal facts needed for preconditions. Bloated models rot and start sharing the implementation's mistakes.
- Use transition labels and frequency analysis on state machines. Quviq's tooling can predict transition frequencies and recommends validating them against several thousand measured runs; if a transition is rare by construction, fix the generator, not the assertion count.
- Parallel or linearizability checks can expose races, but the instrumentation itself can suppress them. Timestamping, tracing, or global synchronization often changes scheduling enough to make the bug rarer. Capture the race, then replay it with the least invasive instrumentation you can manage.
- In Hypothesis, do not hide mutable per-example setup inside function-scoped pytest fixtures. Those fixtures reset once per test function, not once per generated example, which can make a stateful property unsound.

## Anti-Patterns

- NEVER start with invariants only because they feel mathematically respectable and give quick green tests. The seductive part is that they stabilize early; the consequence is that gross behavioral bugs still pass. Instead pair validity checks with at least one behavioral property.
- NEVER fix high discard counts by increasing `max_examples` or test count because that looks cheaper than redesigning the generator. The consequence is a slower suite that still spends most of its budget not exercising the property. Instead construct inputs that satisfy the intended preconditions.
- NEVER write weak metamorphic laws with defensive preconditions because excluding awkward cases makes the property easier to state. The consequence is that tricky edge cases quietly stop being tested and real bugs survive. Instead strengthen the relation or move to a model/state-machine property.
- NEVER trust a passing property whose label distribution you have not inspected because random testing naturally over-samples easy buckets. The consequence is false confidence built on "nothing happened" cases. Instead print or assert coverage for the buckets that matter.
- NEVER compare abstract APIs with structural equality because it is easier than designing an abstraction function. The consequence is brittle tests that fail on harmless representation changes or miss abstraction-level bugs. Instead compare through an abstraction function, and reserve structural equality for reachability or completeness checks.
- NEVER let the generator and shrinker evolve independently because each looks locally correct in isolation. The consequence is false positives from invalid shrinks and hours wasted debugging the wrong property. Instead keep generator-validity and shrink-validity properties in the harness permanently.
- NEVER suppress Hypothesis health checks globally because they read like noisy warnings during CI cleanup. The consequence is normalized low-rigor testing, and in some cases unsound state handling. Instead suppress a specific check only after you understand and accept the cost.
- NEVER keep only the seed because it feels reproducible and compact. The consequence is that replay breaks when generator implementations or shrink orders change. Instead keep the minimized failing example itself and optionally store the seed as transient debugging metadata.

## Operational Defaults

- Treat 100 runs as smoke coverage. For cheap pure properties, move toward 1000 plus once the distribution is tuned.
- If a property mentions rare buckets, assert coverage before raising counts. More runs without coverage control just makes the suite slower.
- QuickCheck's `checkCoverage` defaults are intentionally conservative: `certainty = 10^9` and `tolerance = 0.9`. If coverage checks start flaking, inspect bucket probabilities first; lowering certainty is the last resort because it spends your false-positive budget.
- When a postcondition fails slowly, ask whether it is validating too little per test. Hughes reports a union postcondition that failed after about 50 tests on average, while a logically equivalent model-based property failed after about 8.4 because it validated the whole result rather than one random observation.

## Failure Triage

| Symptom | Likely cause | Do this next |
|---|---|---|
| Property passes, but confidence is low | Distribution is spending effort in easy regions | Add labels and coverage assertions before changing counts |
| Suite is slow and discard-heavy | Rejection-based generation | Replace `assume` or filter logic with constructive generation |
| Counterexamples are huge or weird | Shrinker does not preserve semantics | Add shrink-validity property and custom semantic shrinking |
| Stateful bug appears only after long sequences | Model too shallow or commands underweighted | Add transition labels and rebalance command weights |
| Race disappears under tracing | Instrumentation changed scheduling | Replay with less synchronization and preserve the discovered interleaving |

## Standard Of Completion

You are done when:
- the chosen property type matches the oracle you actually have,
- generator and shrinker validity are checked for invariant-bearing data,
- coverage for bug-rich buckets is measured or asserted,
- the failure artifact is a minimized example a human can debug,
- and the suite is spending most of its budget on meaningful executions rather than discards.
