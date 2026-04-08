---
name: google-sre
description: >-
  Apply Google's Site Reliability Engineering decision heuristics for SLO
  design, burn-rate alerting, error-budget policy, on-call sustainability, and
  postmortem quality. Use when defining or revising SLIs/SLOs, deciding whether
  to freeze releases, tuning burn-rate alerts, diagnosing operational overload,
  reviewing pager design, or rejecting hero-based reliability practices.
  Triggers: "SRE", "SLI", "SLO", "error budget", "burn rate", "toil",
  "on-call", "pager", "postmortem", "reliability", "incident",
  "availability", "latency", "freeze releases", "heroism".
---

# Google SRE - Decision Guide⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​​​‌​​‍‌​‌‌‌‌‌‌‍‌​​​​​‌​‍​‌​‌​‌‌​‍​​​​‌​​‌‍‌​‌‌​​‌‌⁠‍⁠

## Start Here

Before changing anything, ask:

- Which user objective am I defending? If you cannot name the user step, you are about to optimize infrastructure vanity metrics.
- Is the problem in measurement, policy, alerting, or system design? Treating all four as "monitoring work" is how teams accumulate dashboards instead of reliability.
- Will this change reduce future cognitive load at 3 AM, or just move pain between teams?
- Am I about to rely on heroics? Any plan that works only because someone watches graphs, hand-holds launches, or remembers tribal exceptions is already admitting the system is broken.

Choose the path:

- Need outage math, downtime tables, or budget translation? READ `references/error_budget.md` first.
- Need to compute or verify a real service's budget or exhaustion time? RUN `scripts/error_budget_calculator.py` with the actual SLO and window.
- Need to decide whether to page, freeze, loosen or tighten an SLO, or reject a postmortem action item? Stay in this file.
- Do NOT load `references/error_budget.md` for incident triage, on-call staffing, or postmortem review. It is math depth, not operational judgment.
- Do NOT run `scripts/error_budget_calculator.py` for policy design, postmortem quality review, or pager-sizing work. It checks arithmetic, not whether the arithmetic is worth defending.

## 1. Pick the right SLI before arguing about the number

Google SRE guidance is to start with a ratio `good / valid`, then choose the cheapest measurement point that still correlates with user pain. Service-side signals are cheap, low-latency, and under your control, but they have narrow coverage: the server can say "success" while users see blank pages, stale data, or broken client logic. Client-side SLOs widen coverage, but they arrive batched, can lose data when apps close or networks flap, and often lag by 15 to 60 minutes. End-to-end SLOs have the highest confidence and the highest cost; Google notes they can take months of engineering and often run as batch joins across multiple data sources.

Before moving "closer to the user," ask whether the miss is a coverage problem or a threshold problem:

- Support tickets and manually detected outages not reflected in budget burn: recall is low. Move the measurement point outward or cover more user journeys.
- SLO misses with no user pain: precision is low. Relax the objective or narrow the denominator.
- Both bad: fix the SLI implementation instead of arguing over one more nine.

Use paired latency thresholds, not a single percentile. Google's example SLO documents use `90% < fast threshold` and `99% < slower threshold` because a single percentile lets slow-tail regressions hide behind an acceptable headline number.

For asynchronous or data products, availability is often the wrong abstraction. Google's example SLO documents use freshness, correctness, and completeness. A correctness prober with known-good inputs is often higher signal than pretending every enqueue success means the user outcome succeeded.

Treat user-objective annotation as a prioritization tool, not an all-or-nothing migration. Product-focused SRE guidance recommends instrumenting the most critical user steps first and propagating that annotation through dashboards, logs, and load-shedding logic so low-value traffic cannot drown out the business-critical path.

## 2. Set an SLO you can actually defend

Before proposing a number, ask three questions from Google's SLO approval model:

- Will product accept the user experience implied by this target?
- Will development actually slow feature work when the budget is gone?
- Can the defending team uphold it without burnout, Herculean effort, or permanent toil?

If any answer is "no," the number is decorative. Rework the SLI, target, or ownership before publishing.

Start cheap and iterate. Google explicitly allows using current performance as a starting point when you have no better evidence, but only if you document that fact and plan to revisit it. Do not let today's observed behavior silently harden into tomorrow's promise.

Prefer rolling windows over calendar windows. Reset-aligned SLOs invite risky launches right after the clock resets and hide whether today's change is borrowing against the same reliability promise users were relying on yesterday.

