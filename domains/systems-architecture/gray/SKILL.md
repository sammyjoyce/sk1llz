---
name: gray-transaction-systems
description: "Design and review transaction-processing systems the Jim Gray way: recovery-first logging, anomaly-aware isolation, replica/2PC tradeoffs, long-lived workflow compensation, and durability contracts that survive real failures. Use when building or auditing WAL and crash recovery, ledgers or payments, idempotent retries, prepared transactions, checkpointing, replica topologies, or any code that claims a write is \"durable\". Triggers: transaction, WAL, fsync, checkpoint, group commit, isolation level, snapshot isolation, SSI, write skew, 2PC, prepared transaction, idempotency key, compensation, replica, failover, crash recovery, durability."
---

# gray-transaction-systems⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​‌‌‌‌‌‍‌​​​‌​‌‌‍​​​​‌‌‌​‍‌​​‌​​​​‍​​​​‌​‌​‍​​‌​​​​​⁠‍⁠

Gray's useful lesson is not "remember ACID." It is "treat failures, retries, and recovery as the normal execution path, then make the happy path a fast special case."

## Start By Classifying The Problem

If the task is a short OLTP write path, optimize around conflict shape and commit latency.

If the task crosses human think time, third-party systems, or hours of wall-clock time, stop calling it a database transaction and model it as a workflow with savepoints, scratchpad state, and compensations.

If the task spans nodes and must commit atomically, the real question is not "can we do 2PC?" but "what happens to prepared state when the coordinator dies?"

If the task touches durability, the contract is defined by the exact point where the caller hears "committed" and by what is guaranteed to be stable at that moment.

If the task is serializable analytics on top of hot OLTP, your main problem is often lock memory and abort behavior, not SQL syntax.

## Before Touching Code, Ask Yourself

- Where is the exact acknowledgment boundary? Name the line of code after which the client is entitled to believe the result survives process death, machine death, and retry ambiguity.
- Which anomaly is acceptable here: lost update, write skew, stale read, fractured read, or none? "We use SERIALIZABLE" is not an answer until you map it to the engine and version.
- If recovery replays this record twice, or replays it after another transaction touched the same entity, do the operations commute or corrupt state?
- What happens if the first attempt committed, the response was lost, and the caller retries with a fresh connection?
- If a prepared or in-flight transaction is stranded for an hour, who notices, who resolves it, and what resource stays pinned in the meantime?

If any answer is vague, pause and load the task-specific reference file before changing code.

## Design Order

1. Lock down the recovery transcript first: log record types, page or object identifiers, checksum strategy, commit record, and the restart rule for each failure point.
2. Define the unknown-outcome boundary next: every caller-visible side effect after that line needs a client-generated idempotency key or an explicit compensator.
3. Pick isolation only after you know the invariants. Choose by anomaly budget and conflict geometry, not by copying a vendor default.
4. Choose replication and distributed commit last. The wrong topology can dominate deadlocks and operator pain even when local code is perfect.
5. Add crash and restart tests before tuning throughput. If the system has never been killed mid-commit, its recovery story is a guess.

## High-Signal Heuristics

- Snapshot Isolation is strong for short, sparse-conflict updates and excellent for read-heavy workloads, but it is the wrong default for predicate-based invariants. Berenson et al. showed that SI allows write skew while forbidding classic ANSI phantoms; every multi-row "at least one remains true" rule is suspicious under SI.
- PostgreSQL-style SSI is not "SI but a bit stricter." Its win is that it preserves serializability without blocking readers, but its cost is aborts and SIREAD state. Long read-only reports should use `READ ONLY DEFERRABLE` when available so they wait for a safe snapshot instead of pinning read dependencies; the production PostgreSQL SSI paper reports safe snapshots usually appear in 1-6 seconds under heavy load and stayed under 20 seconds in their benchmark.
- Long-lived workflows should be modeled as visible progress plus compensation, not hidden locks. Gray's 1981 guidance still holds: store scratchpad or work-area state durably, checkpoint savepoints, and log the compensator name plus arguments for every externally visible action.
- For long-lived workflows that touch shared entities, prefer delta-style operations that commute under replay and compensation. `balance += 5` and `balance -= 5` can be undone around concurrent work; `balance = 10` cannot. This is why ledgers age better than mutable balance rows.
- Tight checkpoints shorten restart time but increase post-checkpoint full-page-image traffic. On PostgreSQL-style systems, frequent checkpoints often make WAL volume worse, not better, because the first write to each page after a checkpoint logs the whole page again.
- Group commit is only tunable if you measure flush cost first. PostgreSQL's practical rule is to start `commit_delay` at about half the average single-flush time from `pg_test_fsync`; on kernels with 10 ms sleep granularity, any nonzero delay from 1 to 10000 microseconds behaves like 10 ms and becomes a latency trap.
- If WAL insertion is forced to write buffers while holding page locks, the bottleneck is often WAL buffer pressure, not the storage device. Increase WAL buffer headroom before you blame the disk.
- File-system semantics are part of the transaction protocol. Rename, delete, truncate, and file growth are not abstract metadata operations; they define what survives power loss. SQLite's rollback-journal notes are worth internalizing: safe-append, powersafe-overwrite, and hot-journal behavior change what is actually durable, and on many systems `PERSIST` is faster and safer than delete-or-truncate churn because it reuses the journal without forcing directory updates.

## Distributed Commit And Replication

