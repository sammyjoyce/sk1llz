---
name: dean-large-scale-systems
description: "Jeff Dean / Google-scale architecture heuristics for services whose real constraints are fanout, tail latency, hot partitions, shared-cluster scheduling, and multi-region mutation cost. Use when choosing sharding, replication, overload behavior, or control-plane/data-plane boundaries for systems that may span dozens of services or thousands of machines. Trigger words: tail latency, hedged requests, tied requests, stragglers, hot shard, micro-partitions, good-enough results, canary requests, commit wait, TrueTime, quota, shared cell, load shedding."
tags: systems, scale, dean, reliability, distributed systems, google scale, fault tolerance, latency, spanner, bigtable, mapreduce, borg, infrastructure, architecture, capacity planning
---

# Dean Large-Scale Systems

Use this as a design-review mindset, not as a generic distributed-systems explainer.

Loading rules:
- MANDATORY: read `references.md` before writing a cited design memo, paper-backed recommendation, or study plan around MapReduce, Bigtable, Spanner, Borg, or tail-latency work.
- Do NOT load `philosophy.md` for architecture tradeoffs, incident triage, or rollout decisions; it is inspiration, not decision support.

## Start Here

Before you write code or a long design doc, jot down 3-5 rough paragraphs and ask:
- What is the first acceptable failure mode: stale data, partial result, dropped optional feature, or explicit error?
- What is the true fanout of one user action? If it can touch 50 services or 1000 leaves, this is already a tail-latency problem.
- What numbers make or break the design? Same-DC RTT is about 0.5 ms, a disk seek is about 10 ms, reading 1 MB from disk is about 30 ms, and an intercontinental packet round-trip is about 150 ms. If the proposal ignores those orders of magnitude, it is fiction.
- Are you designing for 10-20x growth or 100x growth? Dean's rule is to work well at 10-20x, but keep interfaces reimplementable because the solution for X is often wrong at 100x.
- If this is "platform" work, are you the first demanding user? Dean's advice is to use your own infrastructure early; otherwise you optimize imagined needs.

## Failure Envelope First

- Treat partial infrastructure loss as steady-state, not DR theater. Dean's "typical first year" cluster included roughly one PDU failure that dropped 500-1000 machines for about 6 hours, around 20 rack failures, around 1000 machine failures, and thousands of disk failures.
- If the service contract cannot survive an entire rack disappearing, a few racks showing about 50% packet loss, or routing work that causes minutes of bad connectivity, the design is under-specified.
- Prefer failure containment over component heroics. At this scale, you win by making failures local, observable, and cheap to route around.

## Tail Rules That Actually Change Designs

- In high fanout trees, the last few leaves dominate the user experience. In "The Tail at Scale," a single leaf had 10 ms p99, but waiting for 100% of leaves pushed service latency to 140 ms p99; waiting for 95% cut that to 70 ms.
- Hedge only after evidence, not immediately. Sending the second request only after the 95th percentile expected latency keeps extra load near 5%; in a Bigtable benchmark, hedging after 10 ms cut 99.9th percentile from 1800 ms to 74 ms with only 2% more requests.
- Use tied requests when queueing dominates. A 1 ms delayed tied request with cross-server cancellation cut Bigtable read 99.9th latency by nearly 40% with less than 1% extra disk load.
- Do not probe all backends and then "pick the least loaded." Dean explicitly calls out why this is weaker: queue observations go stale, service time is hard to predict, and clients herd onto the same server.
- Hedging is not magic. It works when latency pathologies are mostly uncorrelated across replicas; if every replica shares the same saturated dependency, duplicate requests just multiply pain.
- Keep low-level queues short. Put priority policy in your own queue, not deep inside the OS or storage subsystem, or interactive work will sit behind old batch work.
- Counterintuitive but real: sometimes background work should be synchronized across machines. A short, aligned burst hurts a few requests; unsynchronized dribble keeps the entire tail bad all the time.
- Another counterintuitive rule: under heavy load, removing a persistently slow backend can improve latency even though total serving capacity drops. Put bad backends on probation, keep shadow traffic for measurement, and re-admit only when their latency distribution recovers.

## Partitioning, Skew, And Mix Shifts

- Static one-shard-per-machine layouts fail twice: hardware is not uniform, and popularity is not stationary.
- Favor micro-partitions. Around 20 partitions per machine lets the system rebalance load in 5% steps and recover faster because many machines can each absorb one small unit of failed work. Bigtable commonly ran 20-1000 tablets per machine for this reason.
- Selective replication beats blind repartitioning for hot items. Google search replicated important or popular documents and even changed language-biased replication across the day because workload mix shifted with geography.
- Test abrupt mix changes, not just steady-state averages. An outage in one region can suddenly redirect language- or market-specific traffic into another region and invalidate your previous "balanced" partitioning.
- For IR-like systems, define "good enough" semantics on purpose. Google often returns slightly incomplete results rather than waiting for the final leaf; if optional systems like ads or spelling correction are late, skip them.
- Use canary requests when a single pathological request could fan out to thousands of workers. Send the query to 1-2 leaves first, then fan out only if the canary behaves.

## Consistency And Mutation Heuristics

