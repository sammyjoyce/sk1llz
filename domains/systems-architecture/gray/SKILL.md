---
name: gray-transaction-systems
description: Engineer transaction processing and fault-tolerant data systems using Jim Gray's hard-won principles — write-ahead logging, isolation anomalies (including write skew and the broken ANSI definitions), replication scaling math (the N³ deadlock law), process-pair failover, and the Heisenbug/Bohrbug failure model. Use when building or reviewing databases, payment systems, ledgers, WAL implementations, replication topologies, recovery code, retry logic, or any system where durability, consistency, and fault tolerance are non-negotiable. Triggers — transaction, ACID, WAL, write-ahead log, fsync, durability, isolation level, snapshot isolation, write skew, replication, two-phase commit, deadlock, crash recovery, process pairs, Heisenbug, idempotency, checkpoint, compensating transaction, redo, undo, two-phase locking.
tags: transactions, acid, wal, recovery, replication, isolation, durability, fsync, fault-tolerance, heisenbug, snapshot-isolation, two-phase-commit, databases, distributed
---

# gray-transaction-systems

Jim Gray's career produced a body of knowledge that still governs every reliable database in existence. This skill encodes the *non-obvious* parts — the ones practitioners learn only after a midnight incident.

## First principle: failures are the workload, not exceptions

Gray's 1985 Tandem field data (166 outages, ~2000 fault-tolerant systems, ~1,300 system-years) inverted the standard mental model:

| Cause                     | Share of outages |
|---------------------------|------------------|
| Administration/operations | ~42% |
| Software (mostly Heisenbugs) | ~25% |
| Hardware                  | ~18% |
| Environment (power, cooling) | ~14% |
| Vendor/other              | ~11% |

**Surprising implication:** in a fault-tolerant system, hardware is a *minor* contributor. If you are spending most of your reliability budget on RAID and dual PSUs while leaving operational procedures ad-hoc, you are optimizing the wrong third of the problem. Gray's MTBF for mirrored disks with fail-fast detection was measured at **>5 million hours** (millennia); three-way mirroring adds nothing because other factors dominate. Remote geographic replication protects against ~75% of failures (everything except shared software bugs) and is the only thing that moves the needle beyond four-nines.

## Before writing any "durable" code, ask yourself four questions

1. **What, precisely, is my fsync contract?** If you call `write()` then `fsync()`, which layers (page cache → disk cache → controller cache → NAND) actually committed to stable media, and what happens when any one of them silently drops the flush or returns EIO on the *second* call?
2. **What is my commit boundary — exactly which line of code?** Name the instruction where the client is told "your change is durable." Everything before must be recoverable; everything after must survive every failure class you designed for.
3. **What is my recovery story for: random process kill, full power loss, `fsync` returning EIO, and a torn 8 KB write spanning two 4 KB sectors?**
4. **What is the Heisenbug/Bohrbug ratio of my bugs?** Gray's Tandem data: **131 of 132 production software faults were Heisenbugs** (transient, did not recur on retry). Your fault-tolerance strategy should be fail-fast + restart + retry — *not* "debug it in production." If you can't fail-fast because your process corrupts shared state, your architecture is wrong.

If any answer is vague, STOP and load `references/wal-durability.md` and `references/failure-taxonomy.md` before continuing.

## Rules that come from experience, not textbooks

### NEVER propagate updates eagerly across >5 replicas for an OLTP workload

**Why it's seductive:** "Synchronous replication everywhere gives strong consistency for free."

