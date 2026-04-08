# TDD Decision Trees⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌‌​‌​​​‍‌‌​‌‌‌​‌‍‌​​​‌​​‌‍‌‌‌​​‌‌​‍​​​​‌​‌​‍‌‌​​‌‌​​⁠‍⁠

Operational decision tables for the moments where TDD practitioners hesitate. Each tree tells you what to do *and* what the expert mistake is for that situation.

## Tree 1 — Which gear?

```
Do you know exactly what to type?
├── YES, and you can type it in <30 seconds
│   └── Obvious Implementation. Run tests. If green, commit.
│       └── If you're surprised by red >1 in 5 → you're wrong about
│           "knowing", downshift to Fake It next time.
│
├── SORT OF — you can see the test pass but the implementation feels foggy
│   └── Fake It. Return a constant that makes this test pass.
│       Then add a second test that forces the constant to become a variable.
│       Replace one constant at a time.
│
└── NO — you don't know the right shape of the answer
    ├── Is this an algorithmic question (Sudoku, pathfinding, parser)?
    │   └── STOP. TDD won't substitute for thinking.
    │       Sketch on paper / REPL / oracle. Come back when you have
    │       a hypothesis to test.
    │
    └── Is the abstraction unclear (one class or two? merge or split?)
        └── Triangulate. Write a SECOND concrete example that
            cannot pass with the constant. The diff between the two
            tests reveals the abstraction. Beck only does this when
            "really, really unsure" — most days he never triangulates.
```

**The expert mistake:** treating Fake It and Triangulate as the *default* TDD ritual. They are recovery moves. Most of Beck's day is Obvious Implementation.

## Tree 2 — Should I write a test for this?

Beck's framing: every test is an **expense** (write + maintain) that must be paid for in **feedback** (informational + emotional). Every test you don't write is a **risk**.

```
Are you confident the code is correct without a test?
├── YES, very confident, the code is internal, callers are trusted
│   └── Don't write the test. (Beck does this constantly.)
│       The risk is tiny; the cost would not pay for itself.
│
├── YES but the code is part of a public API or untrusted callers exist
│   └── Write the test. The risk profile justifies the cost —
│       you cannot guarantee future callers will respect invariants.
│
├── NO, you'd be guessing
│   └── Write the test. This is what TDD is FOR — fear → boredom.
│
└── NO, and writing the test feels harder than writing the code
    └── DON'T write the test yet — fix the INTERFACE.
        A hard-to-test interface is the canary. Refactor for testability,
        then test.
```

## Tree 3 — A test went red. What do I do?

```
Was the suite green before your last change?
├── YES (your change broke it)
│   ├── Was the change supposed to add behavior? → expected red, you're in cycle. GREEN it.
│   └── Was the change a refactor? → REVERT immediately. Refactors must keep tests green.
│       Do NOT "fix" the test. Do NOT keep the change "and fix the test next."
│
└── NO (someone else's commit, or env change, broke it)
    ├── Is it deterministic? Same red every run?
    │   └── Bisect. Find the commit. Fix root cause.
    └── Is it flaky? Different reds or intermittent?
        └── This is a Determinism violation. Find the source of
            non-determinism (clock, RNG, network, parallel access)
            and INJECT it. Never mark the test "retry on failure" —
            that trains the team to ignore red.
```

## Tree 4 — Classicist (Detroit) vs Mockist (London) for this collaborator?

Beck is a Detroit/classicist. The choice matters because it determines whether your tests survive the next refactor.

```
Is the collaborator yours (in this codebase, you can change it)?
├── YES, and it's fast and deterministic in tests (pure logic, in-memory)
│   └── DETROIT. Use the real object. Assert on observable state of the
│       subject after the call. Refactor-friendly.
│
├── YES, but it's slow / non-deterministic / has I/O
│   └── Write a TEST DOUBLE you control (Fake, not Mock).
│       A Fake is a real working implementation with shortcuts
│       (in-memory repo, fake clock). Beck prefers Fakes over Mocks
│       because Fakes preserve structure-insensitivity.
│
└── NO, the collaborator is third-party (AWS SDK, Stripe, requests, OS)
    └── Write a thin ADAPTER you own. Mock the adapter in unit tests.
        Cover the adapter itself with a small integration suite that
        hits the real thing (or its sandbox). Never mock what you don't own.
```

