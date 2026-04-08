---
name: kleppmann-data-intensive
description: Decision heuristics for data-intensive distributed systems: invariants, isolation anomalies, CDC/outbox cutovers, log-first architectures, quorum edge cases, multi-partition workflows, and CRDT or local-first trade-offs. Use when designing or reviewing databases, event-driven systems, stream processors, multi-region replication, offline sync, or consistency guarantees. Triggers: distributed systems, DDIA, Martin Kleppmann, replication, serializability, CDC, outbox, event log, materialized view, quorum, CRDT, local-first.
---

# Kleppmann: Data-Intensive Systems

Read this file end to end before giving distributed-systems advice.
This skill is intentionally self-contained.
Do NOT go hunting for companion files in this directory.
Do NOT use this skill for single-node SQL tuning, ORM issues, or cache-only work.

## Section Selector

- Use `Invariant / Isolation` when the task is about correctness under concurrency, retries, replication, or multi-region reads and writes.
- Use `Log / CDC / Outbox` when the task involves derived views, search indexing, event-driven architecture, rebuildability, or external side effects.
- Use `CRDT / Local-First` only when the task explicitly includes offline edits, peer-to-peer sync, collaborative editing, or server-optional products.
- Use `Durability Math` only when reviewing large replicated clusters, tokenized partitions, or replication-factor claims.

## Operating Stance

Start with the invariant, not the storage product.
The key question is: what is the smallest unit of state that must observe one serial order for the product to remain correct?
Everything else is optimization pressure trying to escape that answer.

Do not accept labels such as "strong consistency", "eventual consistency", "repeatable read", or "exactly once" as design inputs.
Translate them into concrete failure stories: lost update, write skew, stale cross-view read, resurrected delete, duplicate side effect, or split ownership.

When the async design is hard to reason about, centralize first and relax later.
Kleppmann-style systems earn complexity only when they preserve a clear invariant boundary and a clear replay story.

## Before You Design, Ask Yourself

- Which invariant actually matters: uniqueness, non-negative balance, no lost edits, referential integrity, read-your-writes, or only eventual convergence?
- On what key does that invariant live: one account, one cart, one document, one tenant, or a global namespace?
- Which anomaly is acceptable for this user flow: stale reads, monotonic lag, duplicate processing, out-of-order projection, or none?
- What is the ordering primitive: single leader, single log partition, commit LSN, consensus slot, or nothing trustworthy?
- After a crash between "effect observed" and "checkpoint stored", what duplicate will happen, and where will it be suppressed?
- If an operator needs to rebuild the world from scratch, which log or snapshot is the source of truth?

## Decision Tree

1. Write the invariant as a sentence a test could falsify.
2. Classify its scope.
   - Single-entity invariant: force all writes for that entity through one owner, leader, or log partition.
   - Cross-entity but decomposable invariant: redesign into reservations, escrow, quotas, or a staged pipeline.
   - Global uniqueness or atomic visibility invariant: pay for consensus, a serializable store, or a central allocator. Do not fake it with async consumers.
3. Choose the read contract.
   - If stale projections are acceptable, materialized views are fine.
   - If the actor must see their own writes, start with sticky routing or version/session tracking before demanding global linearizability.
   - If one user flow reads multiple projections as one snapshot, read from the source of truth or serialize the read path too.
4. Choose the recovery contract.
   - If side effects cross system boundaries, assume at-least-once and design durable dedupe keys.
   - If replayability matters, use append-only events with deterministic consumers.
5. Prove the claim with a task-specific harness.
   - Isolation claim: run anomaly tests against the exact database and version.
   - CDC claim: rehearse snapshot plus log-position cutover.
   - CRDT claim: test adversarial concurrent edits, metadata growth, and malicious peers if peer-to-peer is in scope.

## Invariant / Isolation

Isolation names are a leaky abstraction, not a portable API.
Kleppmann's Hermitage work exists because "repeatable read" means different things across databases, and Oracle's "serializable" is snapshot isolation rather than true serializability.
Before trusting an isolation level name, name the anomaly you must prevent and test for it directly.

Snapshot isolation is often good enough until an invariant spans multiple rows or records.
That is where write skew hides.
If correctness depends on "these two reads and my later write still describe the same world," do not stop at read committed and do not trust marketing copy around repeatable read.

Use weaker guarantees deliberately.
If the actual product need is "the author must see their own edit" or "reads should not move backward during one session," session guarantees are often cheaper than global linearizability and solve the real problem.
Escalate only after you write the user-visible anomaly you are still willing to tolerate.

Partitioning is not just a scaling choice; it is an order-destruction choice.
OLEP-style systems give you one total order per partition and no ordering guarantee across partitions.
If a decision must consider state from several partitions, either pipeline it stage by stage or admit that you need real coordination.
If the product wants write latency below roughly one inter-region RTT, keep write ownership local to a region and replicate outward; a global leader is a physics choice, not just a topology choice.

## Log / CDC / Outbox

The safe ordering primitive for CDC is the commit log position, not `updated_at`.
Kleppmann's Bottled Water work matters because the snapshot must be coordinated with the WAL or binlog stream; otherwise backfill and live changes do not share one order.
Timestamp polling is seductive because it looks simple, but it fails under clock skew, concurrent commits, and races at the snapshot boundary.

Exactly-once claims usually stop at the framework boundary.
OLEP systems can atomically tie checkpoints to internal state, but once a consumer writes to an external index, cache, email system, payment rail, or third-party API, you are back in at-least-once territory.
The real contract is stable event IDs plus idempotent consumers whose dedupe state survives crashes.