**What actually happens** (Gray/Helland/O'Neil/Shasha, *The Dangers of Replication*, 1996): deadlock rate under eager update-everywhere scales as

    TPS² × Action_time × Actions⁵ × Nodes³ / (4 × DB_size²)

A **10× increase in nodes produces a 1000× increase in deadlocks**. A 2× increase in transaction size produces a 32× increase. The system does not degrade gracefully — it collapses into *system delusion*: a state where replicas have silently diverged and no algorithm can reconcile them.

**Do instead:** Primary-copy lazy replication (N² scaling, survivable), or two-tier replication for mobile/disconnected nodes. If you truly need eager consistency, keep the cluster ≤3 nodes, transactions ≤5 writes, and partition data so transactions rarely cross partitions. Before designing any replicated OLTP system, load `references/deadlock-scaling.md`.

### NEVER equate your database's `SERIALIZABLE` setting with actual serializability, and NEVER trust "REPEATABLE READ"

**Why it's seductive:** ANSI SQL-92 defines four neat levels; vendors ship things with the same names.

**What actually happens** (Berenson/Bernstein/Gray/Melton/O'Neil, 1995, *A Critique of ANSI SQL Isolation Levels*): the ANSI definitions are formally broken. They omit Dirty Writes (P0) entirely. Snapshot Isolation is **incomparable** with Repeatable Read — SI prevents some phantoms that RR allows, and RR prevents **write skew** (A5B) that SI allows. Worse:

- Oracle and PostgreSQL default to "Read Committed," which allows **lost updates** on unprotected read-modify-write.
- PostgreSQL `REPEATABLE READ` is actually Snapshot Isolation (a different algorithm, different guarantees, susceptible to write skew).
- PostgreSQL `SERIALIZABLE` is SSI (Serializable Snapshot Isolation), which works by *aborting* transactions post-hoc — your retry loop must handle `serialization_failure` or your "serializable" system silently loses updates.
- MySQL/InnoDB `REPEATABLE READ` is not actually repeatable — gap locks change its semantics.

**Do instead:** Look up, per database and version, exactly which anomalies your chosen level permits. Audit every read-modify-write for write skew (the on-call doctor example in `references/isolation-anomalies.md`). When in doubt, use `SELECT ... FOR UPDATE` on a sentinel row or an advisory lock to convert the risky section to serial.

### NEVER assume `fsync()` returning 0 means your data is safe

**Why it's seductive:** `man 2 fsync` reads like a simple synchronous flush.

**What actually happens (fsync-gate, 2018):** Linux's kernel, on write-back failure, clears the page's dirty bit and reports the error **once** to the next `fsync()` caller. A second `fsync()` on the same fd returns success even though your data is gone. PostgreSQL — and nearly every major database — had this wrong for years and fixed it only by *panicking* on any fsync error. Additionally: the page cache may have been flushed to a device whose own write cache you never flushed (consumer SSDs, virtual disks, some RAID controllers without BBUs). And a `write()` of 8 KB crossing two 4 KB sectors can crash into a torn write — WAL survives via checksum + truncate, but data files need Full Page Writes (FPW) or atomic writes.

**Do instead:** Treat any EIO from `fsync` as a **fatal, non-recoverable event** — abort the process, let WAL redo/undo on restart. Verify disk write caches are disabled (or BBU-backed) with power-pull testing, not documentation. Enable FPW for data files. Prefer `O_DIRECT | O_DSYNC` for WAL if the filesystem supports it. Load `references/wal-durability.md` before touching durability code.

### NEVER build a retry loop without an idempotency key generated by the client

**Why it's seductive:** "A network timeout just means I should try again."

**What actually happens:** A timeout tells you **nothing** about whether the server-side transaction committed. Without an idempotency key, your retry becomes a double-charge, double-post, or ledger imbalance. "Exactly-once" at the network layer is mathematically impossible; at the application layer it requires `(key → committed-result)` persisted **inside the same transaction** as the work.

**Do instead:** Generate the idempotency key on the client *before the first attempt*. Persist the key and result atomically with the work. On retry, look up the key first and short-circuit to the prior result. TTL ≥24 h (7 days for ledgers). See `references/code-patterns.md` for the executor pattern.

### NEVER design recovery as a feature to add later

**Why it's seductive:** "Let's get the happy path working first."

**What actually happens:** Recovery code is 50–80% of a serious database system. Bolted on later, it forces retroactive changes to data layout, lock order, and commit paths — and then still fails on the rare case that kills you. Gray's rule: **the log format is the most important data structure, and it is decided on day one.** Every subsequent choice must answer: "if we crash here, what does the log say, and how do we replay it deterministically?"

**Do instead:** Write the recovery path *first*, even as a stub that reads the log and prints what it would do. Then route every write through that log format.

## Decision tree: concurrency mechanism

```
Low contention + read-heavy?
  └─► Snapshot Isolation  (audit every RMW for write skew)

High contention + long analytics?
  └─► MVCC: SI for readers, 2PL for writers

Multi-row invariants ("at least one doctor on call")?
  └─► SSI  OR  explicit SELECT FOR UPDATE on a sentinel  OR  advisory lock

Conflicts rare, throughput maximal?
  └─► OCC with retries (budget the abort rate explicitly)

Conflicts on hotspots (counters, sequences)?
  └─► Short pessimistic 2PL, OR funnel writes through a single-writer queue

Distributed across >3 nodes with frequent cross-node writes?
  └─► STOP. Read references/deadlock-scaling.md. Reshape to primary-copy or shard ownership.
```

## Decision tree: durability level

```
Synchronous client waiting on "committed"?
  ├─ Zero-loss cross-DC required?
  │    └─► Sync replication + fsync on remote before ACK. Budget 10–100 ms commit floor.
  └─ Can batch commits?
       └─► Group commit: batch fsyncs every 5–10 ms. 10–100× throughput over per-commit fsync.

Analytics/logging where some loss is acceptable?
  └─► Async commit (Postgres synchronous_commit=off; RocksDB sync=false).
      Document the loss window (page-cache interval) in the runbook. Never call this "durable".

"Losing the last second" is existential?
  └─► No write-back caches anywhere in the stack. Verify with power-pull tests, not docs.
```

## Fallbacks when you can't do it "right"

- **You inherited eager multi-master and can't rewrite it.** Shrink transactions aggressively — the deadlock rate is *fifth power* in action count. Splitting one 10-write transaction into two 5-write transactions cuts the deadlock rate by ~32×.
- **You need strong consistency but can't afford 2PC latency.** Single-leader + primary-copy lazy replication for reads; route all writes to the leader; automate failover. This is what most successful systems do and Gray would approve.
- **You can't change isolation levels in production.** Add application-level locks: Postgres `pg_advisory_xact_lock()` or a dedicated sentinel row. Converts risky RMW into a serial section without schema changes.
- **You can't afford a full WAL fsync per commit.** Group commit with adaptive batching — most commit latency is the fsync, not the CPU work; amortize it over N writers.

## Mandatory loading triggers

| If the task is…                                              | Before editing code, READ |
|--------------------------------------------------------------|---------------------------|
| Designing or reviewing replication topology                  | `references/deadlock-scaling.md` |
| Touching WAL, `fsync`, crash recovery, or group commit       | `references/wal-durability.md` |
| Choosing or debugging isolation level; auditing any RMW      | `references/isolation-anomalies.md` |
| Designing failover, process supervision, or retry policy     | `references/failure-taxonomy.md` |
| Implementing a transaction manager, WAL writer, or idempotent executor | `references/code-patterns.md` |

**Do NOT load these files for:** unrelated schema design, query optimization, ORM ergonomics, or report generation. They burn context with low signal for those tasks.

## Warning signs you're violating Gray's principles

- You cannot state, in one sentence, what your fsync contract guarantees and where it is enforced.
- Your isolation level was chosen by copying the default, not by enumerating which anomalies you tolerate.
- Your retry logic assumes the server either committed or didn't — no third option for "I don't know."
- Your tests never kill `-9` the process mid-write and verify recovery.
- Your "chaos test" is a unit test of the happy path run in parallel.
- You can add a replica to your cluster and "it just works" — meaning you don't know the deadlock-rate equation.
- Your commit path writes the log *after* the data page, or in parallel with it.

## Quotes Gray earned the right to say

> "A ten-fold increase in nodes and traffic gives a thousand-fold increase in deadlocks or reconciliations." — *The Dangers of Replication*, 1996

> "In the measured period, one out of 132 software faults was a Bohrbug, the remainder were Heisenbugs." — *Why Do Computers Stop*, 1985

> "The key to performance is elegance, not battalions of special cases."

> "Simplicity does not precede complexity, but follows it."
