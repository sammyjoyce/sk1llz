---
name: dean-warehouse-scale-distributed
description: "Jeff Dean style for interactive, warehouse-scale distributed systems where fanout, skew, and shared-cluster interference dominate. Use when designing or debugging multi-tenant services, large fanout RPC graphs, data platforms, storage schemas, sharding strategies, tail-latency mitigation, straggler handling, or load-balancing at Google-like scale. Triggers: p99, hedge requests, tail latency, stragglers, Bigtable, MapReduce, hot shards, selective replication, noisy neighbors."
---

# Dean Large-Scale Systems⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​‌​​​​‍​‌‌‌‌‌​‌‍​​‌‌‌​​‌‍‌‌​‌​‌‌‌‍​​​​‌​‌​‍​‌‌‌​‌​‌⁠‍⁠

This skill is for systems that are already "working" but start failing once fanout, skew, and shared-cluster interference show up. It is not a generic distributed-systems primer.

## Operating Stance

- Treat average latency as a decoy. If one backend has a 1% chance of a 1 second hiccup, a request that touches 100 such backends has about a 63% chance of crossing 1 second.
- Separate fault tolerance from variability tolerance. Faults happen on seconds-to-hours loops. Latency hiccups happen on millisecond loops thousands of times per second. Reuse redundancy, but not the same controller.
- Assume the scarce resource is usually not CPU. It is often queueing headroom, metadata locality, network bisection, or human debuggability.
- Prefer simple user-visible abstractions with aggressive internal specialization. Dean systems hide machinery; they do not export it.

## Before You Change The Design, Ask Yourself

- What is the smallest unit I can move: request, partition, replica, or entire machine? If the answer is "machine," rebalancing will be too coarse.
- Which percentile actually misses the product SLO, and what leaf budget does that imply after fanout?
- Which work is safe to duplicate, abandon, delay, or mark partial without corrupting correctness?
- Is the pain caused by skew, queueing, metadata indirection, or a noisy shared cluster? Dean-style fixes differ by failure shape.
- If the control plane or metadata path wedges, does the data path keep serving, or does everything stall waiting for coordination?

## Decision Guide

### If the request fans out to many backends

- Budget from the tail backward. Service-level latency is dominated by the slowest leaf, not the typical one.
- Use differentiated service classes. User-facing RPCs should not queue behind compaction, scans, or batch work.
- Break large requests into smaller queueable units when they create head-of-line blocking. Small requests can slip through; giant ones create convoys.
- Use canary or probe requests when slowness is replica-specific and data-independent. Do not mistake a key-specific hotspot for a sick replica.

### If replicas exist and the tail still dominates

- Hedge late, not immediately. Bigtable's in-memory lookup example improved the 99.9th percentile from 994 ms to 50 ms by sending a backup after 10 ms, with under 5% extra requests.
- Do not confuse "less extra load" with "better latency." The same experiment with a 50 ms hedge kept extra load below 1% but left a worse tail than the 10 ms hedge.
- Prefer tied requests or cross-server cancellation over naive duplication. Dean-style hedging works because the losing replica stops doing work.
- For disk-backed reads, a 2 ms delayed backup with cross-server cancellation cut the 99.9th percentile from 98 ms to 51 ms on an idle cluster and from 159 ms to 108 ms under Terasort, at about 1% extra disk reads.

### If the cluster is shared and "healthy" machines still go slow

- Do not randomize periodic maintenance in a high-fanout fleet. Randomization guarantees that some machine is almost always in a bad state. One synchronized blip is often better than a permanent low-grade tail.
- Use latency-induced probation. When a machine turns slow, temporarily remove it from the hot path, keep a shadow stream of probes, and only return it after latency stays clean long enough.
- Balance on latency signals and requests-in-flight, not CPU alone. Queueing, lock contention, or antagonist load can destroy p99 while utilization looks fine.

### If hotspots and recovery dominate

