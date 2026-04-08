---
name: stonebraker-database-architecture
description: >-
  Make database architecture calls in Michael Stonebraker's style: classify workloads by dominant
  bottleneck, decide when OLTP and OLAP must be split, choose when shared-nothing and main-memory
  designs actually pay off, and avoid hidden costs in column stores, deterministic OLTP, and
  Postgres-style extensibility. Use when deciding OLTP vs OLAP vs HTAP, choosing a partition key,
  evaluating VoltDB/Vertica/Postgres-like designs, designing a storage engine, or arguing against
  "one database for everything." Triggers: "database architecture", "OLTP vs OLAP", "HTAP",
  "shared nothing", "partition key", "column store", "projection", "tuple mover", "VoltDB",
  "Vertica", "Postgres extensibility", "anti-caching", "storage engine".
---

# Stonebraker Database Architecture⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​​​‌​​‍​​​‌‌​‌‌‍​​‌​‌​​​‍​‌‌‌​‌‌‌‍​​​​‌​​‌‍​​​‌​‌‌​⁠‍⁠

Stonebraker's move is not "tune harder." It is "identify the overhead the incumbent system treats as normal, then delete the subsystem that exists only to pay that overhead." The right first question is never "which database is best?" It is "what is the dominant tax here: coordination, scans, memory misses, write amplification, or lack of extensibility?"

## Use Order

- Use this skill standalone for first-pass architecture decisions.
- Do not dive into vendor tuning guides, ORM advice, or index cookbooks before you finish the tests below. Those docs optimize a chosen architecture; they do not tell you whether the architecture is wrong.

## Before You Commit

Before choosing an engine or topology, ask yourself:

- Where does CPU go after you remove the old bottleneck? In the original "Looking Glass," classic OLTP wasted most instructions on buffer management, locking, latching, and logging. In "Looking Glass 2.0," VoltDB-style systems moved the pain to communication: stored-procedure mode still spent 23%/51%/68% of CPU on communication for TPC-C/Voter/YCSB-C, and client-driven mode rose to 47%/63%/68%.
- Are transactions routable before execution starts? H-Store gets its economics only when input parameters tell you the partition(s) up front and the procedure set is known before deployment. If procedures arrive after deployment, the physical design is already misaligned.
- Are you optimizing for a bounded historical window or pretending "time travel" is free? C-Store's HWM/LWM split exists because near-now history consumes write-store space and delays cleanup.
- Do indexes stay memory-resident even when base data does not? H-Store anti-caching assumes they do. If large secondary indexes spill, the design premise changes.
- Is the hot spot semantic or accidental? Shared-nothing does not remove hot rows. Stonebraker's answer is usually to delete the hot aggregate or split it into N subobjects, not to buy a different cluster shape.

## Decision Rules

- If the workload is repetitive OLTP and requests can be routed from their input parameters to one partition, prefer main-memory shared-nothing with predefined transaction classes. The gain is not "RAM is fast"; it is deleting buffer-pool, latch, and lock-manager work.
- If the same product must support ad hoc analytical scans, do not hide behind "HTAP." Stonebraker's claim is that OLTP and OLAP want adversarial physical designs: row-oriented, coordination-minimized execution versus reordered, compressed, scan-oriented execution. Treat them as separate engines even if they share a parser or SQL surface.
- If analytics read a small subset of wide rows, prefer columnar projections, not "a row store plus more indexes." C-Store wins when sort order, encoding, and operators all align around the same scan patterns.
- If queries routinely touch most columns, or the table is narrow, columnar advantage shrinks because tuple reconstruction and join-index work start to dominate.
- If the product needs new types, temporal semantics, spatial operators, or custom indexes, copy Postgres's seam: new types, operators, operator classes, and access methods. Do not keep adding planner special cases one feature at a time.

## Procedures That Matter

### 1. Partitionability Test

1. List the top transaction classes, not the biggest tables.
2. For each class, mark the routing key available before the first statement runs.
3. If touched partitions cannot be inferred from the call boundary, deterministic single-partition execution is the wrong bet.
4. For every cross-partition step, ask whether duplicating a small reference table or moving a derived summary upstream would make it local.
5. If the remaining distributed work is still central to the product, stop asking "how do we shard this?" and ask "why is this service boundary forcing coordination?"

### 2. Column-Store Reality Check

