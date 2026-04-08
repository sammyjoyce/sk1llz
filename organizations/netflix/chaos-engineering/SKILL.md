---
name: netflix-chaos-engineering
description: Design and review Netflix-style production chaos experiments for distributed systems. Use when scoping FIT or ChAP style failure injection, choosing steady-state and abort metrics, prioritizing dependency experiments, or diagnosing noisy results from canary versus control chaos runs. Triggers: chaos engineering, FIT, ChAP, failure injection, steady state, blast radius, canary, fallback, retry storm, Hystrix, resilience.
tags: chaos-engineering, resilience, failure-injection, distributed-systems, reliability, netflix
---

# Netflix Chaos Engineering

## Load Discipline

- Before explaining fundamentals or onboarding someone new, READ `references/principles.md`. It is the canonical intro, not the operator playbook.
- Do NOT load `references/principles.md` for experiment design, triage, or review work. It is intentionally generic.
- Do NOT load `scripts/chaos_experiment.py` unless the task is to scaffold demo code. It is intentionally generic and lowers signal for experiment design, review, or triage.

## Operator Mindset

- Treat chaos work as customer-outcome verification, not infrastructure vandalism.
- Before choosing a fault, ask yourself:
  - What user-visible outcome must remain true? Netflix optimized for successful stream starts, not green host metrics.
  - Where is the real redundancy? If you cannot name the fallback, cache, retry budget, or alternate path, you are not ready to inject.
  - What would make this experiment lie to me? Common answers are stale telemetry, noisy error counters, rare sub-populations, or a fault model that does not match the real incident.
- Prefer faults that exercise timeout, retry, fallback, and queueing interactions. Random instance termination is easy to demo and often low-yield once autoscaling is mature.
- Start from a successful outcome and reason backward: "why did the good outcome happen, and what minimal set of failures would have prevented it?" This LDFI-style search finds deeper bugs than random failure picking.

## When This Pattern Fits

- Use request-scoped failure injection when you need precision over which users, requests, tenants, or dependencies are affected.
- Use control-versus-treatment routing when ambient production drift would swamp the signal. Netflix ran baseline and canary cohorts because raw production moves too much to compare against yesterday.
- Use dependency experiments before region drills when you have not yet verified fallbacks, timeout alignment, or cache behavior on critical edges.
- Skip production chaos during active failover, major traffic anomalies, or incident response. Those conditions invalidate the safety assumptions behind your routing and blast-radius budgets.

## Experiment Design Procedure

1. Define one customer KPI and one local explanatory metric.
   - Customer KPI should be a completed interaction, not an RPC status code. HTTP `200` can still map to a visibly broken experience.
   - Local explanatory metrics should tell you why the KPI moved: fallback rate, thread rejection, queue depth, saturation, tail latency.
2. Build a matched control group.
   - Route a statistically similar baseline cohort through baseline infrastructure with no injected fault.
   - Size treatment and control to the traffic slice they will receive. Netflix's published example used 1 percent treatment plus 1 percent control, with a 180-node service getting two-node baseline and canary clusters.
3. Choose the fault surface that matches the real incident class.
   - A thrown exception does not test service discovery loss, empty endpoint sets, stale-cache dependency, or slow downstream saturation.
   - Model at least one slow case and one broken case. The slow-but-not-yet-timed-out case is where queueing bugs often hide.
4. Bound blast radius on three axes at once.
   - User scope: start with a tiny cohort such as 1 percent treatment and 1 percent control.
   - Service scope: inject on one caller-dependency edge, not an entire stack.
   - Regional scope: cap all concurrent chaos traffic. Netflix capped aggregate concurrent experiments at 5 percent per region.
5. Require fast abort telemetry.
   - If the decisive KPI arrives five minutes late, it is not an abort signal.
   - Use seconds-latency counters for stop decisions and slower statistical analysis for post-run judgment.
6. Predeclare what counts as "interesting."
   - Fallback invoked unexpectedly.
   - Thread-pool rejection or queue growth.
   - Missing telemetry from the treatment cohort.
   - Divergence isolated to a device, tenant, or API slice.

## Safety Envelope And Fallbacks

- Run staffed-hours automation only. Netflix limited automated experiments to weekdays from 9 AM to 5 PM so a human could intervene quickly.
- If you do not have seconds-latency customer telemetry, do not run unattended production chaos. Either add the fast signal first or keep the experiment fully supervised and tiny.
- If you cannot build a matched control group, shrink scope or stop at supervised exploratory work. Do not claim statistical confidence from a one-armed experiment.
- If the target cohort is too rare to measure cleanly, lengthen observation time and stratify metrics before increasing blast radius. Oversampling a rare device class is sometimes necessary, but it is a business decision, not a quiet tuning knob.
- Do not promote a noisy blip into a "bug found." Netflix's LDFI workflow only called an experiment a real finding when the intended fault actually fired and more than 75 percent of impacted requests failed. Anything less needs investigation, not triumph.