Do not forget dependency math. If the request path has multiple hard dependencies, your practical ceiling is the combined reliability of that path. If that ceiling is below the proposed SLO, reduce hard dependencies or lower the target before the team gets trapped defending an impossible commitment.

Use Google's decision matrix instead of metric worship:

- Missed SLO, users still happy, toil low: loosen the SLO.
- Met SLO, users unhappy: tighten the SLO.
- Met SLO, users happy, toil high: reduce false-positive alerting or temporarily loosen the SLO while you automate away the operational pain.

When a tighter target is clearly right for users but the system cannot meet it yet, create an aspirational SLO: measure it, review it, and explicitly exempt it from enforcement. Otherwise you put the team in permanent emergency mode and the policy stops meaning anything.

Remember that error budget is an approximation of user pain, not user pain itself. Four short outages, a constant low error rate, and one long outage can burn similar budget while feeling very different to users. Use budget to govern engineering attention, not as a claim that all failure shapes are equivalent.

## 3. Alert on burn rate, not raw error rate

Google's recommended starting point for a 99.9% class SLO is:

- Page at `14.4x` burn over `1h` and `5m` together. That means 2% of a 30-day budget in 1 hour.
- Page at `6x` burn over `6h` and `30m` together. That means 5% of budget in 6 hours.
- Ticket at `1x` burn over `3d` and `6h` together. That means 10% of budget in 3 days.

The short window should be about `1/12` of the long window. The long window answers "is this significant?"; the short window answers "is it still burning right now?" Without the short window, reset time stays bad long after recovery.

Low-traffic services are special. Google calls out the trap directly: at 10 requests per hour, one failure is a `1000x` burn rate for a 99.9% SLO and consumes about 13.9% of a 30-day budget. For low-QPS services, use this order of operations:

- generate synthetic traffic if it gives representative coverage,
- aggregate related small services that share a failure domain,
- reduce impact per failure via retries, backoff, or fallback paths,
- renegotiate the SLO only if each failed request is not valuable enough to justify a page.

Synthetic traffic has a non-obvious failure mode: if the probes do not exercise the broken path, successful probe traffic can hide the real user signal. Treat probe coverage as a production concern, not a monitoring afterthought.

Keep canaries isolated. Google's canary guidance warns that simultaneous canaries contaminate the baseline and increase mental load exactly when fast diagnosis matters.

Extreme SLOs need custom parameters. Google notes that a 90% availability goal can make the standard "2% of budget in one hour" page mathematically impossible even during a total outage. Re-derive the alert parameters instead of copy-pasting `14.4 / 6 / 1`.

## 4. Error budget policy is the real control surface

The policy is the product, not the dashboard. Google's example policy is explicit:

- above SLO: releases continue,
- exhausted over the trailing four-week window: halt all but P0 and security work,
- one incident above 20% of four-week budget: mandatory postmortem with at least one P0 action item,
- one outage class above 20% of quarterly budget: put a P0 item on next quarter's plan,
- disagreements escalate to a named executive.

Before deciding "freeze or not," classify cause first:

- Our code or process caused the miss: freeze and reallocate effort.
- A hard dependency failed and can be softened: treat that as our reliability work too.
- Company-wide network event, another team's outage already under its own freeze, or out-of-scope traffic consumed the budget: document the exception instead of punishing the wrong team.

Counterintuitive but important: Google also notes that freezing even on dependency-caused misses can make users happier by minimizing additional risk. Pick the rule deliberately and record it in the policy; do not improvise while people are already under incident pressure.

If a service is meeting SLO with low toil and high customer satisfaction, Google's decision matrix says you may either increase release velocity or step back and spend SRE effort on a more reliability-constrained system. Reliability work should follow marginal value, not tradition.

## 5. On-call design is mostly about preventing bad psychology

Google's sustainable on-call numbers are sharper than most teams expect:

- at least 50% of SRE time should remain engineering,
- no more than 25% should be on-call,
- with primary and secondary week-long rotations, that implies a minimum of 8 engineers for a single-site team,
- 6 per site is the reasonable floor for dual-site rotations,
- an incident averages about 6 hours of total work, so the cap is about 2 incidents per 12-hour shift.

Use these as design constraints, not after-the-fact health checks. If you need fewer people than this, shrink the supported scope or change service expectations; do not normalize overload.

Google's on-call chapter is unusually explicit about psychology: repeated pages from the same symptom create confirmation bias, and stress pushes people toward fast but unexamined action. When the incident is multi-team, ambiguous, or has no credible upper bound yet, switch to formal incident management early instead of letting the on-caller improvise under cortisol.

