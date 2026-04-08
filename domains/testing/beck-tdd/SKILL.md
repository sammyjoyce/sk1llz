---
name: beck-test-driven-development
description: Apply Kent Beck's Test-Driven Development the way Beck actually practices it — three gears (Obvious Implementation, Fake It, Triangulate) chosen by confidence, the 12 Test Desiderata as tradeoffs, Detroit/classicist state-based testing, Tidy First separation of structural from behavioral commits, and the test list as a TODO buffer. Use when writing new code from a known interface, growing a tricky algorithm, hardening a reproduced bug, deciding whether to TDD at all (Beck does not TDD log parsers, exploratory UI, or throwaway spikes), or driving an AI coding agent that wants to greenwash tests. Triggers — "TDD this", "red green refactor", "write tests first", "fake it till you make it", "triangulate", "Kent Beck style", "tidy first", "TCR", "test && commit || revert", "test desiderata", "classicist", "Detroit school", "one test at a time", "test list", "babysteps".
---

# Kent Beck Test-Driven Development

> "TDD is a way of managing fear during programming." — Kent Beck
> "The trick is to move slowly very very frequently." — Kent Beck

TDD is a **design discipline that uses tests as feedback**, not a testing strategy. Tests are the byproduct; the product is a design that emerged under the pressure of having to call it from outside before writing the inside.

## Step 0 — Decide whether to TDD at all

Beck himself does NOT TDD everything. Before starting, ask:

| Situation | Beck's choice | Why |
|---|---|---|
| Known interface, uncertain implementation | **TDD** | Tests pin down behavior while you flail |
| Unstructured input (log parser, scraper, ML output) | **No TDD** — visual inspect + REPL | You can't write an assertion for a shape you don't yet know |
| GUI tweak / pixel push | **No TDD** | The oracle is your eyes; tests would lag |
| Throwaway spike / learning a new API | **Learning Test** instead | Test exists to teach you, not to live |
| Reproducing a bug | **Always TDD** — failing test first | The test is your reproduction; never delete it |
| Algorithm whose shape you don't understand (Sudoku, parser combinator) | **Stop and think first** | Ron Jeffries' Sudoku saga — TDD does not substitute for insight |

**If the test you'd write doesn't fit on one screen and you can't predict whether it will pass, the problem isn't ready for TDD yet — slice it smaller or sketch first.**

## The three gears — pick by confidence, not by ritual

Beck's most-misread teaching. He uses **Obvious Implementation by default** and only downshifts when surprised:

1. **Obvious Implementation (2nd gear, default)** — type the real code. Run tests. If you stop being surprised by red bars, stay in this gear. Beck: *"Would I really use Fake It to implement plus()? Not usually."*
2. **Fake It Till You Make It (1st gear)** — return a constant, then gradually replace constants with variables. Use when you can see the test pass but the implementation feels foggy.
3. **Triangulation (lowest gear)** — write a *second* example that forces generalization. Beck: *"I only use Triangulation when I'm really, really unsure about the correct abstraction."*

**Decision rule (Beck's literal words):** *"If you don't know what to type, type the Obvious Implementation. If you don't know what to type, then Fake It. If the right design still isn't clear, then Triangulate. If you still don't know what to type, then take a shower."*

**Calling the shot:** Before pressing run, predict pass or fail. If you're wrong more than ~1 time in 5, downshift. Frequent surprises = your steps are too big.

## The 12 Test Desiderata — pick your tradeoffs deliberately

Beck's actual list (NOT the FIRST acronym, which is someone else's). Tests are points in a 12-dimensional space; no test maxes all 12, and no property should be given up without receiving more of another:

| Property | What it means | Common tradeoff |
|---|---|---|
| **Isolated** | Order of execution doesn't change outcome | Pay setup cost per test |
| **Composable** | Trim redundant assertions; later test ⊃ earlier ⇒ delete the redundancy from the later one (NOT the test) | Less obvious for first-time readers |
| **Fast** | Sub-second per test, ideally sub-100ms | Forces fakes over real I/O |
| **Inspiring** | Green bar makes you confident, not anxious | More tests, more maintenance |
| **Writable** | Cheap to write relative to code under test | Pushes you toward better interfaces |
| **Readable** | Reveals motivation, not just mechanics | Costs naming effort |
| **Behavioral** | Fails when behavior changes | Conflicts with structure-insensitive when over-mocked |
| **Structure-insensitive** | Does NOT fail when internals are refactored | Strict mocks destroy this — it's the #1 cause of "test-induced design damage" |
| **Automated** | No human in the loop | CI cost |
| **Specific** | Failure points to one cause | Conflicts with composable |
| **Deterministic** | Inject clocks, randoms, network — never let tests own them | Costs an injection seam |
| **Predictive** | Green ⇒ shippable | Unit tests are weak here; that's fine — that's what the suite is for |

The ones Claude routinely violates without realising: **Structure-insensitive** (over-mocks), **Deterministic** (lets test grab `now()`), and **Composable** (copies the previous test and just adds a line, never trimming the redundant prefix).