- Eager update-everywhere is rarely a scaling plan for OLTP. The Gray/Helland/O'Neil/Shasha deadlock formula grows with `nodes^3` and `actions^5`; doubling writes per transaction can multiply deadlocks by 32, and a 10x node increase can multiply them by 1000. If you cannot explain why your workload escapes that geometry, you do not have a multi-writer design.
- Primary-copy ownership is the default for mutable data. Multi-primary needs a proof, not confidence.
- Classic 2PC is acceptable only when blocked prepared state is operationally survivable. Gray and Lamport's Paxos Commit result is the clean reminder: 2PC is the `F = 0` case. If coordinator failure must not block progress, you need consensus-backed commit and the extra coordinators and delay that come with it.
- Presumed Abort is the normal OLTP default because "no record means abort" removes force-writes on aborts and lets read-only participants vote read-only and disappear. Presumed Commit only wins when committed distributed updates dominate enough to justify an extra coordinator force-write so workers can commit with less forcing.
- Prepared transactions are a storage and operations feature, not just a protocol feature. In PostgreSQL, forgotten prepared transactions keep locks, interfere with VACUUM, and can drive transaction ID wraparound shutdown. Leave `max_prepared_transactions=0` unless an external transaction manager and sweeper are genuinely in place.

## NEVER

- NEVER call a write "durable" because the code reached `write()` or because one replica received the bytes; the seductive shortcut is lower benchmark latency, but the consequence is unknown-outcome retries and double effects. Instead define durability at the first stable log or quorum that can recover the write without client help.
- NEVER retry a timed-out commit blindly because the seductive mental model is binary `{committed, not committed}`; the real third state is `{committed, reply lost}`. Instead require a client-generated idempotency key whose result row is stored in the same transaction as the business effect.
- NEVER trust isolation level names across engines because the seductive shortcut is vendor-name matching; the consequence is shipping write skew or lost updates under a false sense of safety. Instead enumerate tolerated anomalies per engine and version, then add locks or serialization exactly where the invariant lives.
- NEVER hold database transactions across human think time, partner APIs, or slow queues because the seductive path is "keep it atomic end to end"; the consequence is square-law deadlock growth, impossible restart semantics, and abandoned in-flight work. Instead persist workflow state, commit short steps, and compensate visible actions.
- NEVER treat `fsync()` failure as retryable because the seductive story is "just flush again"; on several kernels, a writeback error may be reported once and later `fsync()` can succeed even though dirty data was discarded. Instead treat commit-path `fsync` EIO or ENOSPC as fatal and recover from WAL after restart.
- NEVER delete or rename journals, manifests, or recycled WAL files out of band because the seductive belief is "the transaction is done, the file is garbage"; the consequence is corrupting a hot journal or losing the metadata that makes recovery deterministic. Instead let the engine's own mode transition or segment recycling remove them.
- NEVER enable prepared transactions "just in case" because the seductive benefit is future flexibility; the concrete consequence is pinned locks and garbage-collection or vacuum blockers that appear only during incidents. Instead keep the feature off until you also have monitoring, aging alerts, and an automated resolver.

## Freedom Calibration

Low freedom: commit sequence, WAL ordering, fsync error handling, prepared-state transitions, and file-durability rules. Do not improvise here.

Medium freedom: isolation choice, lock strategy, group-commit tuning, and replica topology. Use the heuristics above and validate with workload-specific tests.

High freedom: compensation semantics, workflow boundaries, and domain-level recovery UX. The business meaning matters more than the storage engine here, but it still must be logged and replayable.

## Mandatory Loading Triggers

- Replication topology, active-active design, or multi-primary discussion: read `references/deadlock-scaling.md` first. Do not load `references/wal-durability.md` unless the task also changes commit or flush behavior.
- WAL, `fsync`, checkpoints, file rotation, journaling mode, or crash recovery: read `references/wal-durability.md` first. Do not load `references/isolation-anomalies.md` for purely storage-path work.
- Isolation levels, read-modify-write correctness, lost updates, write skew, or SSI behavior: read `references/isolation-anomalies.md` first. Do not load `references/deadlock-scaling.md` unless replicas can write concurrently.
- Retry logic, failover, process supervision, or fault taxonomy: read `references/failure-taxonomy.md` first, then `references/code-patterns.md` if you need implementation patterns.
- Transaction manager, idempotent executor, outbox, or compensation workflow code: read `references/code-patterns.md` first. Do not load `references.md` during an incident; it is for paper lookup, not fast decision support.
- Paper citations or broader Gray background for a design memo: load `references.md` only after you already know the sub-problem. Do not load `philosophy.md` for implementation or incident response work.

## Fallbacks When The Ideal Design Is Off The Table

- Inherited multi-writer replication and cannot re-architect this quarter: shrink transaction write sets aggressively, move cross-node effects to append-only inbox or outbox tables, and assign ownership of hot keys to one writer. The `actions^5` term means reducing writes per transaction pays back immediately.
- Cannot change the global isolation level: add a sentinel row lock or advisory lock exactly around the invariant, not around the whole workload.
- Long read-only serializable reports keep aborting or consuming memory: run them on safe or deferrable snapshots, or move them to a replica with an explicit staleness contract.
- Sync commit cost dominates but data loss of a short window is acceptable: use asynchronous commit with an explicit written loss budget, never `fsync=off`. PostgreSQL's documented risk window is up to three times `wal_writer_delay`.
- You cannot make an external side effect undoable: defer it until after durable commit and pair it with an outbox plus operator-visible reconciliation, rather than pretending it is part of one atomic transaction.
