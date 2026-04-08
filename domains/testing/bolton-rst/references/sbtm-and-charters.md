# Session-Based Test Management & Charters

> **Load this when**: writing a test charter, planning testing for a sprint or release, debriefing a session, or trying to make exploratory testing accountable to a manager.

## Why sessions exist

SBTM (Jonathan Bach, James Bach, around 2000 at HP) was invented to solve one specific problem: **exploratory testing was being dismissed as "ad hoc" and unmeasurable** by managers who only trusted scripted test cases. The session is the unit of accountability that makes exploratory testing tractable to management without killing the exploration.

A session is the unit, not a test case. *The test case is the wrong unit because it conflates the artifact with the performance.* (Bolton's analogy: a test case is a recipe; testing is cooking. Recipes don't make dinner.)

## Definition (from RST)

A test session is a period of time during which a person performs testing, and that:
1. Is **virtually uninterrupted**
2. Is **focused on a specific mission** (the charter)
3. May involve multiple testers; may or may not involve automation
4. Results in a **session report** of some kind
5. Is **debriefed** by the leader (unless the leader runs the session)

## Charter format — Hendrickson template

> **Explore (target) with (resources) to discover (information).**

Examples that work:
- "Explore the payment flow with different sequences of events to discover problems with transactions."
- "Explore the chat state model with a random walk of events and transitions to discover surprises."
- "Explore the new auth system with existing system capabilities to discover possible privilege-escalation risks."

The hard part: **the charter is descriptive, not prescriptive**. It describes the mission, not the steps. Two sentences max. If you find yourself adding "first do X, then click Y," you have written a script, not a charter, and you have killed the exploration before it began.

What kills charters in practice:
- Adding acceptance criteria ("verify the total is correct"). That belongs in a check, not a charter.
- Listing every input to try. That's a script.
- Writing it in past tense or as a requirement. A charter is a *mission*, not a contract.

## Session length: the 60–90 minute window

Why this specific range and not longer or shorter?

- **< 45 minutes**: not enough flow to find deep bugs. Short sessions become checking sessions by default — you do the obvious things and stop.
- **60–90 minutes**: long enough for the tester to get into the product, build a mental model, follow surprises down rabbit holes, *and* short enough that the human attention budget hasn't collapsed.
- **> 90 minutes**: focus collapses, notes become unreliable, and the inevitability of interruption rises sharply. You will pretend you tested for 2 hours but you actually tested for 70 minutes and stared at the screen for the rest.

The harder lesson: **a working tester completes 4–5 sessions per day, not 8.** Debriefing, bug investigation, meetings, email, build problems, and post-session writing eat the rest of the day. Any plan that assumes 8 sessions/day means the planner has not done SBTM and is going to underestimate testing time by ~50%.

## TBS metrics (the only metrics that matter in a session)

In every session report, record rough percentages:

- **T = Test design + execution**. Time the tester was actually hunting for bugs.
- **B = Bug investigation + reporting**. Time spent characterizing a bug after finding it. **B time interrupts T time.**
- **S = Setup + admin**. Configuring environments, reading docs, writing the report. **S time preempts T time.**

Why these and not "tests passed" or "bugs found":

- **High B-time** in an area = the area is buggy and needs deeper testing, *not* better automation. The bugs you stopped to report are the surface; there are deeper ones underneath.
- **High S-time** = a **testability problem**. This is the single most underused signal in software development. The team is paying for it whether it gets reported or not. If S consistently > 30%, file an *issue* (not a bug) about testability.
- **T below 50%** for sustained periods = something is broken in how testing is being supported, regardless of how many bugs were found.

These percentages are **rough** — to the nearest 10%. Stopwatch precision misses the point and turns the report into a timesheet.

## The PROOF debrief structure

After every session, a 15-minute debrief between the tester and a leader (manager, lead, or another tester). Use PROOF:

- **P**ast — what happened during the session?
- **R**esults — what was achieved?
- **O**bstacles — what got in the way? (testability issues, environment problems, missing information)
- **O**utlook — what still needs to be tested? what new test ideas emerged?
- **F**eelings — how does the tester feel about the session, the product, the risk?

Skipping the debrief is the most common SBTM failure. The debrief is where tacit knowledge surfaces, where the next charter is born, and where the manager actually learns the state of the product. Without it, SBTM becomes a paperwork ritual.

The Feelings part is non-negotiable and the part everyone tries to cut. Bolton's defense: feelings are *signals*. A tester who feels uneasy about an area without being able to articulate why has detected something. The debrief is where that something gets articulated.

## The session report (lightweight, not a deliverable)

Minimum useful content:

- Charter (one or two sentences, copied from when the session started)
- Tester name, date, duration
- Areas covered (free-form tags, not a hierarchy)
- TBS percentages
- **Bugs** (numbered, with IDs in the bug tracker)
- **Issues** (testing/project obstacles — this is the part most teams forget)
- **Notes** — free-form: what was tried, what surprised the tester, ideas for follow-up sessions

Mind maps work better than prose for notes when the testing was wide and shallow. Prose works better when the testing was narrow and deep. Pick the format the tester will actually maintain.

## How to charter for an unknown product

The hardest case: a tester is dropped onto a product they have never seen before. The charter cannot be specific because nobody knows what's in there. Bolton's heuristic: the **first session is always a survey session**.

- Charter: "Survey (product) using SFDIPOT to build an initial model and discover risks worth deeper investigation."
- Output of session 1 is *not bugs*. It is a list of follow-up charters. Resist any pressure to log bugs from the survey session — the tester doesn't know enough yet to recognize them as bugs.
- Sessions 2+ are real testing sessions, each with a specific mission derived from the survey.

If you skip the survey session and jump straight to "find bugs in the product," you will find the easy bugs, miss the deep ones, and feel falsely confident.

## Common SBTM failure modes

| Failure | Symptom | Cause |
|---|---|---|
| Charters become scripts | Testers complete the steps but never deviate | Charter author wrote prescriptive steps |
| Debriefs get cut | "We're too busy" | Manager doesn't see the value, has not been trained on PROOF |
| TBS becomes timesheets | Stopwatch precision, arguments over 5% | Treating rough percentages as accounting data |
| Issues get logged as bugs | Bug backlog full of "the build is slow" | No issues column in the report; nowhere else to put them |
| 8 sessions/day plan | Testers burn out or fake reports | Manager has not internalized the 4–5/day reality |
| Sessions fragment to 30 minutes | No deep bugs found, just shallow ones | Open-plan office, Slack, meetings; testability problem at the org level |