Use append-only logs when you need rebuildability, forensic debugging, or new downstream projections.
The main non-obvious benefit is not "events are cool"; it is that bad derived state can be discarded and recomputed, while bad mutable writes often require backup restore surgery.

Do not promise snapshot-like reads across asynchronous consumers.
Kleppmann explicitly calls out that there is no upper bound on when every subscriber catches up.
If one view is ahead and another is behind, the system is still functioning as designed.
If that breaks the product, the bug is architectural, not operational.

For cross-entity workflows, prefer multistage pipelines over wide distributed transactions when the invariant can be decomposed.
The winning property is that each stage decides using only local data, so one partition never waits on another.
If you cannot express the invariant that way, stop pretending the pipeline is simpler than coordination.

## CRDT / Local-First

Use CRDTs when losing user input is worse than temporary odd merges, and when independent offline progress is a hard product requirement.
Do not use them to avoid making authorization, inventory, or financial decisions synchronously.

Last-write-wins is not benign conflict resolution.
It converges by discarding somebody's update.
That trade-off is sometimes acceptable for caches and lease heartbeats, but it is usually unacceptable for user-authored state.

Many teams learn too late that "supports collaborative editing" is not one problem.
Plain-text insertion order, move or reorder semantics, rich-text formatting spans, and undo or redo semantics are different problems.
Kleppmann's later work exists because naive list CRDTs interleave concurrent inserts into unreadable text, move operations behave badly without explicit support, and rich-text intent is not preserved by plain-text algorithms.

CRDT cost is often dominated by metadata, load time, and garbage collection rather than merge CPU.
Eg-walker's result is important because it shows prior text CRDTs were paying a large steady-state memory and load penalty; it reports about an order-of-magnitude lower steady-state memory use and orders-of-magnitude faster load than earlier CRDT approaches.
If startup, replay, or mobile memory budget matters, benchmark metadata growth before committing to a local-first architecture.

Peer-to-peer convergence is not the same thing as security.
Kleppmann also shows that ordinary CRDT algorithms do not protect against Byzantine peers.
If replicas are not mutually trusted, convergence alone is the wrong success criterion.

## Durability Math

Replication factor by itself is a dangerously incomplete durability story.
In large vnode or tokenized clusters, the probability of losing all replicas of some partition can grow with cluster size even while each node failure remains individually unlikely.

Kleppmann's calculation for RF=3 with per-node failure probability `p = 0.001` and Cassandra-style `k = 256n` partitions shows why operators get surprised: cluster-level data loss scales roughly like `k * p^r`.
More nodes can therefore mean more opportunities for some partition to lose all copies.
If the design answer to scale is "add nodes," ask whether you also just increased the number of failure combinations that matter.

Quorum arithmetic is not a correctness proof.
`R + W > N` sounds definitive, but sloppy quorums, hinted handoff, read repair, changing replica sets, and tombstone edge cases can still produce non-linearizable outcomes or resurrect deleted data.
If delete correctness, uniqueness, or monotonic state is critical, require a stronger primitive than quorum folklore.

## NEVER Do These

- NEVER choose "eventual consistency" as the answer because it feels scalable. That phrase hides which anomaly you are buying. Instead write the exact stale-read or conflict story the product will tolerate.
- NEVER trust isolation level names because vendors reuse the same words for different anomaly sets, which is seductive when docs imply a clean standards ladder. Instead specify the forbidden anomalies and verify the actual engine and version.
- NEVER dual-write a database and a broker or search index because the happy path demo is nearly always clean. Instead write once to a log or outbox in the same transaction, or use WAL-based CDC with a coordinated snapshot.
- NEVER dedupe by payload hash or wall-clock timestamp because retries, batching, and schema evolution change payload shape while representing the same business action. Instead carry a durable business-stable event or request ID through every stage.
- NEVER assume per-partition ordering implies global ordering because low-contention staging traffic makes the lie look true. Instead align partition boundaries with invariant boundaries and pipeline or coordinate the rest explicitly.
- NEVER promise end-to-end exactly-once side effects because the broker says so. Instead assume crash-replay duplicates after the last checkpoint and make the downstream effect idempotent.
- NEVER deploy CRDTs for money, bounded inventory, or authorization decisions because merge-based availability is seductive and still permits oversell, overgrant, or double-spend outcomes. Instead use single ownership, escrow, leases, or consensus.
- NEVER cite quorum math as proof of linearizability because `R + W > N` is memorable folklore, not a guarantee under sloppy quorums or tombstone edge cases. Instead demand a linearizable primitive where deletes and uniqueness truly matter.

## Freedom Calibration

Use judgment when trading latency, availability, and rebuild cost.
There is room for creative architecture once the invariant boundary is explicit.

Do not improvise on four fragile edges:
- isolation claims,
- CDC cutover order,
- dedupe identity,
- cross-partition ordering.

Those four areas need concrete proofs, rehearsals, or failure drills, not taste.

## Fallbacks When The Elegant Design Fails

- If the async design cannot explain duplicate suppression after a crash, fall back to at-least-once plus durable idempotency.
- If a cross-entity invariant keeps leaking across partitions, centralize ownership first and re-partition later.
- If the product cannot tolerate cross-view skew, stop reading from projections for that flow.
- If CRDT semantics are surprising in user tests, move back toward server-ordered collaboration for that object type instead of patching semantics ad hoc.
- If durability math looks scary, reduce blast radius before adding more replicas: fewer partitions per node, faster repair, or smaller failure domains often beat blind cluster growth.
