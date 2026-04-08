---
name: bach-exploratory-testing
description: Test software in the style of James Bach and Michael Bolton's Rapid Software Testing (RST) methodology. Use when designing a test strategy, writing or reviewing charters, running exploratory/chartered sessions, debriefing testers, critiquing test plans that over-rely on test cases or automated "tests," deciding whether a failing check is actually a bug, or evaluating whether a tester's report should be trusted. Also use when a stakeholder asks "how much testing is enough?", when SBTM is being introduced, or when testing LLMs/AI systems. Triggers include: exploratory testing, SBTM, session-based test management, RST, test charter, test oracle, FEW HICCUPPS, HTSM, SFDPOT, CRUSSPIC STMPL, "too many test cases", "tests pass but bugs slip through", "automation isn't catching bugs", debriefing a tester, context-driven testing, Bach, Bolton.
---

# Bach-style Rapid Software Testing

## First, drop three beliefs you almost certainly hold

If you don't internalise these, nothing else in this skill will land.

1. **There is no such thing as an "automated test."** Tools do *checking* — the mechanical verification of propositions about the product. Testing is investigation. A Playwright suite, a unit test file, a CI job — all checks. Call them checks. Saying "automated test" in a Bach-style conversation marks you as a novice and leaks into how people measure, staff, and plan testing.

2. **"Exploratory testing" is not a technique.** It is the default posture of any responsible tester. Since Bach & Bolton's *Exploratory Testing 3.0* (2015) the adjective exists only to contrast with *scripted* — and the distinguishing axis is **tester agency**, not activity. If someone asks "do you use exploratory testing or scripted testing?", the honest answer is "both, simultaneously, in every session"; the real question is how much freedom the tester has from moment to moment.

3. **A test case is not a unit of measurement.** Never count them. The same testing activity can be expressed as 1 case or 1,000,000 cases — counting it rewards splitting over thinking. "We have 3,400 regression tests" tells a sophisticated reader nothing except that the speaker doesn't know how to describe coverage. Reply with Bach's three coverage questions (below), not a number.

## The Three Coverage Questions (use these in every testing conversation)

Before claiming you tested something, or accepting someone else's claim:

1. **What did you look at?** (product factors, data, configurations — *coverage*)
2. **How would you recognise a problem if you saw one?** (oracles)
3. **What did you actually do?** (procedures, techniques)

If the speaker can't answer all three crisply, they performed *activity*, not testing. This is the fastest diagnostic for a hand-wavy QA plan or a half-baked test report.

## Oracles: how you recognise a problem you didn't predict

An oracle is a **heuristic**. It can tell you there *might be* a problem, but it can *never* tell you there is *no* problem. That asymmetry is the whole game. Anyone who says "all tests passed, the product works" has forgotten it.

