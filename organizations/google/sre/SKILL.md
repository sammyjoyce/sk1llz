---
name: google-sre
description: >-
  Apply Google's Site Reliability Engineering methodology with expert-level
  precision. Covers SLI selection traps, dependent-service SLO ceilings,
  multi-window multi-burn-rate alerting (14.4×/6×/1× with derivation),
  error budget policy enforcement, Chubby-style planned outages, postmortem
  action-item anti-patterns (Ben Treynor's P0/P1 rule), toil ROI economics,
  and on-call sustainability thresholds. Use when designing SLIs/SLOs,
  setting up burn-rate alerts, writing an error budget policy, conducting
  postmortems, measuring toil, sizing on-call rotations, or deciding whether
  to freeze deploys. Triggers: "SRE", "SLI", "SLO", "error budget",
  "burn rate", "toil", "postmortem", "on-call", "reliability",
  "incident response", "service level", "production readiness",
  "freeze deploys", "pager load".
---

# Google SRE — Expert Decision Guide

## Thinking Frames (ask before acting)

- **Who is the user, and what is the critical user journey?** SLOs measure *journeys*, not system health. If your SLI doesn't correlate with support tickets and churn, you're measuring the wrong thing.
- **What does the error budget say right now?** Healthy → ship. Exhausted by our code → freeze. Exhausted upstream → document, don't freeze. Never negotiate the policy *during* an outage.
- **Is this work toil or engineering?** If it scales linearly with service size and produces no compounding value, it's toil. Cap at 50%.
- **Would a page at 3 AM require human intelligence to resolve?** If no, it is automation or a ticket, never a page.
- **Does this postmortem have a P0/P1 action item?** Per Ben Treynor: "A postmortem without subsequent action is indistinguishable from no postmortem."
- **Before defending this SLO in a planning meeting, can I actually win?** If not, it's decorative — delete it and propose a version you can defend.
- **Before assigning this action item, would the receiving team prioritize it *without* a postmortem forcing the conversation?** If no, the fix is wrong or the owner is wrong — rework it before publishing.

## SLI Selection — Expert Traps

Every SLI is a ratio `good_events / valid_events`. Both numerator and denominator must be inspectable during an incident, or you cannot debug a burn.

**Measurement point is a deliberate choice, not a default.** Server-side success rate misses DNS, TLS, CDN, and everything past the load balancer. Load-balancer metrics miss client-side JavaScript errors. Client-side RUM captures user reality but inherits the client's own reliability and deployment lag. Rule: move the SLI closer to the user only once cheaper signals are exhausted — every hop closer raises cost and complexity.

**Precision vs recall is a real tradeoff you must measure.** Precision = fraction of SLI-captured events that were actually user-visible. Recall = fraction of real user-visible incidents the SLI captured. Improving one usually degrades the other. Replay the last quarter's incidents against your SLI: low recall → tighten SLO; low precision → loosen it or refine the SLI implementation.

**Latency needs *paired* thresholds, not a single percentile.** `99% < 300ms` lets the tail hide — a bug that makes 0.5% of requests take 30s looks fine. Use `90% < 100ms AND 99% < 400ms`. The second line catches the slow-tail class of bugs the first will always miss.

NEVER use mean latency — a 5% tail 50× slower than average hides behind a healthy mean. Use distribution-based ratio SLIs computed as counts, never as a single aggregate number. Instead: `count(requests < 300ms) / count(all)`.

NEVER set SLOs from engineering aspirations. Measure 2–4 weeks of actual performance, then set the SLO slightly below observed. A 99.99% target on a service delivering 99.95% leaves you with *negative* error budget on day one.

NEVER use a **calendar-aligned** SLO window (e.g., "99.9% per calendar month"). Teams will rush risky launches on day 1 of the month knowing the budget resets. Use a **rolling 4-week or 30-day window** so today's deploy is always measured against the last 30 days — there is no reset button to game.

**Low-traffic trap**: a service at 10 req/hour burns 13.9% of its 30-day budget on a single failed request (a 1,000× burn rate). Below ~100 req/hour: generate synthetic probes, aggregate with sibling services into a shared SLO, or widen the window to 90 days — don't just tune alerts.

## SLO Targets — Dependency Math and the Chubby Lesson

**The dependency ceiling.** With N hard dependencies each at SLO *s*, your theoretical max is `s^N`. Five dependencies at 99.9% cap you at 99.5%. Teams routinely set SLOs above their ceiling and then burn budget for reasons they cannot fix. Before committing to a target, compute `product(dep_slos)` — that is your real upper bound. If it's too low, reduce hard dependencies or accept a lower SLO.

**Internal SLO < SLA, always.** Your internal SLO must be tighter than your external SLA by a deliberate margin (e.g., SLA 99.9%, internal 99.95%). The buffer is what absorbs a bad deploy without triggering customer refunds. Publishing your internal number is a footgun — users will rebuild against it.

**Chubby's planned-outage pattern.** If a service consistently overperforms its SLO, users build hidden hard dependencies on the higher number. Google's Chubby lock service deliberately takes planned downtime each quarter to flush out these illegal dependencies before they calcify. If you're massively exceeding your SLO *and* cannot raise it, inject controlled unavailability to reset expectations. Counterintuitive but load-bearing.

**Exclude out-of-scope traffic from SLO math**: load tests, pentests, deprecated API versions, clients that went out-of-quota due to their own bugs. Count them and you'll burn budget on problems with no user impact and fight the wrong fires.

**The "defend the SLO" test** (from the SRE book): if you cannot ever win a priority conversation by quoting a particular SLO, that SLO is not worth having — delete it. SLOs you can't point to during planning are reporting metrics in disguise.

**Canary comparisons are relative, not absolute.** A canary error rate of 0.5% may be fine or catastrophic depending on baseline. The correct rollback trigger is `canary_error_rate > baseline_error_rate * 1.1` (10% worse than baseline), not an absolute threshold. Absolute thresholds rollback innocent canaries during real outages and miss regressions that are "within budget but worse than before".

**Progressive disclosure**:
- Before writing an SLO document, computing budget translation tables, or designing a new policy tier, **READ `references/error_budget.md`** for templates and time tables.
- Before setting up numerical burn-rate monitoring against a historical dataset, **RUN `scripts/error_budget_calculator.py`** against your actual request counts — don't eyeball it.
- Do NOT load `references/error_budget.md` for freeze/don't-freeze decisions — use the decision tree below in this file. Loading it wastes context on templates you don't need.

## Multi-Window Multi-Burn-Rate Alerting

The only alerting pattern that is simultaneously sensitive to real incidents and deaf to flaps.

**Where does 14.4 come from?** The policy is "2% of a 30-day budget burned in 1 hour". Burn rate = budget_consumed / window_fraction = 0.02 / (1/720) = 14.4. The number is derived from the policy; you do not pick it separately.

| Tier | Policy | Long window | Short window (1/12) | Burn rate | Action |
|---|---|---|---|---|---|
| 1 | 2% in 1h | 1h | 5m | 14.4× | Page immediately |
| 2 | 5% in 6h | 6h | 30m | 6× | Page |
| 3 | 10% in 3d | 3d | 2h | 1× | Ticket |

Both windows must exceed threshold simultaneously. The short window is the "is it still burning *right now*?" confirmation — it eliminates stale alerts post-recovery and prevents flap intervals from being treated as an incident.

NEVER add a Prometheus `for:` duration clause to a burn-rate alert. A spike every 10 minutes that clears between spikes resets the timer — you can lose 35% of monthly budget and never fire. The long/short window combination already handles duration.

NEVER write a burn-rate query as a plain division — zero traffic yields NaN and silence during a full outage. Guard the denominator: `sum(rate(errors[1h])) / (sum(rate(total[1h])) > 0)`.

NEVER tune burn-rate thresholds per microservice. Three tiers (CRITICAL / HIGH / LOW) and assign each service to a tier. Per-service tuning creates O(n) cognitive load and on-callers stop trusting the thresholds.

**Recalculate 14.4 when SLO < 99%.** Max possible burn rate is `1/(1−SLO)`. A 90% SLO caps max burn rate at 10 — the 14.4× page threshold is mathematically impossible to hit, so you'll never page. For SLOs under 99%, use ~0.5× of max as the page threshold.

## Error Budget Policy — Enforcement, Not Theater

An unwritten policy is no policy. Pre-approve with VP/Director authority **before** any incident. Minimum contents:

- **Internal-cause freeze**: SLO missed for trailing 4-week window from our code/config → halt all non-P0, non-security changes until back in SLO.
- **External-cause exemption**: Company-wide network failures, upstream team outages, and out-of-scope load tests do NOT trigger a freeze. Document; do not punish the wrong team.
- **Single-incident rule** (from Google's Appendix B policy): one incident consuming >20% of quarterly budget mandates a postmortem with at least one P0 action item.
- **Class-of-outage rule**: one class of outage consuming >20% over a quarter requires a P0 item on the following quarter's planning doc.
- **Escalation clause**: Disagreements on budget calculation escalate to the named executive (usually CTO). Name them in advance.

NEVER grant ad-hoc exceptions to the freeze. Within two quarters, nobody takes the budget seriously. Define a "silver bullet" quota (max 2 per quarter, written approval required) for genuinely critical launches.

NEVER wait until the budget is exhausted to act. Set an orange zone at 25% remaining that requires extra deploy review and longer canaries — by the time you hit 0%, customer-visible damage is done.

## Postmortem Quality — Where Value Lives or Dies

**The Treynor rule**: every postmortem following a user-affecting outage carries at least one P0 or P1 bug. No exceptions without VP approval, and Google's VP of 24/7 Ops personally reviews them.

**Classify every action item** as: Investigate → Mitigate → Repair → Detect → Prevent. A plan containing only Prevent items missed faster wins. A plan containing only Mitigate items hasn't fixed the root cause.

**Action item anti-patterns**:

| Anti-pattern | Why it's seductive | Consequence | Correct alternative |
|---|---|---|---|
| "Train humans not to run unsafe commands" | Feels responsible | Per Dan Milstein: plan for a future where we're all as stupid as today. Humans will repeat the mistake. | Make the unsafe command impossible. Add idempotency, confirmation prompts, blast-radius preview. |
| "Investigate monitoring for this scenario" | Sounds diligent | Can't be marked done. Rots in tracker. | "Alert when endpoint X error rate >1%, owned by @name, bug BUG-1234, due YYYY-MM-DD." |
| Alert on the exact failing query | Directly addresses *this* incident | Only catches identical recurrence, misses the class | Alert on user-facing symptoms (error rate, latency) at the SLO level, not implementation details |
| No named owner ("the team will handle it") | Feels inclusive | Never prioritized by anyone | One named human, assigned at creation, not later |
| Tossing work over the wall | "Other team owns this area" | Receiving team has no context, deprioritizes | Co-author with the owning team, get buy-in *before* publishing the postmortem |
| Unbalanced plan (all strategic items) | Feels ambitious | Nothing ships before the next outage | Include near-term mitigations alongside strategic fixes |
| Finger-pointing in "Things that went poorly" | Feels like accountability | Erodes psychological safety; next incident will be hidden | Blameless language: "the automation allowed X" not "Alice did X" |

**An unreviewed postmortem might as well not exist.** Schedule regular review sessions; close out comments; link every action item to a tracker bug. An AI without a bug is not an AI.

**Signs the postmortem is bad**: animated language ("ridiculous", "!!!"), missing Recovery Efforts section, no numbers in impact assessment, a root cause of "human error" (always ask *why* the human was allowed to cause it), or single-category action items.

## Toil — 50% Cap and Automation ROI

Google caps SRE toil at 50% of team time. Sustained >50% for two consecutive quarters → SRE formally hands the service back to the dev team. That handback threat is the enforcement lever; without it, 50% is aspirational.

**Ticket toil is the insidious kind**: it accomplishes its goal (users get what they wanted), disperses evenly across the team, never loudly demands remediation. Audit quarterly by sampling and categorizing.

**Automation ROI**:
```
ROI_weeks = automation_effort_hours / weekly_toil_hours_saved
```
If ROI > 26 weeks, the toil may die naturally (reorg, retirement, platform migration) before automation pays off. Prioritize items with ROI < 8 weeks. Include hidden benefits in the spreadsheet: fewer human errors, less context switching, better morale, shorter onboarding — real but rarely counted.

NEVER accept "it's a one-time migration" as not-toil. At scale, migrations meet every toil criterion: manual, repetitive, no lasting value, scales with inventory. Track them.

NEVER script around a kernel/library bug when you can patch it (John Looney's Google lesson: 1,000 machines with full disks from a patched-driver log bug — masking the symptom removed incentive to fix the cause, and cost $1,000/hr across the fleet until the kernel was fixed).

## On-Call Sustainability — Concrete Thresholds

- **Min team size**: 8 for a weekly rotation; 6 absolute floor. A team of 5 rotates every 5 days and burns out within one quarter regardless of incident volume.
- **Max pages**: 2 per 12-hour shift on average across a quarter. Exceed → do NOT add people (masks the problem). Fix alerts (non-actionable pages are bugs in the alert) and the top 3 sources of pages.
- **Shift length**: ≤12h with ≥12h rest between. Back-to-back is a postmortem waiting to happen.
- **Every page requires human intelligence.** Robotic responses (restart, clear queue, bounce) must be automated before they can become a page.

NEVER solve "too many pages" by adding headcount — you'll burn out more people at the same rate. Fix the alerts first.

## Decision Tree: Should We Freeze Deploys?

```
Error budget exhausted?
├── YES → Cause internal (our code/config)?
│   ├── YES → FREEZE. Only P0 + security fixes.
│   │         Single incident >20% budget → mandate P0 postmortem action item.
│   │         Single class >20% of quarter → P0 item on next quarter's plan.
│   └── NO (external infra, upstream team outage, out-of-scope load) →
│       Document exception. Do NOT freeze (punishes wrong team).
│       Increment silver-bullet counter — max 2/quarter.
└── NO → Budget remaining > 25%?
    ├── YES → Ship freely. Run chaos experiments.
    │         Consistently overperforming → consider Chubby-style planned outage.
    └── NO (orange zone, 10–25%) →
        Slow velocity. Extra review on risky changes.
        Lengthen canary duration. No experiments.
        Extrapolate current burn → predict freeze date → notify product owner now.
```

## Fallback Strategies (when the primary approach fails)

- **SLI data missing or unreliable**: Start with black-box HTTP probes (cheap, low-fidelity). Add load-balancer metrics next. Move to client-side RUM only once server-side is proven insufficient.
- **Stakeholders can't agree on a target**: Skip the debate. Measure 4 weeks, present the data, propose "observed minus one standard deviation" as the starting SLO. Iterate quarterly.
- **Pager load spiking with no clear cause**: Don't tune alerts yet. Read the last 10 pages verbatim and classify (real / flaky / dependency / user error). Fix the biggest category first.
- **Freeze triggered and product is furious**: Re-read the policy *text*. If it allows this release under a documented condition, spend a silver-bullet and log the decision. If not, the policy is working — escalate per the escalation clause; do not override.
- **Team demands per-service burn-rate thresholds**: Refuse; offer a new tier instead. If three tiers can't express the need, the real fix is an SLI change, not more tiers.
- **Postmortem AI rotted for a quarter, outage recurred**: Don't write a new AI for the same root cause. Find the original AI, promote it to P0, reassign to a senior engineer with schedule authority, and add the rotted AI itself as a datapoint in the new postmortem ("process failure: AI-1234 unowned for 90 days").
- **New service has no SLO and launch is Friday**: Ship with a provisional 99% SLO and a 14-day review. Lower is better than missing — you can always tighten once you have data, but a service without any SLO has no protection at all.

## NEVER List (summary)

- NEVER set SLO at 100% — mathematically impossible, infinite cost
- NEVER set SLO above your dependency ceiling `product(dep_slos)`
- NEVER publish your internal SLO — publish a looser SLA number
- NEVER use a calendar-aligned SLO window; use a rolling 4-week/30-day window
- NEVER use mean latency as an SLI; use ratios with paired thresholds
- NEVER add `for:` duration to burn-rate alerts; use multi-window instead
- NEVER write a burn-rate query without a denominator-guard against NaN
- NEVER tune burn-rate thresholds per service; use tiered buckets
- NEVER negotiate error budget policy during an outage
- NEVER leave a postmortem without a P0/P1 action item (Treynor rule)
- NEVER accept "train humans to be more careful" as a preventive AI
- NEVER count out-of-scope traffic against the SLO (load tests, buggy quota-breachers)
- NEVER page for events that don't require human intelligence
- NEVER solve excess pager load by adding people
- NEVER accept toil >50% for >2 quarters without escalating to hand back the service
- NEVER skip Chubby-style planned outages when massively overperforming — hidden dependencies are worse than the outage
