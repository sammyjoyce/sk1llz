---
name: pavlo-database-performance
description: >-
  Diagnose and tune real database performance problems using Andy Pavlo-style
  heuristics for benchmarking, optimizer pathologies, MVCC version retention,
  buffer managers, and storage-engine trade-offs. Use when a query is slow for
  non-obvious reasons, when EXPLAIN ANALYZE and observed latency disagree, when
  choosing B-tree vs LSM or mmap vs buffer pool, when a mixed OLTP/OLAP workload
  regresses over time, or when benchmarking/tuning PostgreSQL, MySQL/InnoDB,
  RocksDB/MyRocks, NoisePage-style engines, or database internals. Trigger
  keywords: EXPLAIN ANALYZE, cardinality estimation, CREATE STATISTICS,
  shared_buffers, work_mem, autovacuum, backend_xmin, replication slots,
  compaction debt, write amplification, B-tree, Bw-Tree, optimistic lock
  coupling, mmap, TLB shootdown, query compilation, vectorized execution,
  BenchBase, OtterTune.
---

# Pavlo-Style Database Performance⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​‌‌‌​​‍‌​‌‌​​‌​‍‌​​​‌‌​‌‍​‌‌‌​​​​‍​​​​‌​‌​‍​‌‌​​​‌‌⁠‍⁠

This skill is self-contained. Do not waste context on generic SQL primers, "database basics", or blanket index checklists.

## Core Stance

- Treat every regression as one of five failures: bad estimates, bad locality, bad concurrency, bad background maintenance, or bad benchmarking.
- Prefer proof over folklore. `EXPLAIN (ANALYZE, BUFFERS)`, wait events, cache misses, vacuum/compaction debt, and latency-over-time plots beat "CPU is high".
- A fast p50 with a bad p99 is usually a scheduler, flush, or maintenance problem, not a plan problem.
- For design choices, optimize the dominant failure mode of the workload, not the median case.

## Before Doing Anything, Ask Yourself

1. Is the system saturated, or did the benchmark hide queueing? Closed-loop clients stop issuing work while waiting; use open-loop or centrally rate-controlled load so backlog stays visible.
2. Did performance change immediately or decay over time? Immediate regressions are usually plan, estimate, or locality mistakes; slow decay usually means MVCC garbage, compaction debt, checkpoint pressure, or cache pollution.
3. Is the objective throughput, tail latency, or cost per query? Different winners emerge; the fastest point-lookup engine can still be wrong if range scans or write stalls dominate.
4. Is the working set smaller than DRAM, smaller than the OS page cache, or larger than memory? That answer changes whether you are debugging planner math, buffer policy, or storage behavior.
5. Which background task is allowed to interrupt you? PostgreSQL autovacuum/checkpoints, RocksDB compaction, GC, WAL archiving, and snapshot retention are first-class actors, not noise.

## Diagnosis Order

### If one query regressed and the rest are fine

- First compare estimated rows vs actual rows. If any major node is off by more than 10x, assume a statistics problem before an index problem.
- On PostgreSQL, correlated predicates and multi-column `GROUP BY` are classic traps. The planner assumes independence until you create multivariate stats; the docs show simple cases where row counts are wrong by 100x until `CREATE STATISTICS ... (dependencies, ndistinct)` is added.
- If estimates are good but buffers explode, the problem is locality, not the optimizer.

### If p99 spikes but p50 stays flat

- Look for maintenance pulses, not SQL text: checkpoints, WAL flush storms, autovacuum/index cleanup, compaction bursts, or log fsync batching.
- Plot latency over time. A percentile without a time axis hides periodic stalls.
- If the slowdown happens at the same wall-clock time every day, assume a scheduled maintenance job before blaming the workload.

### If throughput degrades over minutes or hours

- Suspect retained versions or compaction debt.
- On PostgreSQL, inspect `backend_xmin`, prepared transactions, and replication slots before touching storage settings; long snapshots pin dead tuples and keep vacuum from reclaiming space.
- On LSM engines, rising L0 files or down-compact bytes means the engine is paying yesterday's writes with today's tail latency.

### If mixed OLTP and OLAP hurts the hot path

- Assume buffer-pool pollution until proven otherwise.
- The real fix is usually workload separation, admission control, or scan isolation, not "bigger cache".
- In PostgreSQL, increasing `shared_buffers` after the hot set already fits can lengthen recovery from a bad scan because more cold pages remain resident and churn out slowly.

## Heuristics That Experts Actually Use

### Benchmarking

- Do not trust 60-second runs. BenchPress exists because even small throughput oscillations make anomalies hard to interpret, and rate control must be precise.
- Minimum credible pattern: warm until caches and JIT stabilize, ramp to target, then measure steady state for at least 10 minutes. Engines with GC, compaction, or vacuum side effects need longer.
- Preserve the arrival process when comparing systems. If one system cannot keep up, postponed work should stay visible; otherwise you benchmark the client's patience instead of the DBMS.
- If a 10-minute production trace turns into a 45-minute replay under a bad configuration, abort early and score it as a failure. OtterTune needed this because bad knob settings stretched replay time by hours.

### PostgreSQL Tuning

