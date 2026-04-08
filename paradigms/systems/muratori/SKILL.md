---
name: muratori-performance-aware
description: >-
  Apply Casey Muratori-style performance reasoning to CPU hot paths by
  separating wasted work from machine limits, triaging front-end vs bad
  speculation vs memory stalls, and choosing data-oriented rewrites only when
  counters justify them. Use when optimizing tight loops, devirtualizing hot
  dispatch, diagnosing cache/TLB/store-forwarding/4K-aliasing issues, fixing
  false sharing, or deciding whether branchless, SIMD, or threading changes
  will really help; triggers: muratori, performance-aware, hotspot, top-down,
  cmov, branch mispredict, hot/cold split, false sharing, store forwarding, 4k
  aliasing, memory wall, data-oriented.
tags: performance, profiling, cache, false-sharing, optimization, low-level, systems, data-oriented
---

Use this skill only when the bottleneck is plausibly CPU or memory-system behavior. If the problem is network latency, I/O waits, allocator churn outside the hot path, or API shape, use a different skill.

# Muratori Performance-Aware

This skill is intentionally self-contained. Do not go hunting for generic profiling or clean-code guidance first; the value here is fast triage on the failure modes people usually learn only after months of counter work.

## Core stance

Muratori-style optimization starts by separating two questions that teams blur together:

1. Are you doing work that never needed to exist?
2. For the work that must exist, what exact hardware limit is stopping it?

If you cannot answer both with measurements, you are not optimizing yet.

## Before touching code, ask yourself

- What is the governing metric: frame time, throughput, tail latency, queue depth, or core-seconds per unit of work?
- Is the program slow because of extra instructions, a long dependency chain, or bytes moving farther than they should?
- If I remove this abstraction, am I deleting real work or just making the source look more "manual"?
- What would silently get worse if I win the benchmark: p99, frequency scaling, code locality, cache footprint, NUMA behavior, or correctness assumptions?
- Am I about to "improve" the wrong layer because the profiler only showed time, not the reason for the time?

## Fast triage tree

1. First ask whether the hot path is dominated by needless work.
   If yes, delete the abstraction before micro-tuning. Muratori's biggest wins come from removing pointer chasing, type erasure, redundant passes, and dynamic dispatch that should never have existed in the hot loop.
2. If the work is necessary, classify the bottleneck.
   Use top-down style buckets: Front-End, Bad Speculation, Back-End Memory, Back-End Core, or Contention.
3. Only then choose a rewrite shape.
   Data layout fixes do not repair branch entropy. SIMD does not repair memory stalls. Branchless code does not repair front-end fetch starvation.
4. Stop when the governing metric stops moving.
   Do not keep a "cleaner" low-level rewrite just because it feels more serious.

## Decision heuristics practitioners actually use

### Branches versus branchless code

- A predicted branch is often cheaper than "branchless" code because `cmov` and masked arithmetic extend dependency chains and force both sides of the work into the issue window.
- Use branchless rewrites only when the branch is truly high-entropy and both sides are cheap.
  A practical rule of thumb from optimization work: if prediction is roughly above the 75 percent range, a normal branch often wins; if each side does more than a handful of simple ops, computing both sides usually loses even when mispredicts hurt.
- Keep the hot path as fall-through code.
  Taken branches and cold in-loop blocks cost more than the source looks like. One Easyperf case improved after moving an almost-never-taken normalization path out of the hot fall-through, because front-end slots were being wasted on code that rarely retired.

### Store forwarding and 4K aliasing

- Store-forwarding failures are a classic "everything is in L1 and still slow" trap.
  Intel's tuning guide calls out the common case: a smaller store followed by a larger dependent load within roughly the prior 10 to 15 instructions. The load cannot forward, stalls, and the fix is often to size-match the producer and consumer.
- 4K aliasing is another invisible stall source.
  If a load follows a store at an address offset by 4096 bytes, the pipeline may assume they alias, attempt forwarding, then re-issue when full address resolution proves otherwise. Intel documents about a 7-cycle penalty, and it gets worse for unaligned loads spanning two cache lines.
- Before changing algorithms, check whether a tiny offset or alignment change deletes the entire problem.
  This is exactly the sort of win people miss when they only look at source structure.

### False sharing and cache-line padding

- False sharing is not "two threads touch nearby fields." It is "coherency turns correct code into a line-migration tax."
- Do not blanket-pad structs to 64 bytes.
  That fix is seductive because it feels principled, but it often bloats the working set, increases TLB pressure, and destroys cache density for no gain.
- Pad only after you confirm HITM or equivalent evidence, and split the contended fields away from the cold ones.
  The right move is usually one small hot shard, not turning the whole object graph into cache-line sculptures.

### Hot/cold splitting and code locality

- Split rare paths out of the loop body even if the source looks less "tidy."
  Front-end starvation often comes from cold error handling, logging, normalization, or debug checks physically sitting inside the hot block.
- If a branch is almost always false, keep the common case fall-through and move the rare path out-of-line.
  This is where `likely` hints and `noinline` can matter: not because hints are magic, but because code layout changes can reclaim front-end bandwidth.
- The same rule applies to data.
  Keep counters, coordinates, states, and indices together; move strings, handles, debug names, and optional metadata out of the cache footprint of the hot pass.

### Threading and the memory wall

