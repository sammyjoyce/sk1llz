---
name: vogels-cloud-architecture
description: "Turn vague distributed-systems plans into Vogels-style availability designs: statically stable, blast-radius-limited, explicit about consistency trade-offs, and safe under overload. Use when designing or reviewing AWS-scale APIs, control-plane/data-plane splits, multi-AZ or multi-region services, retry/idempotency policies, queue-based workflows, cells, or noisy-neighbor isolation. Triggers include Werner Vogels, Dynamo, AWS-scale, eventual consistency, static stability, shuffle sharding, blast radius, control plane, data plane, retry storm, overload, and noisy neighbors."
tags: vogels, aws, distributed-systems, reliability, eventual-consistency, static-stability, shuffle-sharding, control-plane, data-plane, cells
---

# Vogels Cloud Architecture⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌‌‌‌​‌​‍​‌​‌​​​‌‍​‌‌‌‌‌‌‌‍​‌‌​‌‌‌‌‍​​​​‌​​‌‍‌‌​‌​​​‌⁠‍⁠

Use this skill when availability, blast radius, and customer-visible behavior matter more than elegance on a diagram.

## Operating Stance

- Treat every dependency as guilty until proven statically safe. The real question is not "is this service multi-AZ?" but "can the request path keep working if that dependency stops changing right now?"
- Optimize for customer-visible continuity, not internal purity. Vogels-style systems often accept stale metadata, delayed propagation, or reduced features to preserve the action the customer is already taking.
- Prefer one stable operating mode. Recovery paths that launch new infrastructure, bypass caches, or activate rarely used branches are usually the start of a second outage.
- Isolate faults before optimizing throughput. Cells, zonal boundaries, and shuffle shards are not deployment details; they are the unit of survivable failure.
- Strong consistency is a budget, not a virtue. Spend it only on invariants whose violation costs money, legal exposure, or irreversible user harm.

## Before You Draw Boxes, Ask

Before choosing topology, ask yourself:

- Which operations must continue during a control-plane impairment, and which can pause?
- What is the smallest acceptable blast radius: an AZ, a cell, a tenant, or a single request key?
- If this dependency goes gray rather than black, will health checks catch it, or will the failure smear across zones?
- What inconsistency can the customer tolerate: stale global state, or never seeing their own write?
- If overload starts here, where does excess work go: shed, queue, delay, or poison every downstream hop?

Before adding a remote call to a critical path, ask yourself:

- Can the needed data be pushed ahead of time or refreshed in a constant-work loop instead?
- If the call times out, will retries help, or will they only demand more work from an already sick dependency?
- Is the call part of the data plane or only the control plane? If data plane, why is it not zone-local or cell-local?

Before choosing "eventual consistency", ask yourself:

- What is the actual inconsistency window under load and replica lag?
- Do users need global freshness, or just read-your-writes, monotonic reads, and session continuity?
- If a session breaks, what user-visible lie appears next: missing cart items, duplicate actions, or resurrected stale state?

## Choose The Mechanism

| Situation | Default move | Use this instead of | Watch for |
|---|---|---|---|
| Feature must survive dependency impairment with old-but-valid state | Static stability: keep serving with last known good state | Just-in-time reprovisioning or on-failure config fetches | Hidden startup dependencies like config download, credential fetch, service discovery |
| High-volume health/config propagation | Constant-work full-state loops | Edge-triggered workflows that scale with churn | Waste is acceptable only when the per-tick work is bounded and cheap |
| Noisy neighbors, abuse, or tenant-specific hot spots | Cells plus shuffle sharding | Global shared pools with quotas as the only guardrail | Pick a partition key that avoids cross-cell joins and per-request mapping lookups |
| Rare, transient remote failure | Single-layer retries with capped backoff and token-budgeting | Retries at every layer | Retry rate itself must be a first-class metric |
| Money, inventory, singleton creation, leadership | Localize strong consistency to that operation | System-wide strong consistency | Prefer conditional writes or idempotent create semantics before global transactions |
| Async workflow whose output expires | Sideline or discard stale work once it misses usefulness window | Infinite backlog "for durability" | Queue age matters more than queue depth |
| Critical data-plane dependency between services in one region | Keep foundational dependencies zonal if they sit in the packet/request path | "Regional service calling regional service" everywhere | Gray failures multiply across the path even when black failures fail away cleanly |

## Numbers That Should Change Your Design

- Timeout selection starts from tolerated false timeouts, not gut feel. Amazon commonly starts with something like 0.1 percent false timeouts, then uses the matching downstream percentile such as `p99.9`, plus explicit padding if `p99.9` sits too close to `p50`.
- Low timeouts often fail on connection establishment, not steady-state work. Amazon hit deployment-time failures at about `20 ms` because TLS setup was inside the timer; the fix was prewarming connections before taking traffic.
- Retry multiplication is brutal: a five-deep stack with three retries at each layer can magnify database load by `243x`.
- Three-AZ active-active designs need real slack. If any two AZs must carry full load, each AZ should run at about `66%` of tested capacity, which is `50%` overprovisioning.
- Regional-to-regional dependency chains quietly destroy zone independence. With one impaired AZ, two regional services in series avoid the bad zone with probability `4/9`; a zonal dependency keeps that probability at `2/3`, and it stays `2/3` no matter how many zonal hops you add.
- Regular sharding gives linear blast-radius reduction; shuffle sharding gives combinatorial reduction. With `8` workers and `2` workers per shard, regular sharding hurts `25%` of customers; shuffle sharding creates `28` possible shards, shrinking impact to about `1/28`.
- Route 53 pushed this to the extreme with `2048` virtual name servers and shards of `4`, yielding about `730 billion` possible shuffle shards and enough room to keep domains from sharing more than two servers.
- SQS FIFO has a sharp edge: only the most recent `20k` unprocessed messages are polled. Let a few message groups backlog badly enough and fresh groups can starve behind them.

