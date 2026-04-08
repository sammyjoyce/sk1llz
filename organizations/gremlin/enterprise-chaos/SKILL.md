---
name: gremlin-enterprise-chaos
description: Design and review Gremlin chaos experiments, Scenarios, Reliability Tests, and Failure Flags for enterprise systems. Use when choosing between latency, packet loss, blackhole, DNS, shutdown, time travel, certificate expiry, or application-layer failure injection; when defining Health Checks and halt logic; when growing blast radius safely; or when interpreting reliability score changes and GameDay outcomes. Triggers: gremlin, chaos engineering, scenario, health check, reliability test, reliability score, failure flags, gameday, latency, packet loss, blackhole, dns, time travel, certificate expiry.
tags: chaos-engineering, gremlin, reliability, fault-injection, failure-flags, gameday, resiliency, health-checks
---

# Gremlin Enterprise Chaos

This skill is for high-signal experiment design and review, not for explaining chaos engineering basics.

## Load Only What You Need

- Before writing exact experiment syntax or YAML, READ `references/attack_catalog.md`.
- Before AWS ASG, Lambda, or Failure Flags rollout work, verify the live Gremlin docs for that feature; those surfaces change faster than this skill.
- Do NOT load `references/attack_catalog.md` when the task is only "which failure mode should I test?" It is a syntax aid, not a strategy guide.

## Operating Stance

- Treat every experiment as a test of a specific resiliency mechanism. If you cannot name the mechanism, you are not ready to run the experiment.
- Test the caller's ability to survive a dependency failure, not the dependency owner's reliability. For dependency experiments, target the service making the call.
- Prefer one clean failure signature over a "realistic" bundle of failures. Diagnosis quality matters more than theatrical realism on early runs.
- Count any experiment that requires manual repair after halt as a failure, even if the user-facing path recovered during the blast.
- Read score changes together with coverage. A higher reliability score with fewer weekly runs is often a blind spot, not an improvement.

## Before Choosing an Experiment, Ask Yourself

- What exact user-visible symptom am I trying to prove safe: slower success, partial correctness, total dependency loss, bad name resolution, state restart, clock drift, or bad application data?
- Which mechanism is supposed to catch it: timeout, retry with jitter, circuit breaker, cache, load balancer drain, autoscaler, quorum, restart controller, TTL handling, certificate rotation, or feature-flag isolation?
- Am I testing steady-state degradation or recovery after the fault is removed? Gremlin can show the first while still exposing that recovery is broken.
- Will the platform "help" me and hide the real behavior? Autoscaling, Kubernetes reaping, sidecar proxying, and local caches all distort interpretation if not accounted for up front.

## Experiment Selection Heuristics

### Dependency path

- Use `latency` when you want to surface timeout budgets, serial fan-out, queue growth, and connection-pool behavior. Gremlin latency is per packet, not per request; 100 ms can multiply across round trips and serial calls, and even 20 ms has produced multi-x request inflation in real systems.
- Use `packet loss` when you care about retransmits, retry storms, idempotency, and noisy-network behavior. Gremlin packet loss is outbound-only; if you need to test a hard disconnect, do not simulate it with 100% loss.
- Use `blackhole` for a true partition or complete dependency cut. It drops inbound and outbound L4 traffic, but leaves DNS alone by default, so it is the wrong tool for resolver failures.
- Use `dns` when the failure mode is lookup failure, stale resolution assumptions, or overly optimistic DNS caching. Teams often think a blackhole covers DNS; in Gremlin it does not.

### Time and expiry path

- Use `time travel` for JWT expiry, lease timeouts, scheduled jobs, and clock-skew logic. Decide deliberately whether to disable NTP; if you do not, time correction can erase the fault before the code path is exercised.
- Use certificate-expiry testing when you need to validate the whole certificate chain from a specific caller's vantage point. Gremlin's default window is 720 hours (30 days), which is good for operations hygiene but often too coarse for application-path debugging.

### State and restart path

- Use `shutdown` for host or node loss, but remember the semantics change by target type. On hosts it is graceful enough to deliver `SIGTERM`; on containers it behaves more like immediate process death.
- Use process-level attacks when you want supervision and restart-controller behavior without conflating it with node lifecycle or platform replacement.

### Application-layer path

- Use Failure Flags by proxy first when the problem is request/response shaping in managed or serverless environments and you want minimal code change. The proxy path automatically creates flags like `dependency-<hostname>`, `ingress`, `egress`, and `response`.
- Switch to Failure Flags SDK instrumentation only when the failure is inside application logic or data shape: corrupt payloads, double delivery, hot-row lock contention, customer-specific failures, or ordering bugs that network faults cannot express.

## Blast Radius and Halt Design

