# Test Framing & The Three-Part Test Report

> **Load this when**: writing or reviewing a test report, defending a bug to skeptical developers/managers, explaining "we didn't have enough time," or framing your testing for an audience that doesn't trust testers.

## Test framing — what it is

**Test framing** (Bolton's term) is the chain of reasoning that connects what you observed in testing to a claim about the product, and back to the testing mission. Every bug report and every test report is implicitly a frame. The skill is making the frame explicit so it can be inspected and defended.

A complete frame answers, in order:
1. **What is my mission?** (Why am I testing this?)
2. **What did I do?** (Coverage with respect to which model?)
3. **What did I observe?** (The fact, separated from interpretation)
4. **What oracle made this look like a problem?** (Why I think this is a bug)
5. **Who is the person who matters here?** (Whose value is threatened)
6. **What should we do about it?** (A *suggestion*, not a directive — "I suggest, you decide")

If any link in the chain is missing, your bug or your report is brittle and a sufficiently aggressive developer or manager will demolish it. The frame is your defense.

## The three stories every test report tells

This is the move that separates RST reports from "QA status updates."

To test is to compose, edit, narrate, and justify **three stories**:

### Story 1 — The status of the PRODUCT

- How it failed and how it *might* fail
- In ways that matter to your various clients
- Bugs, risks, areas of concern
- Areas you tested deeply that look solid
- **Not** "tests passed/failed" — that's checking, not testing

### Story 2 — How you TESTED it

- How you configured, operated, and observed the product
- What coverage you achieved (with respect to which model)
- What you have *not* tested yet, but what you might test
- What you will *not* test at all unless something changes (testability gaps, missing access, missing data)

### Story 3 — How GOOD that testing was

This is the story almost no one tells, and it is the one that matters most.

- Risks and costs of testing itself
- What made testing harder or slower (testability problems, environment issues)
- How testable (or not) the product is
- What you need from the team to test better
- What you recommend

A report that only contains Story 1 is the standard QA report. A report that contains all three is an RST report. The three-story structure is the main thing managers should learn from RST.

## Bug vs Issue — the precise distinction

| | Bug | Issue |
|---|---|---|
| Threatens the value of | the **product** | the **testing, project, or business** |
| Examples | crash, wrong calculation, security hole, broken accessibility | slow build, no logs, missing test environment, unclear requirements, blocked access, flaky CI |
| Goes in | the bug tracker | the session report's "Issues" section, the test status report |
| Audience | developers + product | managers + leads |

Why this matters: when issues get logged as bugs, the bug backlog becomes full of "the staging environment is broken" and developers stop reading it. Real product bugs get diluted. Worse, the underlying problem — testability — never gets addressed because it's hidden in the bug count.

The conversation to have with a manager: *"We have 12 product bugs and 4 testing issues. Two of the issues are blocking 60% of testing. If you don't fix the issues, we cannot find more bugs."*

## How to frame "we didn't have enough time"

Wrong framing (sounds like an excuse):
> "We didn't have time to test everything."

Right framing (Story 3 in action):
> "Coverage with respect to the [risk model / function model / data model]: we tested deeply in areas A and B; we tested shallowly in C; we did not test D, E, F at all. The unexamined areas have these specific risks: [list]. The cost of examining D would be approximately [N sessions]. I am reporting these gaps so you can decide what to ship with."

The lighthouse metaphor (Connor Roberts, popularizing Bolton): the tester is a lighthouse. The lighthouse casts light on the rocks. It does not steer the ship and it does not prioritize the rocks. Reporting unexamined risk is the lighthouse doing its job. The captain (PM, manager, executive) decides whether to sail through anyway.

## Bug advocacy — framing bugs for different audiences

Bolton's principle: **a bug report is a work of technical writing, and a tester is the bug's lawyer.** The same observation gets framed differently for different audiences:

- **For a developer**: technical, specific, repro steps, environment, logs. Minimum framing — they share your context.
- **For a product manager**: in terms of *user experience and business risk*. "When a user does X, they see Y, which is inconsistent with the product's stated purpose of Z." Skip the stack trace.
- **For an executive**: in terms of *business impact*. "This is a risk to [revenue/reputation/compliance/safety]. The cost of fixing is N. The cost of shipping it is M."

Same bug, three frames. Each frame is honest. None is manipulative. The tester's job is to make the bug *legible* to the person who can act on it.

## "Pass" and "fail" are weak words

When a check "passes," it means: *we (or the automation) did not detect anything that might be a problem in this narrow respect.* It does not mean the product is good in that respect, and it certainly doesn't mean the product is good overall.

When a check "fails," it means: *we (or the automation) detected something that might be a problem.* It does not mean there is definitely a bug — the check might be wrong, the oracle might be wrong, the environment might be off. The next step is *investigation*, not a Jira ticket.

The skilled tester's question after running a check is not "did it pass or fail?" but **"what can I learn from this? And what *else* can I learn?"** Every check is multivariate: while it was running, you could have observed timing, memory, log output, side effects, network traffic. A check that "passes" is a wasted observation if you only look at the assertion.

## Test framing in practice — the "two questions"

Before reporting *any* finding, run it through these two questions:

1. **"How is this connected to the mission?"** If you can't connect the finding to the charter or the broader testing mission, the finding may still be valid but it doesn't belong in *this* report. File it for the next session.
2. **"What would make me wrong?"** What alternative explanations exist? Could this be a test problem rather than a product problem? Could the oracle be misleading you? An RST tester does this *before* hitting "submit," not after.

The second question is where amateurs and professionals part ways. Professional testers expect to be challenged and have already considered the challenge.
