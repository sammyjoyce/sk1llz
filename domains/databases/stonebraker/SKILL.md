---
name: stonebraker-database-architecture
description: >-
  Design database systems applying Michael Stonebraker's architectural principles:
  workload-specific engine selection, OLTP/OLAP separation, main-memory optimization,
  shared-nothing partitioning, and extensible type systems. Use when making fundamental
  database architecture decisions, choosing between row-store and column-store,
  designing partitioning schemes, evaluating NewSQL vs traditional RDBMS, building
  storage engines, or deciding whether to specialize or use a general-purpose database.
  Use when user mentions "database architecture", "OLTP vs OLAP", "column store",
  "partition strategy", "VoltDB", "Vertica", "H-Store", "C-Store", "storage engine design",
  "one size fits all", or "specialized database".
---

# Stonebraker Database Architecture

## Thinking Framework

Before any database architecture decision, ask yourself:

1. **What is the actual workload shape?** Count the ratio of reads to writes, average
   rows touched per query, and concurrency level. A workload touching <100 rows per
   transaction with >10K TPS is OLTP. Scanning >10K rows with aggregations is OLAP.
   Mixing them in one engine is the #1 architectural mistake.

2. **Where does the overhead actually live?** In a traditional RDBMS under OLTP load,
   the 2008 "Looking Glass" study found useful work was <12% of CPU time—the rest was
   buffer management, locking, latching, and logging. The 2025 follow-up found that
   after eliminating those (as VoltDB does), communication/networking becomes the new
   bottleneck: 51–68% of server CPU in stored-procedure mode, higher with client-side
   transactions. Know which era your system lives in before optimizing.

3. **Can you partition such that >95% of transactions are single-partition?** If yes,
   single-threaded partitions (H-Store model) eliminate locking entirely. If multi-partition
   transactions exceed ~10%, performance collapses because they serialize through a
   global coordinator—only one multi-partition transaction executes at a time across
   the entire cluster.

4. **Is this a data-fits-in-memory problem?** If your working set fits in RAM, disk-oriented
   structures (buffer pool, page-level locking, ARIES recovery) are pure overhead. But
   if data exceeds memory, anti-caching or tiered storage is essential—don't pretend
   everything is in-memory when it isn't.

## Decision Tree: Choosing Architecture

```
Workload?
├─ OLTP (short txns, point lookups, high concurrency)
│  ├─ Data fits in memory? → Main-memory engine, no buffer pool
│  │  ├─ >95% single-partition? → Single-threaded partitions (H-Store/VoltDB model)
│  │  └─ Multi-partition heavy? → Shared-memory MVCC (Hekaton/Silo model)
│  └─ Data exceeds memory? → Disk-based with anti-caching or tiered storage
│
├─ OLAP (aggregations, scans, few writers)
│  ├─ Queries touch <30% of columns? → Column store (C-Store/Vertica model)
│  ├─ Queries touch >70% of columns? → Row store may match column store
│  └─ Mixed: wide tables, selective columns → Column store wins decisively
│
├─ HTAP (real-time analytics on operational data)
│  └─ CAUTION: True HTAP is an unsolved problem. Separate engines with
│     CDC replication is more predictable than any single-engine HTAP.
│     The physics of optimization for both is fundamentally adversarial.
│
└─ Streaming / Scientific / Array
   └─ Different data models entirely. Don't force relational.
```

## Expert Knowledge: What Takes Years to Learn

### The OLTP Overhead Taxonomy (Quantified)

Stonebraker's "Looking Glass" (2008) decomposed where CPU time goes in a traditional
RDBMS running TPC-C. The numbers that matter:

- **Buffer pool management**: ~35% of CPU (page mapping, pin/unpin, replacement)
- **Locking**: ~25% (lock table, deadlock detection, lock waits)
- **Latching**: ~15% (short-term mutual exclusion on internal structures)
- **Logging (WAL)**: ~15% (serializing log writes, group commit, fsync)
- **Useful work**: ~10% (actually executing queries)

H-Store/VoltDB eliminated the first four by going main-memory + single-threaded
partitions + deterministic execution + command logging instead of WAL. Result: 82×
faster on TPC-C. But the 2025 "Looking Glass 2.0" (Stonebraker et al., CIDR 2025)
revealed the *new* bottleneck: VoltDB spends only 23% of CPU on transaction processing;
~40% goes to internal DBMS networking, ~38% to Linux kernel networking. Kernel bypass
(DPDK/F-Stack) reclaims this, pushing useful work from 33% to 54% on YCSB.

### Column Store Crossover Points

Column stores don't always win for analytics. The crossover depends on table width:

- **<5 columns in table**: Row store and column store perform similarly
- **Query touches >70% of a wide table's columns**: Row store can match or beat
  column store because you're reconstructing tuples anyway
- **Sweet spot for column stores**: Wide tables (20+ columns) where queries touch
  <30% of columns. Here compression + skip-scan gives 10–100× advantage
