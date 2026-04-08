---
name: bach-exploratory-testing
description: Apply James Bach-style exploratory testing and Rapid Software Testing to choose session types, size charters, frame bugs with strong oracles, decide when to repeat versus vary tests, improve testability, and debrief testers without collapsing into test-case theater. Use when you need survey, analysis, deep-coverage, or closure sessions; when SBTM is being introduced or audited; when a team is over-counting test cases or over-trusting automation; or when testing GenAI/LLM systems responsibly. Triggers include: exploratory testing, RST, SBTM, session-based test management, charter, oracle, A FEW HICCUPPS, HTSM, SFDPOT, CRUSSPIC STMPL, paired exploratory survey, blink test, pesticide paradox, shallow agreement, Bach, Bolton.
---

# Bach-style Exploratory Testing

## Work from these propositions

- Testing is a sampling problem, not a completion problem. Repeating a path through the minefield mainly avoids new mines; vary unless you can name a reason to repeat.
- The axis that matters is agency. If the tester cannot redirect the work, it is scripted regardless of whether a literal script exists.
- Separate experiential evidence from instrumented evidence. If the tool changes latency, workflow, or visibility that a real user would have, you are no longer testing the same experience.
- Risk expands toward the user-facing surface. Cheap low-level checks matter because they are controllable, not because they let you ignore the surface where people actually live.
- Most teams overstate coverage by one level. What gets reported as "well tested" is often only reconnaissance plus a few memorable stories.

## Before you start, ask four questions

1. What job am I doing right now?
   - Unknown area, emergency, or political fog: run a survey.
   - Suspicious behavior with a hunch worth squeezing: run an analysis session.
   - Known risk that deserves brutal combinations: run deep coverage.
   - Stakeholder wants confidence on a claim, fix, or release note: run closure.
2. What kind of evidence do I need?
   - User-experience evidence: stay experiential.
   - State, data, timing, or internal evidence: instrument deliberately and say so.
3. Why would I repeat instead of vary?
   - Good reasons: regression, intermittence, retry after suspect execution, deliberate mutation around a known path, contract/regulatory mandate, or cheap smoke coverage.
   - If you cannot name the reason, variation beats repetition.
4. What will make this reviewable tomorrow?
   - A charter, live notes, parsable artifacts, named oracles, and the decision this work is supposed to inform.

## Session selection

Use a 60/90/120 minute timebox. More than 4 real sessions per tester per day is usually theater.

- Survey: learn the terrain fast, characterise general risk, and generate the next charters.
- Paired Exploratory Survey: use when the area is new, dangerous, or politically charged. Let the less-experienced or domain-heavy partner drive; the senior tester navigates, documents, and keeps the story coherent.
- Analysis: stop wandering and squeeze one anomaly until the product, the oracle, or your model breaks.
- Deep coverage: attack a known risk with combinations of data, interruption, time, state change, and platform variation.
- Closure: answer a stakeholder claim in safety language, never in absolutes.

## Operating heuristics that usually take years to learn

- Before demanding more scripts, demand better logging. In Bach-style work, logging is automatic documentation. Ask for millisecond timestamps, unique event IDs, one-line parseable records, startup and shutdown state dumps, configurable detail levels, and a ring buffer sized for roughly 7 days of worst-case use.
- Treat a smoke suite as the one repetitive block you may deliberately preserve. It should touch broad surface area and finish in about 10 minutes, or at the extreme within 1 hour. Longer than that and you are buying reassurance, not signal.
- If output volume exceeds reading speed, run a Blink Test first. Scroll too fast to read; let anomalies choose where close inspection starts.
- When a whole area has "gone quiet", treat silence as a smell. Change technique, data class, starting state, platform, timing, or tester before concluding stability.
- When a bug "cannot be reproduced", classify the failure first: intermittence, missing observation, or wrong story. Each needs a different response.
- If a tester cannot tell the session story in two minutes, the notes are wrong or the charter was oversized.
- If a debrief takes more than 15 minutes, fix the sheet or the charter. Do not normalize bloated debriefs.
- Use coverage levels instead of confidence theater:
  - Level 0: black box.
  - Level 1: reconnaissance.
  - Level 2: core and critical aspects exercised meaningfully.
  - Level 3: harsh, exceptional, and state-stressing conditions explored.
  Teams regularly mistake Level 1 for Level 2.