- Start with one stable target set and one dependency selector. In Scenarios, Gremlin requires tag-style targeting; exact host picks are intentionally unavailable there.
- Build reusable Scenarios from stable identities only. Ephemeral selectors like hostnames, instance IDs, local IPs, and recycled pod identifiers make Scenarios rot and can block service creation.
- Use existing alerts, SLIs, or monitors as Health Checks whenever possible. A bespoke "chaos passed" check is almost always less strict than the alarm your on-call team actually trusts.
- Continuous Health Checks in Scenarios run every 10 seconds. Thresholds tighter than one poll interval tend to reward scrape timing, not resilience.
- If you configure a Scenario to treat fired monitors as a valid outcome, remember that Gremlin still halts the run when a Health Check triggers. "Pass" does not mean "keep going."
- In AWS, disable ASG autoscaling before blackhole or zone-redundancy work when you want to test fault tolerance rather than replacement logic. If rollback fails, inspect which ASG processes Gremlin suspended and resume those exact ones.
- Do not stack concurrent network experiments on the same network interface or the same pod. Gremlin does not support that cleanly; split targets or test the failure modes sequentially.

## Reading the Result Correctly

- A "DNS outage" on one node is often not DNS. Gremlin has seen single-node dependency failures caused by local runtime state after reboot, while the rest of the fleet was healthy. Re-check whether the blast is local before blaming the provider.
- If aborting a blackhole restores traffic but one service still needs a restart, the experiment exposed broken recovery, not a clean pass.
- If latency experiments explode far beyond the configured milliseconds, suspect repeated round trips, connection churn, or serial dependency calls before assuming the tool over-injected.
- If packet-loss results look mild, verify whether the code path is dominated by inbound traffic or buffered asynchronous work; Gremlin packet loss primarily disturbs outbound traffic.
- If a Time Travel run appears to do nothing, the first suspects are NTP correction, clock reads delegated to another service, or a TTL path using monotonic time instead of wall clock.

## Numbers That Matter

- `latency` defaults to 100 ms for 60 s; this is a product default, not an experimental recommendation.
- `packet loss` defaults to 1% outbound loss; that is enough to reveal brittle retry logic without making everything look broken.
- `time travel` defaults to an offset of 86,400 s for 60 s. That default is huge; narrow the offset to one expiry boundary unless you explicitly want system-wide time havoc.
- Recommended Scenarios support up to 99 nodes, which means large trees are possible; resist the temptation unless you can explain each branch's diagnostic value.
- Reliability reporting treats scores above 70 as healthy, 50-70 as caution, and below 50 as poor. Use those bands for portfolio triage, not for declaring a single service "done."
- Mature Gremlin programs run standard reliability tests weekly per service. If the cadence drops, your confidence should drop with it.

## Anti-Patterns

- NEVER start from product defaults because they are safe generic values, not workload-aware values. This is seductive because it feels objective. The consequence is a test that proves nothing about your actual timeout ladder or queue depth. Instead anchor magnitude to one SLO boundary, one retry interval, or one expiry threshold.
- NEVER use 100% packet loss to represent a total outage because it feels like the "maximum" version of the same test. The consequence is a noisy mix of retransmits and retry behavior that is different from a true partition. Instead switch to `blackhole` once you mean "no traffic gets through."
- NEVER blackhole all outbound traffic just because targeting `egress` is faster than finding the real dependency. The consequence is an attribution mess where telemetry, side effects, and unrelated dependencies fail together. Instead target one hostname, port, provider, or dependency flag.
- NEVER trust a custom green-only Health Check because it is easier to make pass than the monitors your operators already use. The consequence is false confidence and surprise incidents. Instead bind Gremlin to the same alerting surfaces that would page humans.
- NEVER forget autoscaler and scheduler behavior because infrastructure help is seductive to count as resilience. The consequence is that you test replacement speed instead of failover logic. Instead freeze or account for the helper before you run the blast.
- NEVER assume `shutdown + reboot` on a container proves restart safety because the option name sounds symmetric across platforms. The consequence is that Kubernetes may reap the container and never return the exact target you think you rebooted. Instead test host or node shutdown for reboot semantics, or use process-level attacks for restart policy validation.
- NEVER run Time Travel without deciding whether NTP should be blocked because leaving defaults alone feels safer. The consequence is a false negative when the clock snaps back before the application path observes drift. Instead choose the NTP posture deliberately and keep the window tightly bounded.
- NEVER keep exact or ephemeral target identifiers in reusable Scenarios because they work perfectly during authoring. The consequence is silent scenario decay as pods and instances churn. Instead build around stable service tags, namespaces, deployments, or Failure Flags selectors.

## Fallbacks

- If infrastructure targeting is impossible because the platform is managed, move to Failure Flags and keep the infrastructure hypothesis separate from the application hypothesis.
- If the result is ambiguous, shrink the blast radius before changing the fault type. A smaller clean signal is more valuable than a broader noisy one.
- If you cannot explain the result from first principles, rerun with one variable removed: one dependency, one target, one check, one recovery mechanism.
- If the Reliability Report shows a dip with flat weekly coverage, prioritize regression analysis. If it shows a rise with falling coverage, prioritize restoring test cadence.

## Exit Criteria

- The experiment maps to one named resiliency mechanism.
- The target set is stable and intentionally scoped.
- Halt logic uses real operational signals.
- The chosen attack matches the failure signature you care about.
- You know how you will interpret manual recovery, partial recovery, and false-negative cases before you run anything.