## Fault Selection Heuristics

- For an RPC dependency, prefer three experiment types:
  - Failure: hard exception or explicit dependency failure.
  - Near-timeout latency: `highest timeout - recent P99 latency + 5% buffer`.
  - Timeout-breaking latency: `highest timeout + 5% buffer`.
- Prioritize dependencies in roughly this order:
  - Wrapped call paths with declared fallbacks first. Verify the safety net before broader drills.
  - Dependencies touched by a large fraction of inbound requests.
  - Dependencies with retries, because retries amplify customer impact and can create retry storms.
  - Dependencies with many interactions, because they propagate mistakes farther.
- Downgrade or blacklist experiments when:
  - dependency metadata is stale,
  - the call is unwrapped,
  - the dependency has a known direct impact on a hard KPI such as login, signup, order completion, or stream start,
  - the experiment would test latency on a path that already needs timeout tuning,
  - a failover or evacuation is in progress.

## Decision Tree

- If the question is "can we survive one host dying?":
  - Only run instance-kill chaos after dependency fallbacks and autoscaling behavior are already proven. Otherwise you are measuring fleet elasticity, not resilience logic.
- If the question is "why did the fallback fail in prod?":
  - Run request-scoped failure and near-timeout latency on the exact caller-dependency edge.
- If the question is "what should we automate next?":
  - Automate the experiments with the clearest safety case and lowest false-positive rate first. Automation that cries wolf destroys adoption.
- If the question is "why is the signal noisy?":
  - Check rare device populations, repeated-error outliers, and missing telemetry before loosening thresholds or increasing blast radius.

## Anti-Patterns

- NEVER start with Chaos Monkey style random instance kills because they are seductive, easy to explain, and usually exercise autoscaling instead of the dependency logic that causes real customer pain. Instead target the caller-dependency edge where timeouts, retries, and fallbacks interact.
- NEVER judge success from server response codes because they look objective while hiding user-visible failures behind `200 OK`. Instead use customer-outcome metrics such as completed playback, successful login, or checkout completion.
- NEVER use raw error counts as your primary stop condition because one pathological client or device can dominate the signal and force false aborts. Instead gate on completed-interaction deltas and use error counts as supporting evidence.
- NEVER treat missing treatment telemetry as "no issue observed" because the injected fault may have broken the reporting path itself. Instead treat absent device or client metrics as a suspected failure until disproven.
- NEVER replay exact production requests to compare hypotheses because state drift, independent deploys, and non-idempotent flows make replay lie to you. Instead group requests into stable equivalence classes and compare behavior across similar real traffic.
- NEVER automate experiment generation without a human review path because a noisy false-positive stream will make service teams ignore the program. Instead tune heuristics on reviewed findings before widening automation.
- NEVER model only one failure mode for a dependency because a clean result on returned errors says nothing about zero-instance discovery failures, slow drains, or partial registration loss. Instead test the failure shapes that actually occurred, or are operationally plausible, on that edge.

## Edge Cases Practitioners Miss

- Small cohorts hide device-specific breakage. A rare Smart TV class can be fully broken while aggregate success barely moves. Slice by device, region, tenant, or API shape before declaring success.
- Latency just below timeout can be more dangerous than latency above timeout. It can saturate queues and threads without ever tripping the fallback you expected to validate.
- A successful fallback can still reveal design debt. If a fallback silently degrades a supposedly non-critical path yet meaningfully harms the customer KPI, the dependency is more business-critical than the owner believed.
- Different failure models produce different truths. A dependency returning errors, a dependency with zero endpoints registered, and a dependency responding slowly are not interchangeable experiments.
- If the outcome metric is noisy, widen observation time before widening blast radius.
- If you see thread-pool rejection during latency injection, suspect timeout misalignment before blaming capacity. Netflix reported cases where injected latency around 900 ms exposed timeouts that were too high relative to pool size, causing rejection storms and fallback short-circuiting.

## Expected Output

- Name the user-visible steady state.
- Name the exact dependency edge or scope being tested.
- State the smallest safe treatment and control cohorts.
- State abort signals, observation signals, and why they are trustworthy.
- State which failure models are covered and which are not.
- State the next higher-risk experiment only if the current one passes cleanly.