1. Enumerate the 3-5 query shapes that actually matter; every extra projection multiplies write cost.
2. Choose projection sort orders for pruning and compression, not to mimic OLTP indexes.
3. Verify that operators can run on compressed data. If the executor inflates early, most of the architectural gain is already lost.
4. Budget tuple-mover headroom. WS->RS movement also updates delete metadata, storage keys, and join indexes; if write-store growth outruns merge-out, the engine degrades into a dual-format liability.
5. Bound historical visibility. LWM chases HWM because "recent as-of queries" are not free; they delay cleanup and bloat the write store.

### 3. Extensibility Seam Test

1. When a team asks for a new data type, ask whether it needs a new planner branch or just a type + operators + operator class + access method.
2. Prefer indirection that avoids secondary-index churn on ordinary updates. POSTGRES anchor pointers let rows move and delta chains grow without rewriting every index entry.
3. Only choose no-overwrite / append-only internals if you are also willing to own vacuum economics. Instant recovery is the sales pitch; background cleanup debt is the bill.

## Numbers That Change Decisions

- C-Store's published prototype stored the benchmark at 1.987 GB where the compared row store needed 4.480 GB and another column store needed 2.650 GB. Redundant projections can still win on space when sort order and encoding are chosen together.
- C-Store's Type-2 bitmap encoding makes selection cheap, but operators may need one memory page per possible value. A column that looked "few-valued" in design review can become a memory sink when cardinality drifts.
- In H-Store anti-caching, sampled LRU stayed within roughly 2-7% of baseline overhead, while full-LRU tracking cost materially more. Approximate coldness unless the access pattern is nearly flat.
- Anti-caching beat tuned MySQL by 9x/18x/10x on skewed YCSB read-only/read-heavy/write-heavy workloads with data 8x memory size, but that result depended on skew and memory-resident indexes. This is a skew play, not a universal oversubscription strategy.
- Looking Glass 2.0 showed DPDK/F-Stack raising VoltDB's transaction-processing share from 33%/48%/70% to 53.8%/72%/91% on TPC-C/Voter/YCSB-C. After you delete disk-era overheads, the network stack becomes architecture, not plumbing.

## NEVER

- NEVER keep OLTP and analytics in one engine because the seductive story is "one source of truth." The concrete result is that scan costs and coordination costs share the same blast radius. Instead split by bottleneck and make replication or CDC an explicit seam.
- NEVER choose shared-nothing because "it scales linearly" while skipping placement work. The seductive path is buying nodes before proving locality. The consequence is distributed commit machinery plus unchanged hot spots. Instead prove routing keys, hotspot strategy, and local-transaction dominance first.
- NEVER benchmark a column store on read-only RS behavior and declare victory. The seductive path is celebrating scan numbers before tuple-mover, delete-vector, and join-index pressure show up. The consequence is a warehouse that wins demos and loses once writes age. Instead capacity-plan WS->RS merge throughput as a first-class number.
- NEVER treat stored procedures as a cosmetic API choice. The seductive path is keeping client-side transaction logic for debuggability while expecting deterministic OLTP economics. The consequence is that communication becomes the new bottleneck even after disk-era taxes are gone. Instead decide explicitly whether you want stored-procedure locality or client-side flexibility.
- NEVER add projections or indexes "just in case." The seductive path is covering every imaginable query. The consequence is multiplicative write amplification and brittle maintenance paths. Instead keep only the few orderings that materially change pruning or compression.
- NEVER promise append-only history "for free." The seductive path is instant recovery and time travel. The consequence is vacuum debt, longer delta chains, and archival/index maintenance that somebody must eventually pay. Instead bound retention windows and fund the cleanup path.

## If The First Choice Fails

- If locality fails, first replicate tiny read-mostly dimensions and delete synthetic hot rows; if that still leaves unpredictable multi-partition work, move to a design that expects coordination instead of hiding it.
- If the column store cannot keep up with writes, reduce projection count before touching codecs; projection entropy usually hurts more than imperfect compression.
- If anti-caching misses too often, stop tuning eviction policy first and re-check whether the working-set assumption is false. Anti-caching is for skewed access, not uniform churn.
- If the organization insists on "one database product," keep a common parser or SQL layer and specialize the execution engines underneath. Stonebraker's compromise is shared front-end, fractured back-end.