- Multi-datacenter systems should assume disconnection and partitioned operation are relatively common. Dean's bias for user-facing mutable products is often eventual consistency, because "we have your data but can't show it because one replica is unavailable" is a product failure.
- If you need external consistency, budget for the real cost of time uncertainty and write fanout. Spanner shows two-phase commit stays reasonable to about 50 participants, but mean and 99th percentile rise noticeably at 100 participants.
- More replicas are not a free safety knob. Spanner's read-only throughput improves from 3 to 5 replicas, but per-write work rises linearly and can outweigh the gain. Add replicas for a measured read-path need, not as ritual.
- Keep global transactions narrow. If correctness requires every workflow step inside one distributed transaction, you probably modeled ownership too broadly. Re-partition first; weaken guarantees second.
- Stateful mutations are often easier to move off the critical path than people assume. Respond once the durable minimum is safe, then finish secondary updates asynchronously with explicit repair or reconciliation semantics.

## Shared-Platform And Scheduler Heuristics

- Fine-grained resource requests matter. Borg found that rounding CPU and memory requests into power-of-two buckets would cost roughly 30-50% more machines in the median case.
- Dedicated clusters are not a free performance win. Borg measured shared-cell CPU performance only about 3% worse on average than dedicated cells, while smaller cells increased fragmentation and machine count materially.
- Requested resources are political, not factual. Users overbuy quota and over-request RAM because under-requesting gets them killed or throttled.
- Separate admission limit from runtime reservation. A practical pattern is: admit against the declared limit, then after startup transients decay reservation toward observed usage plus safety margin. Borg waited about 300 seconds before decaying reservations.
- Never let latency-sensitive production work rely on reclaimed capacity. Reclaimed resources are for batch and best-effort work that can be throttled or killed first.

## Anti-Patterns

- NEVER make one "universal" infrastructure abstraction because satisfying the seventh and eighth client demand usually explodes complexity and compromises everybody else. Instead solve the common needs well and keep the interface reimplementable.
- NEVER fix fanout latency by probing all backends and picking the "least loaded" one because that snapshot is already stale and clients stampede the same host. Instead use randomized placement plus hedged or tied requests with cancellation.
- NEVER add a second request immediately because it feels like the fastest way to crush p99, but it doubles work exactly when queues are healthy. Instead hedge after the observed 95th percentile and mark backup work lower priority.
- NEVER rely on a static shard map because it looks clean on diagrams and fails under thermal throttling, noisy neighbors, and hot keys. Instead use micro-partitions plus selective replication for predicted hotspots.
- NEVER put every cross-entity mutation into one global transaction because "correct by default" hides the cost of commit-wait, lock hold time, and participant explosion. Instead tighten data ownership until most transactions are local and move cross-domain work into explicit async flows.
- NEVER treat background jobs as harmless because a constant drizzle of compaction, GC, or scanning permanently fattens the tail. Instead throttle them, break them into smaller units, and sometimes synchronize them into brief windows.
- NEVER assume a slow server should stay in rotation because removing capacity feels unsafe. Instead probation the bad backend, keep shadow requests for measurement, and reintroduce it only after the latency distribution normalizes.
- NEVER split into dedicated clusters just because shared environments feel "messy," because the seductive isolation story hides fragmentation costs that can dwarf the few percent of interference you were trying to avoid. Instead keep a shared cell with explicit priority, quota, and reclamation rules until measurements prove isolation is cheaper.

## Decision Tree

- If the request path is read-heavy, fanout-heavy, and tolerant of approximation:
  - Use canary requests to 1-2 leaves first.
  - Define a "good enough" cutoff before waiting for 100% of leaves.
  - Hedge only the tail, and probation bad backends.
- If the request path is read-heavy but exact:
  - Keep fanout shallow, replicate read targets, and use tied requests only when failures are mostly uncorrelated.
  - If correlated slowness dominates, fix the shared dependency instead of spraying replicas.
- If the workload is skewed or flash-crowd prone:
  - Prefer micro-partitions and selective replication.
  - Test with abrupt language, region, or popularity shifts, not just even-key benchmarks.
- If the workload is write-heavy and cross-region:
  - Count participants and replicas first.
  - Above roughly 50 participants, expect noticeable tail pain; at 100, redesign ownership before tuning.
  - If the product can survive stale or partially applied secondary views, use async propagation and reconciliation.
- If the platform is multi-tenant:
  - Keep fine-grained quotas, reclaim from observed usage, and make batch the first victim.
  - If production needs dedicated capacity to stay alive, the scheduler policy is wrong or the SLO class is underspecified.

## Incident Triage

Before changing the architecture, ask:
- Is this a bad-tail problem, a hot-partition problem, or a hard dependency problem?
- Did one pathological request hit thousands of workers? If yes, use canary fanout and request fingerprinting.
- Did a handful of machines poison the percentile? If yes, probation them before adding capacity.
- Did the queue get deep enough that old work is now worthless? If yes, shorten low-level queues and let optional work die sooner.
- Can users accept degraded truth? If yes, drop optional branches before you drop the core result.

## Observability Minimum

- Export health plus enough live metrics to answer "why is it slow?" without redeploying.
- Capture all error RPCs and sample slow requests at concrete thresholds such as 50 ms, 100 ms, 500 ms, and 1 s, not just averages.
- If you cannot tell whether the pain is CPU, lock contention, packet loss, hot key, or queueing, the design is not production-ready.