Use **A FEW HICCUPPS** (Bach & Bolton's current list — the original 2005 HICCUPPS has been updated; do not teach the old one):

- **A**cceptability — is it *good enough*, not just "not wrong"?
- **F**amiliarity — inconsistent with a pattern of familiar bugs?
- **E**xplainability — can you describe its behaviour without hedging?
- **W**orld — consistent with what you know about reality?
- **H**istory — consistent with past versions?
- **I**mage — consistent with the org's desired reputation?
- **C**omparable products — consistent with similar systems?
- **C**laims — consistent with docs, specs, marketing?
- **U**sers' desires — consistent with what reasonable users want?
- **P**roduct — internally self-consistent?
- **P**urpose — consistent with apparent and implied uses?
- **S**tatutes — consistent with laws, regulations, standards?

**Key move:** when something "feels off" but you can't explain why, walk the list aloud until a word makes you say *"this one."* That word is the framing for your bug report. Reports without an oracle citation get dismissed; reports with one become very hard to argue with.

For the full oracle catalogue, Doug Hoffman's complementary list, SFDPOT product elements, CRUSSPIC STMPL quality criteria, and the Honest Manual Writer heuristic, **READ `references/oracles-and-coverage.md`.**

## Writing a charter (the craft beginners get wrong)

Format: *Explore **[target]** with **[resources]** to discover **[information]**.*

Hidden skill: **charter sizing**.

- **Too big** ("Explore checkout to find bugs") → no testing story, nothing to debrief, nothing to finish.
- **Too small** ("Verify the Save button stores the record") → it's a *check* in disguise. Automate it and move on.
- **Right-sized:** one focused tester reaches a satisfying stopping point in **60–120 minutes** and produces a reviewable session sheet. If it can't, split it.

**Before writing a charter, ask yourself:** *"What information am I buying, and which stakeholder will act on it?"* If you cannot name a decision the result will inform, the charter is vanity. Kill it and rewrite.

For sizing worked examples, risk-to-charter translation, and the top six charter anti-patterns, **READ `references/charters.md`.**

## Running and debriefing sessions (the SBTM parts people skip)

- **Session length.** Short = 60m, normal = 90m, long = 120m (±15m). The hard ceiling is **2–4 sessions/tester/day** before cognitive fatigue destroys signal. A schedule of five back-to-back normals is a schedule of cargo-cult SBTM.
- **Note in real time.** If you reconstruct the session sheet from memory afterwards, you are writing a *memoir*, not a test report. You will unconsciously smooth over dead-ends — which is exactly where the next bug lives.
- **Track T / B / S** inside on-charter time as rough percentages:
  - **T** – test design & execution (produces coverage)
  - **B** – bug investigation & reporting (interrupts coverage)
  - **S** – setup & configuration (interrupts coverage)
  A session reported at 10/80/10 means you found a bug and chased it — that may have been correct, but the manager now knows this charter is **not done**.
- **Opportunity time (O)** is relevant work *outside* the charter's mission. Record it separately. It often produces the best bugs, but do not pretend it was the charter.

**Debrief with PROOF** (Jon Bach) — a 5–10 minute *conversation*, not a status meeting:

- **P**ast — what did you actually do?
- **R**esults — what happened? (bugs, surprises, questions)
- **O**bstacles — what got in the way? (testability, environment, missing info)
- **O**utlook — what should happen next?
- **F**eelings — how does the tester *feel* about the product and the session?

**The most-dropped letter is F, and dropping it is the single biggest mistake in SBTM.** "I'm uneasy about the payment flow" from an experienced tester is worth more than forty green checks. If a debrief consistently runs longer than 15 minutes, the *session sheet* is broken — fix the sheet, not the debrief.

For the exact session-sheet schema, Jon Bach's PROOF definition, T/B/S/O math, and parser-friendly tagging, **READ `references/sbtm-session-sheet.md`.**

## Anti-patterns (with the part that makes each one seductive)

- **NEVER count test cases as a measure of progress or coverage.** Seductive because the number always goes up and managers love charts. Consequence: the team optimises for *splitting*, not *thinking*; a single activity gets re-tagged into 500 rows and nobody learns anything. Instead, report coverage as answers to the three coverage questions and as risk areas addressed.

- **NEVER call a suite of automated checks a "test suite" or its runs "test runs."** Seductive because the whole industry does it. Consequence: managers hear "tests pass" and conclude testing is done; real testing gets defunded. Instead, put "automated checks" and "exploratory testing" on separate lines in every plan and report.

- **NEVER write session notes after the session from memory.** Seductive because note-taking feels like it interrupts flow. Consequence: you lose the one detail that would have reproduced the bug, and your sheet reads like a victory lap. Instead, take ugly real-time notes (bullets, timestamps, screenshots), then spend the last five minutes of the session tidying them — inside the time-box.

- **NEVER turn a charter into a step-by-step script.** Seductive because it makes the session look "accountable" to auditors and junior testers feel safer. Consequence: you've built a check paid for at a tester's salary, and you've thrown away the tester's judgement — the only thing automation can't do. Instead, leave the *how* to the tester and hold them accountable on the *information* they produced.

- **NEVER report "passed" or "works as expected."** Seductive because it's the language managers want to hear. Consequence: it is literally false — you only know you did not *see* a problem in the specific actions you took. Instead, use **safety language**: *"So far I haven't found a problem in X under conditions Y"* / *"No failure observed in the scenarios I ran."* This sounds pedantic until the first postmortem, when it becomes the only sentence anyone trusts.

- **NEVER trust your best technique on familiar code.** Seductive because it worked before. Consequence: the *pesticide paradox* — tests become immune to new bugs; you swear an area is clean while users find bugs daily. Instead, **defocus**: deliberately vary technique, data class, starting state, platform, even tester. If boundary testing found the last five bugs, the next one almost certainly isn't a boundary bug.

- **NEVER accept "shallow agreement"** on loaded words (*quality*, *done*, *coverage*, *tested*, *automated*, *AI*). Seductive because the meeting ends faster. Consequence: three people leave with three different plans and blame each other in two weeks. Instead, ask each person for a concrete example of what they mean. Takes ninety seconds; saves a sprint.

- **NEVER delegate oracle judgement to an LLM.** Bach calls this "Automated Irresponsibility." Seductive because a one-shot demo looks magical. Consequence: LLMs are non-deterministic retrievers — run the same prompt 25× and watch the output drift. Instead, when using AI in testing, run every judgement ≥10×, measure the variance as your *first* signal, and treat any single-shot LLM output as a lead, never as an oracle.

- **NEVER use SBTM metrics (session counts, T/B/S ratios) to grade or rank individual testers.** Seductive because it feels like "accountability." Consequence: testers learn to game the numbers — they stop reporting bugs during B-heavy sessions, inflate S on hard days, and never admit a session was wasted. You destroy the honest reporting that made SBTM valuable in the first place. Instead, use the numbers to generate *questions* in debriefs, never conclusions.

## Decision tree: "we have a result — what now?"

```
A check failed
├─ Is the check's proposition still valid about the current product?
│  ├─ No → the check is stale; fix or retire it (it was a proposition, not testing)
│  └─ Yes → is the product wrong, or was the proposition wrong?
│     ├─ Product wrong → investigate with oracles, file bug with A FEW HICCUPPS framing
│     └─ Proposition wrong → this is a testing-the-tests moment; capture the learning
│
All checks passed
├─ What did the checks NOT look at? Answer the three coverage questions for the gap.
├─ Which A FEW HICCUPPS oracles have you not consulted in the last two weeks?
├─ Charter one exploratory session against the riskiest gap.
└─ Expect that session to find a bug. If it doesn't, defocus and run another.

Stakeholder asks "is it ready to ship?"
└─ Never answer yes/no. Answer with:
    (1) coverage so far, (2) notable risks remaining, (3) obstacles encountered,
    (4) what you'd test next given one more day.
   The ship decision is theirs; your job is to inform it, credibly.

A whole testing area has "gone quiet" (no new bugs for weeks)
└─ Do NOT conclude it's stable. Apply the pesticide paradox:
    change techniques, data, state, platform, or tester. Treat silence as a smell.
```

## When to deviate from this skill

- **Regulated / safety-critical domains** (FDA Class III, DO-178C, IEC 62304): the auditor demands traceable scripts. Do the scripted work *in addition to* session-based testing, never instead of it. Bach's two-paragraph protocol for a Class 3 medical device (which replaced 50 pages of procedural "test cases") is the model — scripted parts can be tiny.
- **Pure algorithmic code with a cheap reference oracle** (compilers, parsers, numeric libraries, consensus protocols): lean heavily on property-based and high-volume automated checking. Responsible testing still matters, but the economics tilt hard toward machine checks.
- **You are embedded in a shallow-agreement culture you cannot fix politically**: translate internally. Write "test case" in Jira; think "charter + check" in your head. This is survival, not surrender.
- **You are testing GenAI / LLMs**: every principle here applies *more strongly*, not less. The product is non-deterministic, so no single-run observation is ever an oracle — measure variance across runs first.

## Further reading inside this skill

- `references/oracles-and-coverage.md` — full FEW HICCUPPS definitions, Hoffman's complementary oracles, SFDPOT, CRUSSPIC STMPL, the Honest Manual Writer heuristic, the Blink Test, and the Generic Test Procedure. **Load before writing a test strategy, reviewing a bug report framing, or building an oracle checklist.**
- `references/charters.md` — charter sizing worked examples, risk-to-charter translation, six charter anti-patterns with fixes, and tour heuristics (Money, FedEx, Landmark, Anti-tour). **Load when writing charters or reviewing someone else's charter library.**
- `references/sbtm-session-sheet.md` — exact session-sheet schema, parser tags, T/B/S/O math, PROOF debrief script with sample dialogue, and rules for when to run paired sessions. **Load when introducing SBTM, running your first debrief, or auditing an existing SBTM implementation.**

**Do NOT load the references for quick questions already answered in this file (e.g. "what does FEW HICCUPPS stand for", "what's a charter's format"). Only load them when you need depth, exact schemas, or worked examples.**
