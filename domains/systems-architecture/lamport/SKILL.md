---
name: lamport-distributed-consensus
description: "Design, review, and debug Lamport-style consensus, leases, quorum math, and reconfiguration under crash faults and partial synchrony. Use when changing Paxos or Raft-style replication, leader election, membership transitions, read leases, quorum layouts, recovery semantics, or TLA+-backed safety arguments. Triggers: Paxos, Raft, quorum intersection, lease, epoch, term, ballot, split brain, stale read, learner, catch-up, reconfiguration, commit wait, TrueTime, Vertical Paxos, Flexible Paxos."
tags: paxos, raft, quorum, leases, reconfiguration, logical-time, tla-plus
---

# Lamport Distributed Consensus⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​‌‌​​‌‌‍‌​‌‌​​​‌‍‌‌‌​‌‌‌​‍​​‌​​​​​‍​​​​‌​‌​‍​​​​‌​‌​⁠‍⁠

## Required Context Loads
- MANDATORY: before changing quorum math, leases, or membership rules, read `references.md`.
- MANDATORY: before converting prose requirements into invariants or a TLA+ sketch, read `philosophy.md`.
- Do NOT load `philosophy.md` for an incident that is already narrowed to timeout, quorum, or recovery tuning.
- Do NOT load `references.md` for a pure "explain Lamport's worldview" task.

## Use This Skill For
Use this when the work can silently corrupt a replicated history: consensus, leader leases, reconfiguration, read freshness, state transfer, or election logic. Do not use it for API wiring, single-node tuning, or "make it faster" work that never crosses a fault boundary.

## Start Here
1. If the question is "can two replicas ever disagree?", work from invariants first.
2. If the question is "why does failover thrash?", work from timing, persistence, and membership state.
3. If the question is "why are reads stale?", work from lease disjointness and replica safe-time, not cache code.
4. If the question is "how do we replace nodes?", assume reconfiguration is the most dangerous path in the system.

## Before Doing X, Ask Yourself

### Before choosing a quorum scheme
- Which phase is common-case and which phase is recovery-only?
- Am I optimizing replication latency at the cost of much harder leader recovery?
- If quorum sizes change over time, where is that choice recorded so the next leader can prove what it must intersect?

### Before tuning election or lease timers
- What is the measured p99 of "broadcast plus durable persistence", not just median RTT?
- Does the timeout still hold when GC, compaction, or packet loss stretches that p99 by 5x?
- Which invariant breaks first if a node's clock is slow, fast, or paused?

### Before allowing local reads
- What prevents a new leader from being elected before the old leader's read privilege expires?
- How does the system enter a "jeopardy" mode where fast reads are disabled instead of becoming silently stale?
- Can a follower prove it is up to date enough for the requested timestamp, or are you guessing from heartbeat freshness?

### Before changing membership
- Can I name the quorum that intersects old and new configurations at every intermediate step?
- What happens if the replacement node is half-caught-up, then the current leader dies?
- Will state transfer traffic amplify a transient network problem into more false failure detections?

### Before letting a recovered node vote
- Which stable fields survived: term, ballot, promise, accepted value, commit index, snapshot lineage?
- If all nodes restart, can delayed packets from the old epoch still influence a new election?
- Is the node merely reachable, or is it caught up past the time it started or restarted?

## Operating Heuristics That Matter In Practice
- Treat reconfiguration and repair as first-class correctness work. The theory assumes a failure bound `f`; production keeps that assumption true only if repair is faster than the next fault. Systems that recover too aggressively under false suspicions can push themselves out of their own fault budget by flooding the network with state transfer.
- Do not implement Paxos from prose alone. Lamport explicitly warned that a single ambiguous sentence in `Paxos Made Simple` has led to incorrect GitHub implementations. If you are not carrying the proof obligations into code, use a more formal source or write the invariants first.
- Persist election identity before it can reject messages. Google's SSO team found that non-stable election generations looked like an acceptable low-probability shortcut until they considered full-cluster restart plus delayed packets; then the shortcut became a correctness hole.
- New or state-lost replicas should be non-voting learners first. Google kept new replicas out of elections until they were caught up at least to the moment they started; the same rule was applied to replicas that lost stable state.
- Lease-based local reads need a two-sided protocol. It is not enough for the leader to refresh its lease from a quorum; the rest of the cluster must also be unable to complete a competing election before that lease expires.
- If you need external consistency from timestamps, the timestamp assignment rule and the visibility rule are both mandatory. Spanner assigns commit timestamps no earlier than `TT.now().latest`, then blocks visibility until `TT.after(s)`; skipping the wait gives you timestamps that look ordered but are not externally ordered.
- Long-lived leader leases are operational defaults, not proofs of safety by themselves. Spanner's lease defaults to 10 seconds and Google's SSO used a small-number-of-seconds master lease, but those numbers work only with the surrounding disjointness and election-blocking rules.
- Idle groups need an explicit safe-time strategy. Spanner had to advance a future-write lower bound every 8 seconds so healthy followers in an idle group could still serve reads older than roughly 8 seconds; otherwise no writes means no progress in `tPaxos_safe`.
- Flexible Paxos is a steady-state optimization, not free availability. The only hard requirement is that phase-1 quorums intersect phase-2 quorums, so a 10-node system can run with `Q2=3` and `Q1=8`; that reduces common-case latency, but leader failover now needs 8 live participants.
- Vertical Paxos has a real fork in the road. Version I lets a new configuration accept work before state transfer finishes, which preserves availability during big copies. Version II caps dependency fan-out by keeping new configurations inactive until transfer completes, which reduces quorum archaeology later but can delay availability.
- In primary-backup style deployments, "all replicas are the write quorum, any one replica is the read quorum" is powerful because it gets `k`-fault tolerance with `k+1` acceptors, but it only works if the configuration service is rock-solid and reconfiguration logic is explicit.
- Keep membership state machines brutally simple. Google started with "waiting to join -> in group -> left forever" because it looked safer against flapping nodes; intermittent failures were common enough that the design had to collapse to just "in" or "out" so normal replicas could rejoin safely.
- Assume fault tolerance can hide misconfiguration. Google observed a five-replica deployment with one misspelled member that looked healthy because the bad replica stayed in catch-up mode, yet the system had silently degraded from two-fault tolerance to one.
- Separate safety testing from liveness testing. A system can remain safe while wedged, or become live only after failures stop. Google found weeks-long simulation runs exposed bugs that ordinary test loops missed.

