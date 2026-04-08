# The 12 Test Desiderata — Operational Detail⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​‌​​‌​‍‌​‌​​‌‌‌‍‌‌​​​‌​​‍‌​​‌​​​‌‍​​​​‌​‌​‍​​‌​​‌‌​⁠‍⁠

Beck published these in 2019 because he realised the qualities of good tests he assumed everyone knew were not in *TDD By Example*. They are **not a checklist** — they are 12 axes you trade against each other. Every test gives up some properties to maximise others. The skill is choosing the trade deliberately.

> "No property should be given up without receiving a property of greater value in return." — Kent Beck

## The 12, with the non-obvious failure mode for each

### 1. Isolated
Same result regardless of run order. The test creates its own fixture from scratch.

**Failure mode you'll write:** sharing a class-level `setUpClass` DB connection that the test mutates. Looks fine locally; explodes the moment CI runs in parallel or alphabetises differently.

**The fix that experts learn:** clean up *after*, never *before*. Cleaning before is a code smell that proves the previous test was dirty.

### 2. Composable
A subtle property Beck only fully articulated in 2025: **if test₂ would pass for every program where test₁ passes, the redundant prefix in test₂ is dead weight.** Trim it.

**Concrete example.** N×M variants — say 4 ways to compute interest × 5 ways to format it. Naive testing: 20 tests. Composable testing: 4 + 5 + 1 wiring test = **10 tests** with the same predictive power. The wiring test proves the orthogonal dimensions are actually orthogonal.

**Why nobody does this:** trimming an assertion *feels* like deleting safety. It isn't, if the deleted assertion was a strict precondition of a later one.

### 3. Fast
Sub-100ms per test, ideally microseconds. The unit you actually care about: **how long until the suite stops you from refactoring fearlessly?** Once any single suite exceeds ~10s, programmers stop running it on every save and TDD collapses into "test after."

### 4. Inspiring
Green should *feel* good. If your green bar produces anxiety ("…but does it really work?"), the suite has lost its inspiring property and you'll start ignoring it. Add the test you're afraid is missing.

### 5. Writable
Tests should be cheap to write *relative to the cost of the code they test*. Beck's diagnostic: **if the test is harder to write than the code, your interface is bad.** Don't fix the test — fix the interface. Difficult-to-write tests are the canary in the coal mine for poor design.

### 6. Readable
A test reveals *why* it exists, not just *what* it does. Name pattern Beck favours: `subject_circumstance_expectation` — e.g. `withdraw_whenInsufficientFunds_throwsAndLeavesBalanceUnchanged`. Avoid `test1`, `testWithdraw`, `testHappyPath`.

### 7. Behavioral
Test fails when behavior changes. Sounds trivial but is violated whenever you assert on `toString()` output, log messages, or HTML markup as proxies for state.

### 8. Structure-insensitive
**The desideratum Claude violates most often.** A test that depends on the *structure* of the code (which class delegates to which, the order of method calls, the existence of private helpers) breaks the moment someone refactors — even when behavior is identical.

**The killer:** strict mocks. Every `verify(mock).method(args)` is a structural assertion masquerading as a behavioral one. Beck's tweet that started the desiderata: *"Tests should be coupled to the behavior of code and decoupled from the structure of code."*

### 9. Automated
Runs without human intervention, no "and then click yes to the dialog." Includes: no manually starting external services, no env vars only you have, no "works on my machine" file paths.

### 10. Specific
When a test fails, the failure message tells you exactly where. The test exercises a small enough surface that "test failed" ≈ "this exact thing is wrong." A 200-line integration test that fails with "expected true, got false" is a diagnostic dead end.

### 11. Deterministic
If nothing changes, the result doesn't change. **Operative rule:** code that uses a clock, RNG, network, filesystem, or thread schedule must have those values *injected*, not pulled from ambient context. The test then passes a fixed value.

The expensive lesson: every flaky test you tolerate trains the team to ignore red bars. One flake erodes the discipline of the whole suite.

### 12. Predictive
If all tests pass, the system is shippable. Unit tests are *intentionally weak* on this — that's what acceptance/integration/canary tests are for. Don't try to make unit tests predictive by mocking everything; that destroys structure-insensitivity. Build the predictive property at the suite level, not the test level.

## How "kinds of tests" map onto the 12-d space

Beck's framing: a "kind of test" is just a point in this space — a deliberate set of slider settings.

| Kind | Maximises | Sacrifices |
|---|---|---|
| **Programmer / unit** | Writable, Fast, Specific, Isolated | Predictive, Inspiring (alone) |
| **Acceptance** | Readable (by non-programmers), Predictive | Fast, Specific |
| **Monitoring / canary** | Predictive (in production) | Automated (alerting only), Specific |
| **Property-based** | Behavioral, Predictive | Specific (shrinking helps), Readable |
| **Approval / golden master** | Writable (cheap to add), Behavioral | Structure-insensitive (very brittle to format changes) |

## Diagnostic questions for any test you're about to write

1. Which 3 desiderata is this test maximising?
2. Which 3 is it deliberately sacrificing?
3. Is the sacrifice paid for? If you're losing Specific and not gaining anything, the test is just bad.
4. If a teammate refactors the internals tomorrow without changing behavior, will this test fail? If yes, you have a structure-sensitivity bug — fix it now or accept you'll throw the test out next week.