- **C-Store's projection trick**: Store overlapping sorted projections, not just
  columns. Each projection is a subset of columns sorted on a chosen key.
  The optimizer picks the projection whose sort order best matches the query.
  Maintaining multiple projections costs write amplification—budget 2–4× storage

### The Multi-Partition Transaction Cliff

In VoltDB/H-Store, multi-partition transactions serialize globally. Practical thresholds:

- **<5% multi-partition**: System performs well, single-partition throughput dominates
- **5–15%**: Noticeable degradation; multi-partition becomes the scheduling bottleneck
- **>15%**: System effectively serializes; throughput collapses to single-threaded speed

The database designer's job is to find a partitioning key that keeps cross-partition
work below 5%. For e-commerce: partition by customer_id, not order_id (because
payment transactions join customer and warehouse—partition on the anchor entity).
When you can't partition cleanly, consider replicating small read-only tables to every
partition to convert joins into local lookups.

### C-Store's Write Store / Read Store Architecture

C-Store maintains two physical stores: a Write Store (WS) optimized for inserts
(row-oriented, uncompressed) and a Read Store (RS) optimized for queries
(column-oriented, compressed, sorted). A Tuple Mover periodically migrates WS→RS.

Practitioner traps:
- **Tuple Mover stalls**: If write rate exceeds Tuple Mover throughput, WS grows
  unboundedly and query performance degrades because the executor must merge
  WS and RS results. Monitor WS size as a health metric.
- **Projection maintenance cost**: Each write must update ALL projections. N projections
  means N× write amplification. Keep projection count to 3–5 per table maximum.
- **Join index fragility**: Join indexes connecting projections are expensive to maintain
  under updates. Each modification to a projection requires updating every join
  index that points into or out of it.

## NEVER

- **NEVER run OLAP queries against your OLTP database** "just for now." A single
  analytical scan holding locks or consuming buffer pool pages will spike p99 latency
  for all OLTP transactions. Stonebraker calls this the cardinal sin. Even read-committed
  isolation causes cache pollution that degrades OLTP for minutes afterward.

- **NEVER assume HTAP solves the separation problem.** HTAP systems (TiDB,
  AlloyDB, SingleStore) use internal replication from row-store to column-store replicas.
  This is the same architecture as separate engines + CDC—just hidden. You still pay
  for replication lag, and the column replica's resource consumption can impact the
  OLTP engine through shared memory/CPU. Prefer explicit separation where you
  control the blast radius.

- **NEVER design a partitioning scheme without profiling your actual transaction mix.**
  The seductive path is to partition by primary key. But if 20% of transactions join
  across the partition boundary, you've built a system that serializes 20% of throughput.
  Profile first, then partition on the key that minimizes cross-partition transactions.

- **NEVER use ad-hoc SQL in a high-throughput OLTP system.** Stored procedures
  (or pre-defined transaction classes) allow the system to pre-plan execution,
  determine partition routing at compile time, and eliminate per-query optimization
  overhead. Ad-hoc SQL in OLTP is a 2–5× throughput penalty from parsing,
  planning, and extra client-server round trips.

- **NEVER add a general-purpose index to a column store "just in case."** B-tree
  indexes on column stores add write amplification without benefiting scan-heavy
  workloads. Column stores achieve selectivity through sorted projections, zone maps,
  and min/max pruning—not traditional indexes.

- **NEVER ignore the network stack in a modern OLTP engine.** Once you eliminate
  buffer pool/locking/latching overhead, 50–70% of CPU goes to networking. If you
  benchmark only the query engine, you're measuring the wrong bottleneck. Measure
  end-to-end including serialization, socket I/O, and kernel overhead.

## Stonebraker's Postgres Extensibility Insight

Postgres survived 40+ years because of one architectural bet: user-defined types,
operators, index methods, and procedural languages as first-class citizens. This enabled
PostGIS, JSONB, pgvector, and hundreds of extensions without forking the engine.

The principle: **build extensibility at the type system level, not at the query level.**
If you let users define new types + operators + index access methods, the optimizer
and executor handle them automatically. This is cheaper than special-casing every
new data type in the query engine. When designing a new database system, the
question isn't "what types should we support?" but "how do we let users add types
we haven't imagined?"

## Fallback Strategies

| Primary approach fails | Fallback |
|---|---|
| Partitioning can't get <5% cross-partition | Replicate hot dimension tables to all partitions; or switch to shared-memory MVCC |
| Column store write throughput insufficient | Add a row-oriented write buffer (WS/RS pattern); batch writes into larger appends |
| Main-memory engine exceeds RAM | Anti-caching: evict cold tuples to SSD, fetch on demand; or use tiered storage with access-frequency tracking |
| Stored procedures too rigid for evolving schema | Use parameterized query templates with pre-compiled plans; avoid full ad-hoc but allow controlled flexibility |
| Deterministic execution can't handle external calls | Isolate non-deterministic operations (e.g., current_timestamp, random) into a pre-processing step that resolves them before entering the deterministic engine |
