---
name: beck-tdd
description: "Apply Kent Beck style TDD as a step-size and design-control system: decide when not to TDD, choose Obvious Implementation vs Fake It vs Triangulate, keep red phases tiny, separate tidy from behavior, and avoid structure-cementing tests. Use when implementing behavior from a known interface, pinning a reproduced bug, deciding between real collaborator, fake, or mock, coaching an AI agent through red-green-refactor, or routing work out to learning tests, spikes, or legacy characterization when TDD is the wrong tool. Triggers: \"TDD this\", \"red green refactor\", \"Kent Beck\", \"fake it\", \"triangulate\", \"obvious implementation\", \"Tidy First\", \"test list\", \"Detroit school\", \"classicist\", \"TCR\", \"learning test\", \"characterization test\"."
---

# Beck TDD⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​‌​‌‌​‌‍‌‌​​​​‌​‍​​​‌​‌‌‌‍​​​​​​‌​‍​​​​‌​​‌‍​​‌​​​​​⁠‍⁠

Kent Beck style TDD is not "write tests first" as ritual. It is a control system for fear, step size, and design damage. Most failures come from the wrong move size, the wrong oracle, or tests coupled to structure instead of behavior.

## Use This Skill When The Problem Fits

Use TDD when the interface is mostly known, the oracle is cheap, and you can predict the next red or green transition before you run it.

Route out immediately when:

- The output shape is still unknown. Do a spike or learning test first.
- The oracle is visual or experiential. Use exploratory or visual testing.
- The code is legacy with no seam. Use characterization tests and sprout seams first.
- The task is pure tidying with no behavior change. Use refactoring discipline, not new tests.

## Before The First Test, Ask Yourself

- What uncertainty am I buying down: behavior, interface, or algorithm? TDD helps with behavior and interface pressure. It does not substitute for algorithmic insight.
- Can the next example fit on one screen and can I call pass or fail before I run it? If not, the slice is still too large.
- If this test were deleted tomorrow, what production risk would return? If the answer is "none", skip it.
- Is the pain in the test or in the interface? If the test is harder to write than the code, fix the interface before writing more test code.
- Will a teammate be able to refactor internals tomorrow without breaking this test? If not, you are about to write a structure-sensitive test.

## Operating Procedure

1. Write a flat test list before coding. New ideas go onto the list, not into a second live red bar. If the list grows faster than you cross items off for two cycles, stop coding and sketch the problem.
2. Choose the gear by confidence, not by ceremony.
   - Obvious Implementation is the default when you can type the answer in about 30 seconds and your surprise rate is below about 1 run in 5.
   - Fake It when you can see a passing example but the real implementation is still foggy. Replace one constant with one variable at a time.
   - Triangulate only when the abstraction is still unclear after the first passing example. Two examples that force no new distinction are ceremony, not feedback.
3. Keep red phases tiny. If a test stays red for longer than roughly a minute or two breaths of thought, revert to the last green state and take a smaller move.
4. Keep unit tests cheap enough to preserve rhythm. A useful target is under 100 ms per test; once a focused suite drifts past about 10 seconds, people stop running it on every change and TDD silently degrades into test-after.
5. Split behavior from tidying. A behavioral commit contains the failing test and the minimum code to satisfy it. A structural commit contains rename, extract, move, or inline steps with the suite green before and after. Never mix both kinds of change in one commit.

## Freedom Calibration

- High freedom: pure in-memory business logic with a known interface. Favor the smallest direct move that keeps feedback tight; do not add ceremony just to "look like TDD."
- Low freedom: time, randomness, filesystems, threads, network, or third-party APIs. Be rigid. Create seams first, inject non-determinism, and keep commit boundaries extremely clean.

## Collaborators, Doubles, And Determinism

Before you mock anything, ask:

- Do I own this collaborator?
- Can the real collaborator run fast and deterministically in tests?
- Am I asserting behavior, or just freezing call order?

Default choices:

- Use the real collaborator when it is in-memory, fast, and deterministic.
- Use a fake you own when speed or determinism matters but behavior still needs to feel real.
- Wrap third-party systems in an adapter you own, then mock only the adapter and cover it with a small real integration suite.

The expensive lesson: every flaky or structure-cementing test taxes the entire suite, not just itself. One tolerated flake teaches the team to ignore red. One strict mock of an internal collaborator makes the next refactor look like a functional regression when nothing user-visible changed.

## Anti-Patterns You Must Refuse

- NEVER write a second failing test because it feels efficient. It is seductive because it captures fresh ideas while you still have them. The consequence is you lose shot-calling and cannot tell which red your next edit is supposed to clear. Instead append the idea to the test list and finish the current red-green cycle.
- NEVER triangulate by ritual because two examples feel more rigorous than one. The consequence is fake abstraction pressure and slow, theatrical loops. Instead stay in Obvious Implementation until your surprise rate climbs above about 20 percent, then downshift.
- NEVER "fix" a refactor by editing the test because stale assertions are easy to blame. The consequence is you destroy the contract that refactoring preserves behavior, so you no longer know whether code or test moved. Instead revert to the last green state and re-run the structural change in smaller steps.
- NEVER mock what you do not own because copying a third-party call shape is faster than designing an adapter. The consequence is your tests validate your guess about Stripe, AWS, or `requests`, not the real integration, so upgrades break production while unit tests stay green. Instead introduce an adapter seam and cover that seam with a small integration slice.
- NEVER strict-mock your own collaborators because call-order assertions feel precise. The consequence is structure-sensitive tests that explode on harmless extracts, renames, or reordered calls. Instead prefer Detroit style state assertions or a fake that preserves behavior without freezing internals.
- NEVER clean shared fixtures before a test because it feels safer than teardown. The consequence is you hide the previous dirty test and preserve order dependence until CI parallelism exposes it. Instead make each test build its own world and clean up after.
- NEVER assert on rendered strings, logs, or serialized blobs as a proxy for domain state because snapshot-like assertions are cheap to add. The consequence is formatting churn detonates unrelated tests and destroys structure-insensitivity. Instead assert on semantic state, and reserve approval-style checks for outputs whose format is itself the product.
- NEVER keep using TDD on unknown input formats because writing assertions feels disciplined. The consequence is you freeze ignorance into tests and spend the afternoon debugging the oracle instead of the code. Instead spike in a REPL, capture real examples, or write disposable learning tests until "correct" is concrete.

## AI Agent Guardrails

- Force a visible red bar before implementation. Agents cheat by writing code first and backfilling agreeable tests.
- If a previously green test goes red during a refactor, revert code before touching the test.
- If the agent proposes changing test and implementation in the same breath, demand the behavioral reason. Mixed edits are where greenwashing hides.
- Use TCR only when the agent keeps taking oversized steps and the codebase is stable enough to tolerate frequent reverts. The pain is the point.

## Mandatory Reference Loading

- Before choosing between real collaborator, fake, mock, learning test, characterization test, or TCR, READ `references/decision-trees.md`.
- Before trading off fast vs predictive vs specific vs structure-insensitive, READ `references/test-desiderata.md`.
- Before designing a matrix of cases that looks combinatorial, READ `references/test-desiderata.md` for composability first.
- Do NOT load either reference file for routine pure-function or in-memory TDD on a known interface; the body above is enough.
- Do NOT load `references/test-desiderata.md` when the real question is "should I route out of TDD entirely?" That is a `references/decision-trees.md` problem.