## Decision Heuristics Experts Use

### Control Plane vs Data Plane

- If customers care more about "keep serving current traffic" than "accept new configuration", split the planes and make the data plane statically stable.
- If recovery requires creating instances, fetching config, discovering peers, or minting credentials after the failure starts, the design is not statically stable.
- For foundational data-plane services, zonal independence beats architectural neatness. It is worth extra NAT gateways, route tables, or per-AZ service slices to keep one zone's sickness inside that zone.

### Consistency

- Do not ask "eventual or strong?" Ask "which invariant is expensive enough to deserve unavailability?" Inventory allocation, payment capture, leader election, and singleton creation usually qualify.
- For user-facing state, session guarantees usually buy more than global linearizability. Read-your-writes plus monotonic reads remove most UX absurdities while keeping the system highly available.
- Idempotency is about intent, not deduping payloads. Reusing a caller-supplied request ID must return the same semantic result for the retry window; if the same ID arrives with different parameters, fail validation instead of guessing.

### Overload

- Queue depth lies; queue age tells the truth. Track age of first attempt or age of oldest useful work so you can see when durability has turned into useless debt.
- Retries are selfish. Budget them at one layer, meter them locally, and stop when they are no longer increasing successful completion.
- Jitter periodic work deterministically per host or key, not randomly each run. Stable jitter preserves repeatable patterns and makes race conditions diagnosable.

### Blast Radius

- A cell is only real if it can fail alone. Shared databases, synchronous auth lookups, shared feature-flag evaluators, or per-request routing maps can turn "cell architecture" back into one giant service.
- Pick partition keys that keep most reads and writes inside one cell. If your key requires frequent cross-cell fan-out, the mapping layer becomes the real monolith.
- When abuse or pathological tenants dominate failures, isolate the customer before isolating the code path. Shuffle sharding is often more effective than smarter rate limiting alone.

## NEVER Do This

- NEVER add a database-bypass or cache-bypass fallback because it feels safer than returning partial results. It is seductive because it "uses the source of truth," but when the cache fleet or fast path fails together, every caller stampedes the primary store and turns a partial feature outage into a full-site outage. Instead, degrade the feature, proactively push the needed data, or run both paths continuously as real failover.
- NEVER recover an AZ impairment by launching fresh capacity or rebuilding state on demand because cloud APIs appear infinitely elastic. It is seductive because autoscaling demos well, but the recovery path now depends on the control plane, config delivery, discovery, and credential systems precisely when they are least trustworthy. Instead, pre-provision the capacity and promote or shift to already-running standbys.
- NEVER let every layer own its own retries because local autonomy sounds robust. It is seductive because each team can "improve availability" independently, but retry trees multiply load and can pin a sick dependency under `243x` extra demand. Instead, retry at one layer, require idempotency for side effects, and meter retries with a local token budget.
- NEVER call a regional dependency from a zonal critical path because the dependency itself is "highly available." It is seductive because the interface is simpler, but gray failures make the probability of avoiding the impaired AZ collapse from `2/3` to `(2/3)^N`. Instead, keep foundational data-plane dependencies zonal and replicate hard state separately for durability.
- NEVER say "eventual consistency is fine" without naming the client guarantee. It is seductive because it avoids hard design work, but users do not experience "eventual"; they experience missing carts, duplicate creates, and stale reads after a successful write. Instead, specify the inconsistency window and implement read-your-writes, monotonic reads, or session stickiness where the UX needs them.
- NEVER treat a queue as infinite shock absorption because backlog feels safer than rejection. It is seductive because depth can look manageable while producers stay happy, but queue debt creates bimodal latency, stale work, and long recovery tails. Instead, cap useful age, sideline old work, reserve poller headroom, and choose backpressure only when delayed completion is still valuable.
- NEVER re-roll jitter randomly for every scheduled execution because randomness sounds fair. It is seductive because it smooths bursts statistically, but it erases reproducible patterns and makes overload and races impossible to debug. Instead, derive stable jitter from host or workload identity so the spread is consistent across runs.
- NEVER call something a "cell" if requests still need cross-cell coordination or a hot mapping service. It is seductive because the deployment diagram looks isolated, but you have only relocated the single point of failure. Instead, keep cell membership computable from a stable key and keep most requests entirely inside one cell or one shuffle shard.

## Practical Review Loop

1. Mark every dependency as `data plane`, `control plane`, or `operator convenience`.
2. For each data-plane dependency, ask: "If it freezes in place, can current traffic still complete?" If not, redesign before debating implementation details.
3. For each operation, write the exact invariant and pick the weakest consistency model that preserves it.
4. For each overload path, decide one of four outcomes explicitly: shed, queue, delay, or duplicate onto independent capacity. "We will see" is not a policy.
5. Walk one gray-failure scenario, not just a black failure. Assume health checks miss it for several minutes.
6. Verify observability aligns with blast radius: metrics and alarms should dimension by cell, AZ, API, tenant class, and plane. Fleetwide averages hide the failures Vogels cares about.

## Progressive Disclosure

- MANDATORY: Before writing a historical explainer, keynote summary, or quote-heavy overview of Vogels himself, read [`philosophy.md`](philosophy.md).
- MANDATORY: Before compiling primary sources, further reading, or a bibliography for the user, read [`references.md`](references.md).
- Do NOT load `philosophy.md` for architecture reviews, failure analysis, API design, retry policy, consistency choices, or blast-radius work. The working heuristics are in this file.
- Do NOT load `references.md` during live debugging or design critique unless the user explicitly asks for citations or source material.