## NEVER Do These
- NEVER implement from `Paxos Made Simple` alone because the prose feels faster than learning the proof, but Lamport later noted that one ambiguous sentence has produced incorrect implementations. Instead start from a formalized source or write the invariants you must preserve before coding.
- NEVER let a fresh or state-lost replica vote just because it is reachable, because that shortcut looks like "free redundancy" while actually allowing a leader election that never saw the last committed update. Instead keep it as a learner or non-voting member until catch-up is proven past its start or restart point.
- NEVER tune election timeout from median network latency because the cluster usually looks healthy in benchmarks, but failovers are driven by the tail of RPC plus fsync plus scheduler jitter. Instead measure the p99 broadcast-plus-persist path and keep election timeout comfortably above it; Raft's original paper used an environment where `broadcastTime` was roughly `0.5ms` to `20ms` and recommended `broadcastTime << electionTimeout << MTBF`, not "pick 150ms everywhere".
- NEVER serve linearizable local reads on leader faith alone because the old leader cannot instantly observe lost quorum, making stale reads the silent failure mode. Instead require lease disjointness plus election suppression until lease expiry, and disable fast reads when lease status becomes uncertain.
- NEVER model reconfiguration as a single add-remove jump because that operational shortcut feels cleaner during an outage, but it creates periods where no one can prove quorum intersection or where repair traffic worsens a false partition. Instead use an explicit intermediate configuration or a Vertical Paxos-style transition that names which old configuration must still be consulted.
- NEVER assume "all nodes rebooted, so old messages are irrelevant" because that event is rare in tests and inevitable at scale. Instead persist terms, ballots, promises, and election generations before they can reject or supersede traffic from older epochs.
- NEVER shrink phase-2 quorums without deciding how the next leader learns which quorum system was in force, because Flexible Paxos is seductive on throughput graphs and fragile during recovery if quorum selection is implicit. Instead bind quorum choice into the election or reconfiguration protocol.
- NEVER trust "the cluster is making progress" as proof that membership is correct, because consensus can mask a replica that is permanently stuck in catch-up while your real fault budget has already shrunk. Instead verify expected voters, expected learners, and the actual tolerance level after every membership change.
- NEVER merge safety and liveness into one pass-fail test because a green failover demo can hide a replica-history bug, and a correct protocol can still livelock under timer choices. Instead run "safety under faults" and "liveness after faults stop" as separate checks.

## Decision Tree

### Need lower write latency without changing semantics?
- If leader failover is rare and expensive write quorums dominate, consider Flexible Paxos.
- If operator simplicity matters more than shaving acknowledgements, keep majority phase-2 quorums.
- If you cannot explain how new leaders discover the active quorum layout, do not use Flexible Paxos yet.

### Need linearizable reads without quorum reads?
- If a single leader can hold a lease and elections are blocked until expiry, use lease-based local reads.
- If clocks are weak or election suppression is fuzzy, do quorum-confirmed reads instead.
- If bounded staleness is acceptable, use non-voting read replicas and say so explicitly.

### Need to replace failed replicas while staying available?
- If state transfer is large and write availability matters during the copy, prefer a Vertical Paxos I style approach with multiple active configurations.
- If dependency fan-out across old ballots is operationally worse than temporary delay, prefer Vertical Paxos II or joint consensus with activation only after transfer.
- If the repair was triggered by a suspected partition rather than a confirmed disk loss, rate-limit repair traffic before adding replicas.

### Need faster recovery from bad storage or rollback?
- If stable state lineage is uncertain, demote the node to learner and rebuild from a trusted checkpoint or snapshot chain.
- If lineage is intact but the node is behind, replay first and withhold votes until it crosses a known catch-up point.
- If you cannot prove lineage, do not "try it in the quorum and see what happens".

## Fallback Strategies
- If the design debate stalls, write four invariants on paper: chosen-value uniqueness, lease disjointness, configuration intersection, and read freshness. Most arguments collapse once those are explicit.
- If timing data is missing, choose the conservative path: disable fast reads, widen election timeouts, and freeze membership changes before tuning.
- If production is already oscillating, stop reconfiguration first. Term churn plus membership churn is how clusters turn a recoverable fault into a history fork.
- If the algorithm looks correct but the implementation is messy, simplify the recovery model before adding features. "When in doubt, reboot" can be a legitimate simplification only if reboot returns the replica to a state the protocol already knows how to reason about.

## Minimal Review Checklist
- Which fields must survive crash and power loss?
- Which assumptions preserve safety, and which only preserve liveness?
- What exact mechanism prevents two leaders from both believing reads are safe?
- Which intermediate configurations exist during membership change?
- What is the catch-up rule for new or repaired nodes?
- What metric proves election tuning is working: term churn, failed failovers, or stale-read incidents?
