---
name: pavlo-database-performance
description: >
  Optimize databases in the style of Andy Pavlo, CMU professor and database
  systems researcher. Applies hard-won lessons from NoisePage, OtterTune,
  Peloton, and 15-721 on query compilation trade-offs, MVCC garbage collection
  pitfalls, index selection mistakes, buffer pool diagnostics, and storage
  engine architecture decisions. Use when tuning query performance, choosing
  between B-tree and LSM-tree, evaluating vectorized vs compiled execution,
  diagnosing MVCC-induced slowdowns, benchmarking databases, designing storage
  engines, selecting concurrency control schemes, or evaluating database
  technologies. Trigger keywords: query optimization, EXPLAIN ANALYZE,
  buffer pool, write amplification, cardinality estimation, lock-free index,
  HTAP, self-driving database, OtterTune, BenchBase.
---

# Pavlo-Style Database Performance

## Thinking Framework

Before touching any database performance problem, ask yourself:

1. **Where is wall-clock time actually going?** Not CPU vs I/O — decompose into: buffer pool misses, lock waits, log flushes, network round-trips, GC stalls, or cardinality misestimates causing wrong join order. Most "slow query" issues are actually buffer pool or cardinality issues masquerading as something else.

2. **Am I measuring the right thing correctly?** Averages lie. Report p50/p99/p999. Warm the cache first. Use realistic skew — uniform random is never realistic. Run long enough for GC and compaction to kick in (60-second benchmarks hide the worst problems — Peloton degraded from 100K to 6 TPS over months because nobody ran long benchmarks).

3. **Does the architecture match the workload's access pattern?** The best optimization is choosing the right architecture. No amount of index tuning saves a row store doing full-table analytics, or a column store doing point lookups with random inserts.

## Critical Decision Trees

### Vectorized vs Compiled Execution

| Factor | Favors Vectorized | Favors Compiled |
|--------|-------------------|------------------|
| Query complexity | Many joins, complex predicates | Simple filters, point lookups |
| Compilation latency tolerance | None (interactive) | Acceptable (prepared statements) |
| SIMD utilization | High (tight primitives loop) | Low (gains vanish when data > L1 cache) |
| Debugging | Easier (step through primitives) | Harder (generated LLVM IR has no stack trace) |
| Engineering effort | Thousands of pre-compiled primitives | Code-generating compiler |

Key insight from CMU's research: SIMD speedups in vectorized joins nearly vanish once hash tables exceed L1 cache. At TPC-H SF10+, memory latency dominates and SIMD gives <5% improvement on join-heavy queries. Hyper-threading can hide ~50% of the performance gap between the two models.

Practical crossover: NoisePage abandoned HyPer-style direct LLVM IR generation because students couldn't debug it. SingleStore's DSL→opcode→LLVM approach is the pragmatic winner — interpret immediately, compile in background, swap seamlessly.

### B-tree vs LSM-tree Selection

- **B-tree when**: read:write ratio >10:1, need predictable read latency, range scans are primary access pattern, can tolerate random write I/O, dataset fits in buffer pool
- **LSM-tree when**: write:read ratio >1:1, can tolerate write amplification (10-30× is typical for leveled compaction), space amplification budget exists, point lookups dominate reads
- **The trap**: LSM write amplification compounds with compaction — a single long-running OLAP query on an LSM engine (e.g., MyRocks) causes compaction to stall, which causes memtable flushes to stall, which causes write stalls. Budget 2-3× your expected write throughput for compaction headroom.

### MVCC Scheme Selection