Operational overload is usually a monitoring or service-design problem before it is a staffing problem. Google recommends driving toward a `1:1 alert-to-incident` ratio; if one condition fans out into several pages, the monitoring system itself is generating toil.

If sustained overload cannot be fixed quickly, "give back the pager" is a valid temporary move. The point is not punishment; it is preventing the SRE team from living indefinitely above supportable load while the service remains below SRE standards.

## 6. Postmortems should change the system, not the mood

A good postmortem changes the system, not the narrative. Google's example policy requires at least one P0 action item for a sufficiently expensive incident. If the write-up has no named owner, no tracker, or no schedule authority behind the fix, it is theater.

Before accepting any action item, ask:

- Does this make the failure harder to repeat, or just document that it happened?
- Does it remove risky human judgment from the path, or merely tell humans to concentrate harder?
- Would the owning team prioritize it without the postmortem forcing the conversation?

Heroism is seductive because it is low risk, immediately rewarding, and hides the fact that the system is broken. Google's "Why Heroism Is Bad" guidance is direct: heroes get praise for hand-holding launches, manually rolling back bad canaries, or babysitting noisy systems, but the real consequence is that the team never has the realism conversation about the SLO, the process, or the architecture.

Spend error budget to expose broken systems when you need the signal. If the only reason an SLO appears healthy is that a human is quietly compensating for the system every day, the SLO is measuring hero availability, not service reliability.

## NEVER do these

- NEVER treat service-side success as user happiness because it is cheap and low-latency. It misses empty responses, client breakage, and asynchronous failures. Instead move outward only when ticket or incident data proves recall is insufficient.
- NEVER "fix" poor SLO coverage by only tightening the target because changing one number is easier than re-instrumenting. The consequence is endless false positives or false negatives. Instead decide whether the SLI, denominator, or measurement point is wrong.
- NEVER add a `for:` timer to burn-rate alerts because it looks like a cheap precision boost. Google shows 100% spikes every 10 minutes can consume 35% of budget and never alert. Instead use long and short windows together.
- NEVER copy `14.4 / 6 / 1` blindly because the numbers are memorable. They assume enough traffic and roughly 99.9-class goals. The consequence is impossible alerts on low-SLO services and useless noise on low-QPS systems. Instead re-derive parameters from budget share, traffic shape, and maximum possible burn.
- NEVER use calendar-aligned SLO windows because they feel operationally tidy and make reporting easy. The consequence is reset gaming: teams learn that day-one risk is cheaper than day-twenty-eight risk. Instead use rolling windows so every release is judged against the same recent history.
- NEVER negotiate an error-budget freeze during the outage because that is the most emotionally convenient moment to water the rule down. The consequence is a policy nobody believes. Instead pre-approve the rule and the escalation path.
- NEVER solve excess paging by adding more people because that hides broken alerts and broken services behind a larger blast radius. Instead fix alert fan-out, non-actionable pages, and the top recurring failure sources.
- NEVER rely on a hero to meet an SLO because the work feels urgent and praise arrives immediately. The consequence is masked system debt, burnout, and no pressure to repair the actual gap. Instead let the miss become visible and use the budget as the forcing function for structural fixes.
- NEVER accept "train people to be careful" as a preventive action because it sounds responsible and cheap. The consequence is recurrence under stress. Instead remove the dangerous edge with automation, guardrails, idempotency, confirmation, or rollback safety.
- NEVER keep supporting a service whose toil and on-call load stay above SRE limits quarter after quarter because the relationship feels easier than escalation. The consequence is permanent overload and stalled engineering. Instead renegotiate responsibilities or give back the pager until the service is supportable.

## Fallbacks

- No trustworthy historical data: start with black-box probes or load-balancer metrics, publish the caveat, and set a short review date.
- Support tickets and budget burn disagree: compare incident dates to ticket spikes and SLO dips; use that gap to decide whether to tighten, relax, or re-instrument.
- Stakeholders want an impossible target: bring dependency math and toil cost to the meeting instead of arguing from taste.
- Canary signal is noisy: Google recommends one canary at a time. Multiple simultaneous canaries contaminate the baseline and increase cognitive load exactly when fast reasoning matters.
- Low-QPS pages are useless noise: prefer synthetic traffic or grouping by shared failure domain before lowering the SLO. Lower the SLO only when the business truly accepts the extra failure impact.