- `shared_buffers`: start around 25% of RAM on a dedicated host; going past 40% rarely wins because PostgreSQL still leans on the OS cache, and larger settings usually require a larger `max_wal_size` to avoid violent writeback.
- `work_mem` is per sort or hash node, not per query. Hash operations can use `work_mem * hash_mem_multiplier` (default 2.0). Global increases are dangerous because concurrency multiplies memory faster than most people estimate.
- The subtle knob trap is `autovacuum_work_mem = -1`, which means "inherit `maintenance_work_mem`". A harmless-looking increase to `maintenance_work_mem` can multiply across `autovacuum_max_workers`.
- If you want fewer sort spills, first ask whether the spill is local and acceptable. Raising `work_mem` globally to save one report query is a common way to trade temp I/O for OOM risk.

### LSM vs B-tree

- When read tail matters more than ingest averages, B-trees win because they do not defer today's write cost into tomorrow's compaction debt.
- Use LSM only when sustained write rate is the dominant requirement and you have CPU plus I/O headroom for compaction. If steady ingest already runs near device limits, future compaction stalls are not a surprise; they are deferred billing.
- In RocksDB leveled compaction, `level_compaction_dynamic_level_bytes=true` is the recommended default since 8.4 because it keeps the tree stable and puts about 90% of data in the last level. If the tree is mis-sized, upper levels grow too fat and compaction work migrates upward at exactly the wrong time.
- A long scan over tombstone-heavy key ranges is where "great write throughput" often cashes out as miserable read amplification.

### Compilation vs Vectorization

- Full JIT recompilation is the wrong hammer for short or unstable queries. Pavlo's group measured complex query compilation in the several-hundred-millisecond range; that is enough to erase gains when distributions or concurrency shift between invocations.
- If selectivities or hot keys move during execution, prefer compile-once, adapt-in-place techniques or hybrid engines that mix precompiled vectorized primitives with compiled pipelines.
- Query compilation is a locality play as much as an instruction-count play. If the plan still chases cold hash tables or remote NUMA pages, fancy codegen will not save it.

### Concurrency Control and Indexes

- Lock-free is not automatically faster. CMU's OpenBw-Tree beat the original Bw-Tree, but it still lagged B+Tree plus optimistic lock coupling; removing delta updates improved inserts 40%, and removing the mapping table cut L3 cache misses 52%.
- Use optimistic lock coupling as the baseline concurrent ordered index until measurements disprove it. It is simpler, more predictable, and often faster than "latch-free" designs once cache behavior is counted.

## Anti-Patterns

- NEVER add an index first because bad row estimates make the optimizer ignore or misuse it, and you pay permanent write amplification for a statistics bug. Instead compare estimated vs actual rows, fix `ANALYZE` coverage, and add multivariate stats when predicates are correlated.
- NEVER trust a closed-loop benchmark because it is seductive to reuse app clients, but once latency rises they issue less work and hide saturation. Instead use open-loop or explicit rate control with backlog visibility.
- NEVER reach for `VACUUM FULL` during a live bloat incident because it visibly shrinks files, but it takes `ACCESS EXCLUSIVE`, rewrites the table, and often worsens the outage. Instead remove blockers (`backend_xmin`, prepared xacts, stale replication slots), run plain `VACUUM`, and rewrite only as planned maintenance.
- NEVER treat `mmap` as a cheap replacement for a buffer pool because the code looks tiny and the OS seems smarter than you. On larger-than-memory workloads with fast NVMe, page-table contention, single-threaded eviction, hidden page faults, and TLB shootdowns dominate. Instead use an explicit buffer manager, `pread`, or `O_DIRECT`, except for narrow read-mostly embedded cases with a known-hot working set.
- NEVER lower `random_page_cost` or raise `work_mem` globally just because the storage is SSD. That shortcut is seductive, but cloud/network storage, cold data, and concurrency often invalidate the assumption. Instead validate with buffer-hit data, temp spill counts, and worst-case memory math.
- NEVER celebrate "no lock" as a performance result because the marketing story is exciting, but delta chains, indirection layers, and compare-and-swap retries still burn cycles and cache lines. Instead compare against optimistic lock coupling using perf counters, not adjectives.

## Fallback Playbooks

### When the optimizer still chooses a bad plan after fresh stats

- Force the smallest reproducible case.
- Reduce the query to the smallest join or predicate set that preserves the bad estimate.
- If correlated columns are involved, create multivariate stats.
- If estimates are good and the plan is still bad, use a targeted hint or plan fence as a temporary quarantine and document why it exists.

### When cache-hit ratio looks healthy but latency is bad

- Stop staring at cache-hit ratio. It can look fine while tail latency is terrible.
- Check wait events, fsync/checkpoint cadence, temp-file growth, and background maintenance.
- If the workload is mixed, isolate scans before resizing caches.

### When autotuning or ML tuning produces nonsense

- Abort configurations that make a trace run several multiples longer than baseline.
- Normalize or penalize failed runs explicitly; otherwise the tuner learns from junk.
- Audit hidden knob dependencies before expanding the search space. OtterTune hit real failures from cross-knob constraints and from cache sizes that passed startup but crashed during replay.

## Done Means

- You can name the dominant bottleneck class.
- You have one metric that proves it.
- You have one change that attacks the bottleneck directly, and one fallback if it fails.
- You did not "fix" a measurement problem with a configuration change.