A single long-running analytical query can collapse OLTP throughput to near-zero in PostgreSQL, WiredTiger, and InnoDB. This happens because:
- GC cannot reclaim versions visible to the long-running snapshot
- Tombstones accumulate in queue-pattern indexes (like TPC-C's new-order table)
- Version chains grow unbounded, turning O(1) lookups into O(n) chain traversals

Mitigation hierarchy:
1. Separate OLTP snapshot tracking from OLAP (OSIC-style per-thread commit logs)
2. Use a Graveyard Index to move tombstones out of hot paths
3. Adaptive version storage: inline versions for cold tuples, delta chains for hot tuples
4. If none of those are available: physically separate OLTP and OLAP (which is why most orgs still run separate systems despite HTAP promises)

## Anti-Patterns

NEVER trust a lock-free index will outperform a lock-based one. CMU's SIGMOD'18 Bw-Tree paper showed a well-implemented B+Tree with optimistic lock coupling beats the Bw-Tree by 1.5-4.5× on multi-core CPUs. The Bw-Tree's indirection layer and delta chain traversal cause 2× more L3 cache misses. Lock-free sounds faster but the CaS retry storms under contention and cache-line invalidation from delta appends dominate.

NEVER benchmark for only 60 seconds. Peloton's CI ran 60-second tests and showed 1% regressions that were ignored. Over months, it accumulated to 99.99% throughput loss (100K→6 TPS). GC pressure, compaction storms, buffer pool thrashing, and log file growth only manifest over minutes to hours. Minimum credible benchmark: 10 minutes steady-state after warmup, with p99 tails tracked.

NEVER add an index without checking cardinality estimates first. The optimizer may not use your index if its cardinality estimate is off by >10×. In PostgreSQL, multi-column statistics are opt-in (`CREATE STATISTICS`) and disabled by default — without them, the optimizer assumes column independence, which can produce estimates off by 1000× on correlated columns. Check `EXPLAIN (ANALYZE, BUFFERS)` — if `rows=` estimate differs from `actual rows=` by >10×, fix the stats before adding indexes.

NEVER assume bigger buffer pool = faster queries. Past ~80% cache hit ratio, increasing buffer pool size yields diminishing returns. But running mixed OLTP+OLAP on a shared buffer pool causes catastrophic "buffer pool pollution" — the OLAP scan evicts the OLTP hot set, and recovery time after the scan completes scales linearly with buffer pool size. Larger pool = longer recovery.

NEVER use vendor benchmarks without checking: (1) did they flush writes to disk? (SurrealDB reported great numbers without fsync), (2) is the dataset larger than memory? (in-memory results don't predict disk-bound behavior), (3) what percentile are they reporting? (p50 hides tail latency disasters), (4) how long did the benchmark run? (short runs miss GC and compaction effects).

## Knob Tuning Heuristics

From OtterTune's research (improved Oracle performance 50% after expert DBAs had already tuned it):

- Most DBMSs have 100-500 knobs but only 10-15 matter for any given workload
- PostgreSQL's most impactful: `shared_buffers` (start at 25% RAM), `effective_cache_size` (75% RAM), `work_mem` (watch for per-sort multiplication), `random_page_cost` (set to 1.1 for SSD, not the default 4.0)
- The knob that causes the most confusion: PostgreSQL's `maintenance_work_mem` with value `-1` silently inherits from `autovacuum_work_mem`, creating an invisible dependency. Always set both explicitly.
- Knob interactions are non-linear: increasing `shared_buffers` beyond physical RAM causes double-buffering with the OS page cache and can *decrease* performance

## Benchmarking Methodology

| Phase | Duration | Purpose |
|-------|----------|---------|
| Load | Until complete | Populate with realistic data distribution + skew |
| Warmup | 2-5 min | Fill buffer pool, trigger JIT, stabilize caches |
| Ramp | 1-2 min | Gradually increase to target concurrency |
| Steady-state | 10+ min | Actual measurement window |
| Cooldown | 2 min | Catch GC/compaction tail effects |

Report: p50, p99, p999 latency + throughput. Plot latency over time (not just aggregates) — look for periodic spikes from compaction, checkpointing, or autovacuum. If p99/p50 ratio > 10×, you have a tail latency problem that averages will never reveal.

## Fallback Strategies

When EXPLAIN ANALYZE shows a bad plan:
1. First: check if `ANALYZE` has been run recently on all involved tables
2. If stats are fresh but estimates are still wrong: create multi-column statistics or use expression statistics
3. If the optimizer still won't cooperate: use `pg_hint_plan` or CTE materialization fences as last resort — but log every hint so you can remove them when the optimizer improves
4. If it's a parameterized query with plan instability: consider `plan_cache_mode = force_custom_plan` for that specific query

When buffer pool hit ratio drops below 99%:
1. Check for sequential scan flooding (a single `SELECT *` can evict your entire hot set)
2. Verify no long-running transactions holding snapshots open
3. Consider partitioning the hot set into a separate tablespace on faster storage
4. Last resort: increase `shared_buffers`, but only if total buffer pool < 40% of available RAM
