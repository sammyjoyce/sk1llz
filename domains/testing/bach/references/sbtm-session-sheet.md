# SBTM Session Sheets and Debriefs — Deep Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​‌​‌​‌‌‍‌‌‌​​‌‌‌‍‌‌​​​‌‌​‍​​​​​‌​‌‍​​​​‌​‌​‍​‌‌‌​‌​‌⁠‍⁠

Load this when introducing SBTM to a team, running your first debrief, or auditing an existing SBTM implementation.

## The canonical session sheet format

SBTM session sheets are designed to be **machine-parseable** so that coverage and metrics can be aggregated across many sessions without hand-typing data into spreadsheets. The original parser tags are ALL-CAPS keywords followed by content.

```
CHARTER
  Explore the payment-retry and webhook-replay paths with simulated network
  drops to discover idempotency and ledger-consistency issues.

#AREAS
  OS: macOS 14.4
  Browser: Firefox 125
  Strategy: simulation via mitmproxy
  Feature: Checkout / Payment Retry
  Feature: Webhook Ingestion

#START
  2025-06-12 09:30

TESTER
  L. Chen

#DURATION
  normal

#SESSION CHARTER  85
#SESSION OPPORTUNITY  15

#CHARTER VS OPPORTUNITY
  On charter: 85%
  Opportunity: 15%

#ON CHARTER BREAKDOWN
  T (test design/execution): 55%
  B (bug investigation/reporting): 35%
  S (setup): 10%

#DATA FILES
  mitm-replay-001.log
  stripe-test-tokens.txt
  screenshots/retry-loop-*.png

#TEST NOTES
  - Started from clean cart, tok_visa, intentional 500 on /charge endpoint
  - Retry kicked in after 2s; noted exponential backoff
  - After 4th retry, duplicate ledger row appeared in admin (see #BUG 1)
  - Webhook replay from Stripe CLI produced different result on 3rd attempt (#BUG 2)
  - "Feeling: something about the idempotency key derivation bothers me"
  - Switched to tok_mastercard — same behavior observed
  - Dove into console logs — see opportunity notes

#BUGS
  BUG 1: Duplicate ledger entry on 4th retry. Repro in data files.
         Oracle: Product (internal inconsistency with ledger contract)
  BUG 2: Webhook replay non-idempotent across identical X-Idempotency-Key.
         Oracle: Claims (docs promise idempotency for 24h window)

#ISSUES
  - Admin panel has no way to view raw ledger rows without DB access
  - Stripe test-mode webhooks lag 3–8s — hard to reproduce timing issues
  - No dev has ownership of the webhook-replay code path (unclear code-owner)

#QUESTIONS
  - Is the idempotency key meant to include retry count or not?
  - What is the ledger's reconciliation window?
```

### Why the format matters

- **Tags are stable** (`CHARTER`, `#BUGS`, `#ISSUES`, `#DURATION`) — a shell script, regex, or parser can aggregate hundreds of sheets without the tester typing into a bug tracker.
- **Percentages are rough.** Nobody stopwatches T/B/S to the minute. Round to the nearest 5%. Excessive precision is a smell — it means the tester is performing accountability theatre instead of testing.
- **Test notes are real-time.** They look like stream of consciousness because they *are*. Do not polish them after the fact; polishing is memoir.
- **`#ISSUES` is separate from `#BUGS`.** A bug threatens the value of the *product*. An issue threatens the value of the *testing process* (missing info, bad tools, broken test data). Conflating them is why testability never gets prioritised.
- **`#QUESTIONS` is where stakeholder debts accumulate.** Every unanswered question is a place you are testing on assumptions. Review them in the debrief.

## T / B / S / O metric math

All percentages are within *on-charter time*:

| Metric | What it means | Pathologies |
|---|---|---|
| **T** — test design & execution | Actually producing coverage | 100% T with no bugs = either pristine code or the tester is blind; always probe |
| **B** — bug investigation & reporting | Necessary interruption to T | Sustained >50% B means the tester is triaging for the team; they aren't *testing*, they're *diagnosing* |
| **S** — setup | Preparing environment, data, tools | Sustained >20% S is a testability red flag — raise as an issue, not a tester failure |
| **O** — opportunity (outside on-charter) | Relevant work the charter didn't cover | Consistent high O means the charter library is out-of-date with where bugs actually live |

**Use these as question generators, never as scores.** A session at T=5% is not a "bad" session if the tester filed a critical bug at 10% B and found a testability issue at 85% S. A session at T=95% may be useless if the tester wasn't looking at risk areas.

## Session length conventions

| Label | Duration | When to use |
|---|---|---|
| Short | 60m ±15m | Tight context, familiar territory, or tester is new |
| Normal | 90m ±15m | Default. Use until you have a reason not to. |
| Long | 120m ±15m | Deep dive into an unfamiliar area, or stateful scenarios that need warm-up |