## Tidy First — never mix structural and behavioral changes in one commit

Beck's hardest-won discipline (see *Tidy First?*). A commit either:

- **Changes behavior** (a test goes red→green or red→red with new content), OR
- **Changes structure** (rename, extract, inline, move — every test stays green at every step)

**Never both in the same commit.** Mixed commits are unreviewable: a reviewer can't tell which line is the meaning and which is the move. When you spot a tidying you want, **stash, tidy, commit the tidy, unstash, continue**. The order matters: usually tidy *first*, because tidying makes the behavior change easier — but if you don't yet know what tidying you need, do the behavior change ugly, then tidy after.

## Working with an AI coding agent (you)

Beck has publicly documented the failure: **agents want to write code first, then write tests that pass it.** They will also delete or weaken failing tests to "fix" them. Guardrails:

- **Write the test first in a separate turn**, then run it and *show the red bar* before writing implementation. Don't proceed until red is verified — assume you'll cheat otherwise.
- **NEVER edit the test to make it pass.** If the test seems wrong, articulate *why*, propose the new test, get approval, then change.
- **If a test that was green goes red after a refactor, the refactor is wrong** — revert, don't "fix" the test. (The whole point of refactor's contract is that tests stay green.)
- Consider **TCR** (`test && commit || revert`) for high-stakes loops: every red bar wipes the change, forcing genuinely tiny steps. Hostile to long flailing sessions; that's the feature.
- Solo end-of-session: **leave the last test broken** so tomorrow-you has an obvious entry point. Team end-of-session: **all tests green**.

## The test list

Before starting, write a flat TODO list of every test you can imagine. **Do not write the tests** — just list them. As ideas surface mid-cycle, add to the list (so they don't break flow). Cross off as you complete. When the list is empty and no new items have been added in a cycle or two, you're done.

## Anti-patterns — every NEVER has a non-obvious reason

**NEVER refactor while red.** The seductive lie: "I'm right there, I'll just fix this one thing." Consequence: you now have two unknowns (did the refactor break it? did the original change break it?) and no working baseline to bisect against. **Instead:** revert to green (or hardcode-pass), THEN refactor, THEN re-attempt the real change.

**NEVER write more than one failing test at a time.** Two reds means you can't tell which one your next change addresses. You also lose Beck's "shot calling" — the prediction that builds intuition. **Instead:** jot the second idea on the test list and finish the first.

**NEVER mock what you don't own.** Mocking the AWS SDK or `requests` couples your tests to *your guess* about how they behave; the next library upgrade silently invalidates every mock. **Instead:** write a thin adapter you DO own, mock the adapter, and cover the adapter with a small set of integration tests against the real thing.

**NEVER strict-mock collaborators in your own codebase** (the London-school overreach). It welds tests to call sequence, killing structure-insensitivity. The first refactor that reorders calls breaks 50 tests for no behavior change — this is "test-induced design damage." **Instead:** Detroit-style — use the real collaborator and assert on observable state.

**NEVER assert on `toString()` / serialized output as a proxy for state.** Meszaros calls this *Sensitive Equality*. A formatting tweak in one place reddens hundreds of unrelated tests. **Instead:** assert on the structured field you actually care about.

**NEVER let tests share fixture state across runs (DB rows, temp files, env vars, singletons).** Order-dependent suites fail mysteriously in CI parallelism and can't be bisected. **Instead:** every test builds its own world and tears it down (Beck: "leaves the world the way it found it"). Only clean up *after*, never *before* — cleanup-before hides the fact that the previous test was dirty.

**NEVER one-assertion-per-test as a literal rule.** Beck's rule is *one logical concept per test*; multiple physical assertions about the same concept (object's three fields after construction) are fine. The literal rule produces 60 tests for triangle classification when Beck writes 6. **Instead:** ask "if this test fails, do I learn one thing or many?"

**NEVER stay red longer than ~1 minute.** Once the red period exceeds the cycle Beck calls "two breaths," you've lost the rhythm and started speculating. **Instead:** revert to last green, take a smaller step, even if it feels embarrassing.

**NEVER triangulate when you already know the answer.** Beck: triangulation is *teensy-weensy tiny steps* reserved for "really, really unsure." Two examples to drive `plus(a,b) → a + b` is theatre, not discipline. **Instead:** Obvious Implementation, then add the second test only if it covers a *different* case (negative, overflow, empty).

## When you need more

- **Deeper on the 12 desiderata, with what each one looks like in code:** READ `references/test-desiderata.md`.
- **Decision tables for triangulation vs fake it vs obvious, the gear-shifting heuristic, classicist vs mockist tradeoffs, and the Tidy First commit cadence:** READ `references/decision-trees.md`.
- **For pure refactoring without behavior change**, this is not the right skill — `references/decision-trees.md` § "When this is not a TDD task" routes you out.

Do NOT load reference files for routine red-green-refactor on a known interface — the body above is sufficient.
