---
name: bolton-rapid-software-testing
description: Apply Michael Bolton and James Bach's Rapid Software Testing (RST) methodology — the testing/checking distinction, FEW HICCUPPS oracles, SFDIPOT product modeling, session-based test management, and three-part test reports. Use when designing a test strategy, writing or reviewing test charters, evaluating test automation proposals, framing bug reports, defending exploratory testing to managers or auditors, replacing scripted test cases, deciding what to automate, or when someone says "all our tests pass" and you need to know whether that means anything.
---

# Bolton & Bach Rapid Software Testing

## The one idea that reframes everything

In RST, **testing ≠ checking**. A *check* is an observation linked to a decision rule, both applicable non-sapiently. *Testing* is the human cognitive activity of evaluating a product by learning about it through experiencing, exploring, and experimenting. Machines and instructed humans can do checks. Only humans can test.

This is not pedantry. The RST Namespace exists because the word "test" was historically drained of meaning in programming (Turing called it "checking a large routine"; the distinction was clear in 1972's *Program Test Methods* and dissipated after). When someone says "the tests pass," ask: *which tests, and what does pass mean here?* In RST, a passing check means "we (or the automation) didn't notice anything that might be a problem" — not "the product is good."

## Mental model (load this into working memory)

```
TESTING                              CHECKING
─────────────────────────────────    ────────────────────────────────
Investigation, learning, judgment    Confirmation of expected output
Generative (produces test ideas)     Confirmatory (verifies a claim)
Cannot be automated                  Can be automated
Finds bugs we couldn't predict       Detects regression of known behavior
A performance, like dance            An artifact, like sheet music
Output: a story about the product    Output: a green/red signal
```

Checking is *part of* testing, not opposed to it. The error is conflation: pretending the green CI bar tells you the product is good. It tells you only that a finite set of decision rules did not fire.

## Before you do anything, ask yourself

1. **Am I being asked to test, or to check?** "Write a test for X" almost always means "write a check." That's fine — say so out loud.
2. **Who is the person who matters?** Quality is value to *some person(s) who matter(s)*. Different stakeholders disagree on what counts as a bug. Name them before reporting anything.
3. **What is my coverage with respect to *which model*?** Coverage is not a percentage. It is *how thoroughly I have examined the product with respect to some model*. Code coverage, risk coverage, function coverage, claims coverage are different models. 100% of one is 0% of another.
4. **What's my charter?** If you can't state your mission in one or two sentences, you're not testing — you're wandering. Use Hendrickson's template: *Explore (target) with (resources) to discover (information).*
5. **What oracle am I applying?** Every bug report rests on an oracle (a principle by which you recognize a problem). If you can't name it, you can't defend the bug. See `references/oracles-few-hiccupps.md`.

## Decision tree

```
Task includes the word "test"?
├─ Is it "write a test for X"?              → It's a CHECK. Build the check, don't pretend it's testing.
├─ Is it "find bugs in X"?                  → It's TESTING. Charter a session. Load references/sbtm-and-charters.md.
├─ Is it "improve our test strategy"?       → Load references/test-framing-and-reports.md and the FEW HICCUPPS reference.
├─ Is it "automate our regression suite"?   → It's CHECKING work. Load references/breaking-test-case-addiction.md FIRST.
└─ Is it "explain why a passing test isn't enough"? → Load references/test-framing-and-reports.md.

Stuck on what to test?
├─ Use SFDIPOT to model the product:        → references/oracles-few-hiccupps.md (Product Elements section)
└─ Use FEW HICCUPPS to recognize problems:  → references/oracles-few-hiccupps.md
```

## Procedures for the common tasks

### When asked to "test X" or "write tests for X"

1. **Disambiguate first.** Ask (or state): "Do you want me to *check* X (encode known expected behaviors as automated assertions) or *test* X (investigate it for problems we don't yet know about)?" These are different deliverables. Don't silently pick one.
2. **If checking**: write the smallest set of checks that would catch a real regression. State explicitly what risks the checks do and do not cover. Don't write checks that duplicate type-system guarantees.
3. **If testing**: write a charter using the Hendrickson template, then run a session (or simulate one in a report). Apply FEW HICCUPPS *retrospectively* to anything you observe. Output is not pass/fail — it is a session report with bugs, issues, and a coverage statement.

### When reviewing someone else's test plan or test strategy

Run it against this checklist (in order):

1. **Does the plan distinguish testing from checking?** If "automated tests" and "manual tests" are the only categories, the author has not internalized the distinction. Most "automated test" failures are actually checking failures.
2. **Does the plan name *which model* coverage is measured against?** "80% coverage" without naming a model is meaningless. Push back until a model is named.
3. **Does the plan name the stakeholders?** "Quality" without a person is theater.
4. **Are the charters descriptive or prescriptive?** If charters contain numbered steps, the plan is a script with charter cosplay. Rewrite as missions.
5. **Does the plan account for testability?** If S-time is not budgeted (setup, environment, debugging the test infrastructure), the schedule is wrong.
6. **Does the plan allow for follow-up sessions?** Bugs found in session 1 generate test ideas for session 2. A linear plan that doesn't reserve capacity for "we don't know yet" is brittle.

### When someone says "all tests pass" and wants to ship

Do not say "you can't ship." Say:

1. **"Which tests, and what model do they cover?"** Get the actual coverage relationship. Often it's code coverage of a small subset of the codebase. That's a narrow claim, not a quality claim.
2. **"What was tested *outside* those checks?"** If the answer is "nothing" — the tests are *all* the testing — name the FEW HICCUPPS principles that have not been examined and the SFDIPOT areas that have not been explored.
3. **"What is the worst plausible bug a passing check wouldn't detect?"** Make the failure mode concrete. "Could these checks pass while a logged-in user sees another user's data?" gets attention faster than "tests are not testing."
4. **Frame the gap, not the verdict.** Lighthouse, not captain. Tell the team what the unexamined risks are and let them decide.

### Fallback: if there's truly no time for any of this

When the request is "I need this in 20 minutes":
- Run a single time-boxed survey session (15 min) with charter "Survey (target) using SFDIPOT to discover the most likely failure modes."
- Apply FEW HICCUPPS retrospectively to anything surprising.
- Report: top 3 risks, what was not examined, what 1 hour of further testing would cover. That report is more honest and more useful than running 200 unrelated checks in the same time.

## Anti-patterns (NEVER list)

- **NEVER report "tests passed" as if that means the product is good.** Seductive because the green bar feels like proof. Consequence: stakeholders ship products on the strength of checks that confirm only what was anticipated, missing whole classes of unanticipated failure. Instead, report "these checks did not detect a problem" *and* describe what testing was performed beyond them and what was deliberately left unexamined.

- **NEVER write a charter that tells the tester *what to do*.** Seductive because it feels like leadership. Consequence: charters become scripts, testers stop thinking, the session produces what the charter-writer already knew. A charter is *descriptive of a mission*, not prescriptive of steps. Two sentences is plenty. If you're tempted to add a third, you're micromanaging.

- **NEVER let "100% coverage" be a goal.** Seductive because it sounds rigorous. Consequence: the team optimizes for the easiest-to-cover model (usually code lines) while leaving claims, risk, configuration, data, and time-based coverage at zero. Instead, ask "100% with respect to *which* model?" and pick the model that matches the risk.

- **NEVER frame a bug report in terms of the bug alone.** Seductive because it feels objective. Consequence: developers triage on technical severity and miss business risk; the bug gets "won't fix" and surfaces in production. Instead, frame the bug as a *story about risk to a person who matters* — name the stakeholder and the harm. Bolton calls this *bug advocacy*.

- **NEVER conflate a "bug" with an "issue".** Seductive because both are "problems." Consequence: testability problems (slow builds, no logs, no test environment) get logged as bugs and ignored, while real product bugs get diluted in the same backlog. RST distinction: a **bug** threatens the value of the *product*; an **issue** threatens the value of the *testing, project, or business*. Track them separately. Issues belong in the testing report's "obstacles" section, not the bug tracker.

- **NEVER accept "FDA / ISO / SOX requires detailed test case scripts" without reading the regulation.** Seductive because nobody wants to fight an auditor. Consequence: months wasted writing 100+ pages of "click here, observe this" that find fewer bugs than two paragraphs of test ideas would. James Bach replaced 50 pages of medical-device test scripts with two paragraphs (a general protocol + concise test ideas) and the auditors accepted it — and it found bugs the scripts had missed for years. See `references/breaking-test-case-addiction.md`.

- **NEVER claim "we didn't have enough time" without saying what *coverage gap* that produced.** Seductive because it sounds like an excuse the manager will reject. Consequence: managers hear "tester is making excuses" and discount the report. Instead: name the model, name the unexamined area, and let the person who matters decide whether to ship with that gap. The lighthouse reports the rocks; it does not steer the ship.

- **NEVER let your test report be just bugs.** A real RST report has *three stories*: (1) the status of the product, (2) how you tested it, (3) how good that testing was — including testability obstacles, things you couldn't cover, and what you'd recommend.

## Specifics that take years to learn

- **Session length: 60–90 minutes uninterrupted.** Below 45m there's not enough flow to find deep bugs; above 90m human focus collapses and notes become useless. A "normal" RST tester completes **4–5 sessions per workday**, not 8 — debrief, admin, and bug investigation eat the rest. If your manager is planning 8 sessions/day, your manager has not understood SBTM.
- **TBS metrics** in a session report = percentage of session time spent on **T**est design/execution, **B**ug investigation/reporting, **S**etup/admin. These are *rough* percentages, not stopwatch precision. Their purpose is to reveal *patterns*: high B-time means buggy code (deeper testing needed there); high S-time means a testability problem worth reporting as an issue.
- **Testopsy**: a post-session close inspection of *how you tested*, not what you found. Bolton and Bach use it to surface tacit knowledge — the heuristics you applied without knowing it. Run one on yourself after a session that went unusually well or unusually badly.
- **Oracles are heuristic and fallible.** Every oracle is a guess at "what would be problematic," not a proof. The oracle can be wrong. The product can be inconsistent with its history *because we fixed a bug*. Always be ready to defend not just the bug, but the oracle that detected it.
- **The Productivity Paradox**: faster tooling (now AI) raises the apparent throughput of checking and lowers the *visible* cost of testing skill. The result is more code, more checks, fewer testers, and the illusion of higher quality — until a deep bug lands in production. RST's response: invest in skill and critical thinking, not just throughput.

## When to load the references

- **Designing a test strategy or charter** → READ `references/sbtm-and-charters.md` and `references/oracles-few-hiccupps.md`.
- **Writing a test report or defending your testing** → READ `references/test-framing-and-reports.md`.
- **Replacing scripted test cases or fighting bureaucracy** → READ `references/breaking-test-case-addiction.md`.
- **Trying to recognize a problem and you have no spec** → READ `references/oracles-few-hiccupps.md` (FEW HICCUPPS section).
- Do **NOT** load every reference for every task. Each reference is ~100–200 lines; loading all of them at once defeats progressive disclosure.