**Hard limit: 2–4 sessions per tester per day.** After 4 sessions of real cognitive work, signal quality collapses. Teams that schedule 6–8 sessions/day are producing session sheets, not testing.

## PROOF debrief — the exact protocol

A debrief is a **conversation**, not a status meeting. Typical length 5–10 minutes. Run by the test lead (or another tester if no lead is available — *never* by the project manager, who will turn it into a status meeting in under 30 seconds).

Run the mnemonic in order — do not skip ahead.

### P — Past

*"Walk me through what you actually did."*

Listen for the narrative arc. If the tester can't tell a coherent story, the session sheet was written badly — or they started testing something else halfway through and didn't update the charter.

### R — Results

*"What did you find?"*

Cover bugs, but also: surprises (things that happened you didn't expect, good or bad), questions raised, assumptions that turned out wrong. Note each with an oracle tag (A FEW HICCUPPS) for the report.

### O — Obstacles

*"What got in the way?"*

This is where `#ISSUES` come in. Broken test data, missing docs, slow environments, unresponsive devs, unclear requirements. These are investments the team must make if they want faster future sessions.

### O — Outlook

*"What should happen next?"*

Should this charter be re-run? Should the bugs be chartered for follow-up investigation? Should an adjacent area be chartered? Should we escalate a testability issue?

### F — Feelings

*"How do you feel about the product right now? About this area specifically?"*

**Do not skip this.** Feelings are tacit knowledge leaking into language. An experienced tester saying *"I'm uneasy"* or *"something about this feels rushed"* is surfacing pattern recognition they can't yet articulate. Bach-style testers treat these as high-priority leading indicators. Chart a follow-up session on whatever produced the feeling.

Junior testers may resist this question — they think feelings are unprofessional. Normalise it with phrasing like *"What's your gut say?"* or *"If you had to bet money on where the next bug lives, where?"*

## Debrief anti-patterns

- **Debrief as status report.** Testers lining up to "present" to the manager. Kills honest reporting instantly. Fix: 1-on-1 conversations, no PowerPoint, no audience.
- **Debrief-by-Slack-message.** Acceptable for mature senior testers who write tight sheets. Fatal for juniors — they lose the coaching and their sheets never improve. Fix: verbal for anyone with <1 year at Bach-style SBTM.
- **Debrief used as performance review data.** The moment testers suspect their numbers will be used against them, T/B/S ratios become fiction. Fix: publicly commit that metrics are question-generators, never grades. Repeat often.
- **Debriefs consistently >15 minutes.** The session sheet is doing a bad job. Fix the template or the tester's note-taking habit, not the debrief.
- **No debrief at all.** The information in the session sheet evaporates in a week and bugs get rediscovered. The investment in SBTM produces roughly nothing without debriefs.

## Paired sessions

Two testers, one machine. Roles: **Driver** (operates the product, narrates aloud) and **Navigator** (watches, questions, takes notes, suggests ideas, keeps the charter honest). Swap roles every 20–30 minutes.

**When to pair:**
- Onboarding a new tester onto an unfamiliar area.
- A senior tester mentoring a junior (navigator is the senior).
- Two specialists from different domains (e.g. security + accessibility) testing the same feature.
- When you genuinely don't know where to start and need two perspectives to form a model.

**When NOT to pair:**
- Routine charters. Pairing is expensive — save it for high-value learning moments.
- Performance testing or timing-sensitive work — the navigator's interruptions break the measurement.

A paired session produces *one* session sheet with both names in the `TESTER` field.

## Round-robin debriefing (when the lead is unavailable)

If the test lead is swamped, use the round-robin:

- Charter 1: Tester A runs the session, Tester B debriefs Tester A, Tester C inherits the sheet for future regression design.
- Charter 2: Tester B runs, Tester C debriefs, Tester A inherits.
- Charter 3: Tester C runs, Tester A debriefs, Tester B inherits.

Every tester sees every sheet from a different angle (executor, reviewer, consumer). Knowledge distributes automatically. Use this especially on teams of 3–5 where the lead is also a practitioner.

## Introducing SBTM without destroying morale

If SBTM is new, do **not** drop the full protocol on day one. Bach's own advice:

1. **Week 1–2:** start with charters and real-time notes only. Ignore T/B/S. Ignore parser tags. The goal is to build the habit of *mission-driven* testing and *real-time* reporting.
2. **Week 3–4:** introduce debriefs (PROOF). Keep them short. Praise testers who raise `#ISSUES` — this sets the culture that testability is a first-class concern.
3. **Month 2:** introduce T/B/S *as questions the tester asks themselves*, not as numbers the manager tracks.
4. **Month 3:** introduce parser tags only if you actually plan to parse them. If you don't, don't bother — the overhead is pure ceremony.

Teams that adopt all of this in week 1 produce beautifully formatted sheets that contain nothing real. Teams that adopt it incrementally produce sheets that are occasionally messy but always honest. Pick honest.