- More threads are not a free multiplier.
  Once the workload is dominated by DRAM traffic, extra threads often buy you memory stalls, higher coherence traffic, and lower turbo clocks.
- Test scaling in stages: 1 thread, one SMT sibling, more physical cores, and then turbo-disabled if you need to expose whether you are bandwidth-bound or just frequency-lucky.
- If one thread already consumes a large fraction of sustained memory bandwidth, the next serious win probably comes from moving fewer bytes, not from adding worker threads.

### Prefetching and "just vectorize it"

- Software prefetch only helps when you can create a real time window before use.
  If the loop has no independent work between prefetch and consumption, you are just issuing extra instructions.
- SIMD helps when the loop is compute-dense or when vector loads reduce instruction pressure.
  It does not rescue pointer-heavy, branch-heavy, latency-bound code. Vectorizing a cache-miss machine just gives you a wider cache-miss machine.

## NEVER rules

- NEVER replace a branch with `cmov` or masked arithmetic just because branchless code sounds more advanced. It is seductive because it looks deterministic. The concrete failure mode is a longer dependency chain plus duplicated work, which often loses to a mostly-predictable branch. Instead, measure branch entropy first and use branchless code only for genuinely unpredictable, cheap branches.

- NEVER fix false sharing by padding everything in sight because "64-byte aligned" sounds like universal medicine. The non-obvious consequence is that you trade a coherency problem for a working-set and TLB problem. Instead, isolate only the proven hot shared fields and keep cold fields tightly packed.

- NEVER trust a clean object model in a hot loop if it implies pointer chasing, vtables, or cold data riding along with hot state. It is seductive because the code reads like architecture. The consequence is cache misses plus control-flow unpredictability that no amount of local instruction tweaking will save. Instead, specialize the hot path into contiguous arrays, tables, or tagged data that matches the actual traversal.

- NEVER celebrate a microbenchmark win before checking the real workload. Tiny loops over-emphasize front-end quirks, turbo effects, and toy cache behavior. The consequence is shipping a rewrite that wins the lab and loses the product. Instead, validate the app-level metric and keep the benchmark only as a microscope, not as the goal.

- NEVER add threads to a memory-bound loop because the single-thread graph still "looks busy." The seductive story is that spare cores must equal spare throughput. The consequence is early scaling collapse, coherence traffic, and lower clocks. Instead, reduce bytes moved, improve locality, or shard ownership before adding parallelism.

- NEVER assume all L1 hits are equal. Store-forwarding failures, 4K aliasing, line splits, and load-use timing can make "cache-resident" code stall badly. Instead, inspect the specific load/store path before rewriting whole subsystems.

## What to do in each top-down bucket

### If Front-End bound dominates

- Suspect cold code sitting in the hot block, too many taken branches, or instruction footprint that no longer fits the front-end well.
- First moves:
  move rare paths out-of-line, keep the hot path fall-through, simplify dispatch, reduce instruction count before trying clever intrinsics.
- Fallback:
  if source looks clean but counters do not move, inspect generated code layout and whether the compiler kept the uncommon path inside the loop anyway.

### If Bad Speculation dominates

- Find the exact branches with high entropy.
- First moves:
  restructure data so the branch becomes predictable, separate rare cases, or use branchless selection only when both sides are cheap.
- Fallback:
  if the branch is inherently random, redesign the algorithmic shape around batches, lookup tables, or state partitioning rather than polishing the same branch.

### If Back-End Memory dominates

- Assume the problem is bytes moved, latency, or ordering hazards before you assume the problem is arithmetic.
- First moves:
  hot/cold split, SoA or tighter packing, fewer passes, better ownership, check TLB pressure, check line splits, check store-forwarding and 4K aliasing.
- Fallback:
  if layout changes do not help, quantify whether the issue is cache capacity, bandwidth, or latency. The next move depends on which one is real.

### If Back-End Core dominates

- This is the time to care about dependency chains, divider usage, port pressure, and vector width.
- First moves:
  delete scalar divides where reciprocals are valid, shorten chains, fuse passes if it reduces total ops, and only then look at SIMD.
- Fallback:
  if SIMD complicates the loop and the counters barely move, revert. Wider instructions are not a trophy.

### If contention dominates

- Suspect lock convoying, shared counters, allocator sharing, or line ownership thrash.
- First moves:
  thread-local accumulation, sharded ownership, batching, and removing write-sharing from hot lines.
- Fallback:
  if the "contention" disappears on one core, re-check whether the real limit is memory bandwidth rather than locks.

## Muratori-style stop rules

- Stop when the measured bucket changes but the governing metric does not. You moved the bottleneck without creating value.
- Stop when the new version only wins under one compiler flag, one data shape, or a warmed-up microbenchmark. That is not a stable improvement.
- Stop when the optimization survives only because the team is afraid to delete a heroic rewrite. Performance code is not exempt from evidence.

## Final check before claiming a win

- Re-run the real metric, not just cycles in isolation.
- Check p50 and tail behavior when latency matters.
- Verify that the new layout did not regress locality for the next stage of the pipeline.
- Confirm that the change still wins when the dataset size crosses cache boundaries.
- Keep the simplest version that still wins. The Muratori lesson is not "write gnarly code." It is "remove waste, then align the remaining work with the machine that actually runs it."