- Use more partitions than machines. Dean's serving/storage pattern is often 10-100 partitions per machine so load can move in few-percent increments.
- Keep movement units small enough that many machines can absorb a failure in parallel. Bigtable used tablets around 100-200 MB by default; MapReduce commonly used 16-64 MB tasks.
- Replicate selectively, not uniformly. Hot languages, hot geographies, and hot keys deserve extra replicas before the rest of the fleet does.
- Encode the read pattern into the key. Bigtable's reverse-hostname row keys are a classic example: they trade lexical convenience for locality that matches real scans.

### If the workload is batch or storage-heavy

- Assume the last few workers determine wall-clock completion. MapReduce backup tasks typically cost only a few percent more resources yet made one large sort 44% faster than running without them.
- Re-execution is the preferred failure primitive only when outputs are atomic and idempotent.
- Treat network bandwidth as scarce until measured otherwise. Favor locality and shrink intermediate data early with partial aggregation.
- Be suspicious of "small random reads" from disk-backed LSM storage. Bigtable measured only about 1200 random 1 KB reads/sec because each miss pulled a 64 KB SSTable block and saturated CPU.

## Anti-Patterns

- NEVER optimize the mean and declare success, because fanout turns rare backend hiccups into common user-visible failures. Instead set per-leaf p99 or p99.9 budgets and design from the slowest path backward.
- NEVER send naive duplicate requests everywhere, because the seductive "easy fix" doubles expensive work exactly when the system is already distressed. Instead hedge after a measured delay and cancel the losing copy remotely.
- NEVER randomize background daemons in a large fanout service, because local smoothing becomes a globally continuous slow set. Instead synchronize the disruption or gate it behind explicit load checks.
- NEVER size shards, tablets, or tasks at machine granularity, because coarse units make balancing and recovery lumpy. Instead choose units small enough to move load in small increments.
- NEVER add a general feature before you have workload evidence. Bigtable delayed general distributed transactions because real users mostly needed single-row atomicity plus specialized index maintenance, not a universal transaction layer.
- NEVER build a protocol on obscure coordination-service behavior, because the clever path creates corner cases no other workload exercises. Instead depend on the most widely used primitives in the control plane.
- NEVER monitor only the storage or serving tier, because many real bottlenecks live in clients, metadata lookups, RPC stacks, or lock paths. Instead sample end-to-end traces that include control-plane hops.
- NEVER assume failures are clean fail-stop events, because real fleets produce hung machines, clock skew, asymmetric partitions, memory corruption, quota exhaustion, and dependency bugs. Instead make protocols tolerate ugly failure surfaces and unexpected error codes.

## Edge Cases And Fallbacks

- If hedging makes overload worse, first lower backup priority, then lengthen the hedge delay, then restrict hedges to idempotent reads. Do not disable the mechanism globally until you know whether the problem is threshold, cancellation, or request class.
- If metadata lookups become part of the latency budget, cache aggressively on the client and prefetch adjacent metadata. Bigtable notes that stale location caches can expand a lookup to as many as six round-trips.
- If a coordination service becomes a hidden availability tax, treat it as production traffic, not admin plumbing. Bigtable measured average unavailability from Chubby issues at 0.0047% of server-hours, with the worst cluster at 0.0326%; tiny control-plane outages still show up at scale.
- If partial answers are acceptable, dynamically cut off slow subtrees and mark the result as tainted in caches. Never let incomplete data be cached as authoritative.
- If multiple teams share the same table or cluster, add quotas early. Bigtable learned that shared per-user tables accumulate column families and neighbor effects faster than governance catches them.
- If you are choosing between a smart generic mechanism and a boring specialized one, pick the specialized one unless you can name the production workload that needs the extra generality.

## What This Skill Is For

- Use it when the bug or design problem appears only after fanout, shared-cluster interference, hot partitions, metadata bottlenecks, or stragglers show up.
- Do not use it for ordinary CRUD services, single-node tuning, or textbook consensus explanations.
- Keep this skill at the architecture and control-loop level. Once the design choice is clear, switch to the relevant language or storage skill for implementation details.