**The mockist trap:** strict mocks of your own collaborators welds tests to the call sequence. You cannot rename a method, reorder calls, or extract a helper without breaking 50 tests for no behavior change. This is what DHH labelled "test-induced design damage."

## Tree 5 — I want to commit. Is this commit clean?

The Tidy First rule: **structural and behavioral changes never share a commit.**

```
Look at the diff. Does any test go from green to red, or red to a
different green, between the parent commit and this commit?
├── YES — this is a BEHAVIORAL commit.
│   └── The diff should contain ONLY: the new/changed test, and the
│       minimum production code to satisfy it. If it also contains
│       renames, extracts, or moves → SPLIT.
│
└── NO — every test passes both before and after, with no test changed.
    └── This is a STRUCTURAL / TIDYING commit.
        It should contain ONLY: renames, extract method/class, inline,
        move, reformat. If it also contains a logic change → SPLIT.
```

**Splitting workflow:** `git stash` the unrelated half, commit the half you want, `git stash pop`, commit the rest. Two clean commits, two reviewable diffs, two safe revert points.

## Tree 6 — When this is NOT a TDD task

Route out of this skill if:

- **You're refactoring with no behavior change.** Use a refactoring discipline (extract, inline, rename) with the existing test suite as your safety net. TDD is not the right tool — you're not writing new tests, you're proving existing ones still pass.
- **You're exploring an unknown input format** (log scraping, HTML parsing, ML output). Use a REPL + visual inspection to discover the shape, *then* TDD the parser once you know what "right" looks like.
- **You're tweaking a UI / pixel-level change.** Your eyes are the oracle. Snapshot or visual regression tests, not TDD.
- **You're spiking to learn an API.** Write a Learning Test if you must, but mark it `@spike` and delete it after. Don't pretend the spike is production code.
- **You're under a deadline with code that has zero tests** (legacy rescue). You need Michael Feathers' *Working Effectively with Legacy Code* — characterisation tests, seams, sprout method/class. TDD assumes you can write a test in seconds; legacy assumes you cannot.

## Tree 7 — TCR (test && commit || revert) — should I use it?

Kent Beck's experimental workflow: every test run that passes auto-commits; every failure auto-reverts the working tree.

```
Are you in a context where:
- The codebase is small and well-tested
- You're doing a focused, single-feature session
- You want to *force* yourself into smaller steps
- You're okay losing 30s of work to gain rhythm
?
├── YES → Try TCR. The forced reverts feel awful for an hour and then
│         retrain your motor cortex to take genuinely tiny steps.
│         Beck reports the surprise: it works.
│
└── NO  → Plain TDD with manual commits. TCR is a discipline drill,
          not a daily driver for most teams.
```

**The non-obvious benefit of TCR:** it counteracts the sunk-cost fallacy. Code you spent 10 minutes writing that doesn't work is *gone*, instantly, before you can talk yourself into salvaging it. You almost always find a smaller, surer way the second time.

## Tree 8 — How many tests is "enough" for this feature?

Beck's pragmatic answer: **write tests until fear becomes boredom.**

```
Look at your test list. For each remaining item, ask:
- "If I deleted this test, would I worry about the system in production?"
  ├── YES → keep / write it.
  └── NO → don't write it. Risk is low enough; the cost wouldn't pay back.

When the test list is empty AND you're not adding new items each cycle
AND you're bored running the suite green → you're done.
```

The Triangle Classifier example: Beck shipped with 6 tests. A professional tester wrote 65 tests for the same problem. Both were "right" — they were solving different problems (programmer confidence vs. exhaustive validation). Know which one you're doing.
