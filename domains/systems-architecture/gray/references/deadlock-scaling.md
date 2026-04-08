# Replication scaling: the N³ deadlock law

Source: Gray, Helland, O'Neil, Shasha. *The Dangers of Replication and a Solution*. SIGMOD 1996.

The paper every engineer designing "multi-master" replication must read before they start. Gray's model is simple and its conclusion is brutal: naive eager update-everywhere replication does not scale, it **collapses**.

## The model

A fixed database of `DB_size` objects, replicated on `N` nodes. Each node originates `TPS` transactions/sec; each transaction performs `Actions` writes at `Action_time` per write. Objects are chosen uniformly.

## Single-node baseline

Per-transaction deadlock probability:

    PD ≈ (TPS · Action_time · Actions⁵) / (4 · DB_size²)

The fifth power of `Actions` is the first thing that should terrify you. **Doubling transaction size produces a 32× increase in deadlock rate** on a single node. This is why Gray's `NEVER do long transactions` is not style advice — it is the dominant term.

## Eager update-everywhere (the scary case)

In an eager scheme, every transaction locks its `Actions` writes on every one of the `N` replicas, so effective transaction size becomes `Actions × N` and transaction duration grows linearly with `N`. Substituting into the single-node formula and aggregating across nodes:

    Eager_Deadlock_Rate ≈ (TPS² · Action_time · Actions⁵ · N³) / (4 · DB_size²)

**The consequence that practitioners miss:**
- 10× nodes ⇒ **1000× deadlocks** (cubic in N)
- 2× transaction size ⇒ **32× deadlocks** (fifth power in Actions)
- 2× TPS per node ⇒ **4× deadlocks** (quadratic in TPS)

If the database size scales linearly with N (as in TPC-A/B/C benchmarks), the exponent on N drops from 3 to 1 — still unstable, but survivable for a while. If it doesn't, you hit a wall.

## Lazy-group replication: out of the frying pan…

Lazy-group schemes (update locally, ship to replicas asynchronously) don't deadlock but they **reconcile**. Reconciliation rate scales similarly, and failed reconciliations mean each node's database diverges from the others permanently. Gray's term for the endgame: **system delusion** — "the database is inconsistent and there is no obvious way to repair it."

Lotus Notes's convergence model is lazy-group done deliberately, but it is **not** serializable. It works only because the application semantics (append-only + timestamped replace) are commutative and idempotent. If your app doesn't fit that mold, don't use it.

## Lazy-master: the actually-reasonable option

One master per object, lazy propagation to replicas. Deadlock rate:

    Lazy_Master_Deadlock_Rate ≈ (TPS · Actions⁴ · N²) / (4 · DB_size)

Quadratic in N, not cubic. This is the scheme most successful systems pick (PostgreSQL streaming replication, MySQL async replication, MongoDB with one primary per shard). The cost: a single-leader bottleneck and a failover story you must design.

## Two-tier replication (Gray's proposed solution)

For disconnected/mobile scenarios where lazy-master alone doesn't cut it:

1. **Base nodes** hold the master copy and run serializable transactions.
2. **Mobile nodes** accept *tentative* transactions while disconnected, displaying tentative results locally.
3. On reconnect, tentative transactions are re-executed against the master as **base transactions** with an explicit *acceptance criterion* (e.g., "balance still ≥ 0" or "bit-identical results").
4. Tentative transactions that fail acceptance are rolled back and a diagnostic is shown to the user.
5. Designed so that most updates are **commutative** (increments, appends), which eliminates most reconciliation.

The key insight: reality's replication schemes (checkbooks, phone books, Git, banking) work exactly this way. The user proposes, the master disposes.

## Decision heuristics from the math

- **≤3 nodes, short transactions (≤5 writes), rare cross-partition writes** → eager is tolerable.
- **>3 nodes, any workload with RMW** → primary-copy lazy replication.
- **Mobile / disconnected nodes** → two-tier with commutative updates.
- **Any scheme with >10 nodes and frequent writes** → you *must* partition ownership; there is no replication algorithm that will save you.

## Concrete anti-patterns

- **Galera, Group Replication, or any "synchronous multi-master" at >5 nodes with >5 writes per transaction.** The math says you will hit the wall. The wall is there.
- **"Active-active" across two datacenters with the same write keys.** This is lazy-group or eager depending on the day. Either way: system delusion in slow motion. Use active-passive with automated failover, or partition ownership by key (each DC owns half the keyspace).
- **Retry loops on deadlock that don't exponentially back off *and* don't shrink transaction size.** You just converted deadlocks into livelocks.

## What to measure

If you run a replicated system, these metrics are the early-warning signals:
- **Deadlock rate per node** (Postgres: `pg_stat_database.deadlocks`). Track the derivative, not the absolute. A rising slope means your workload or topology changed.
- **Reconciliation / conflict-resolution rate** (for lazy schemes). Same story — track the slope.
- **p99 transaction duration.** The fifth-power `Actions` dependency means a workload that drifts from 3 writes/txn to 6 writes/txn silently increases deadlock rate by 32×.
- **Replica lag** for lazy-master. When lag exceeds the window in which a client might read-your-writes, you have a correctness bug waiting to happen.