## Oracles and reports

Before filing anything, ask yourself:

- What did I look at?
- How would I recognize a problem?
- What did I actually do?

If you cannot answer all three, you do not yet have a testing story.

Use A FEW HICCUPPS for fast framing. Use more operational oracles when you need checkable propositions. If the product feels wrong but the oracle is fuzzy, report the uncertainty explicitly; confusion is still evidence about model quality, documentation, or testability.

Before writing strategy or bug framing, READ `references/oracles-and-coverage.md`.
Do NOT load it for quick recall questions already answered in this file.

## Charters

A charter is only good if it buys information someone will act on.

- Good size: one tester reaches a satisfying stopping point in 60-120 minutes and produces a reviewable sheet.
- Too big: multiple incomparable stories could emerge.
- Too small: it is a check disguised as a charter.
- Use tours as lenses, not as whole charters: Money, FedEx, Landmark, Guidebook, Bad Neighborhood, Garbage, Museum, All-Nighter, Anti-tour.

Before writing or reviewing charters, READ `references/charters.md`.
Do NOT load it when you only need the charter sentence shape.

## SBTM and debriefs

Use SBTM to defend agency, not to bureaucratize it.

- Track T, B, and S within on-charter time, plus O outside it, but only as question-generators.
- High S is a testability problem. High B often means the tester is diagnosing for the team instead of extending coverage.
- PROOF debriefs matter because tacit knowledge leaks out in the F. If an experienced tester says "something about this feels rushed," charter that discomfort next.
- Never introduce the whole SBTM apparatus in week one. Start with mission plus live notes, then debriefs, then T/B/S, then parser tags only if you will actually parse them.

Before introducing SBTM or auditing sheets, READ `references/sbtm-session-sheet.md`.
Do NOT load it for quick questions about session duration or the PROOF mnemonic.

## NEVERs

- NEVER ask for repeatability as a proxy for discipline, because the seductive part is audit comfort and the consequence is minefield walking: you optimize for avoiding new information. Instead, name the reason for repetition and vary everything else you can.
- NEVER let tooling quietly alter the user experience while you still call the result "user testing", because the seductive part is richer observability and the consequence is false confidence about the surface where the people live. Instead, label the work experiential or instrumented and collect both kinds of evidence on purpose.
- NEVER polish session notes after the fact, because the seductive part is professional-looking prose and the consequence is loss of dead ends, timing, and uncertainty that explain the next bug. Instead, take ugly live notes and spend the last few minutes only making them legible.
- NEVER turn SBTM metrics into tester grades, because the seductive part is managerial neatness and the consequence is dishonest sheets, hidden setup pain, and suppressed bug investigation. Instead, use T/B/S/O to ask what constrained coverage.
- NEVER answer "is it ready?" with yes or no, because the seductive part is sounding decisive and the consequence is borrowed authority over a business decision you do not own. Instead, answer with coverage level, risks remaining, obstacles, and what one more day would buy.
- NEVER accept shallow agreement on words like tested, done, automated, quality, or AI, because the seductive part is faster meetings and the consequence is teams diverging under fake consensus. Instead, demand one concrete example from each speaker before planning.
- NEVER trust one-shot GenAI demos, because the seductive part is the impressive first answer and the consequence is certifying non-deterministic behavior you never measured. Instead, run the same task at least 10 times, preferably 25 for high-stakes work, mutate the prompt slightly, inspect every word, and use self-consistency or an external oracle where possible.

## Decision tree

- New product or no shared model yet:
  - Start with a survey or paired exploratory survey.
  - Build the next charter library from the risks and surprises you uncover.
- A repeated path keeps passing:
  - Assume pesticide paradox.
  - Mutate data, starting state, platform, timing, or tester before spending more passes.
- A bug seems intermittent:
  - Repeat only to separate intermittence from observation failure.
  - Add logging or recording before you add more scripts.
- Stakeholder demands evidence in a regulated context:
  - Keep the required scripts, but support them with logs, charters, and minimal protocol language instead of procedural theater.
- Testing GenAI or LLM features:
  - Treat the model as a flaky platform, not an app.
  - Measure variation first. One clean run proves almost nothing.
