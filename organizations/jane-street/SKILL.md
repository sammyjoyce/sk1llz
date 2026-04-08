---
name: jane-street-functional-trading
description: "Design or review trading systems in a Jane Street style: atomic risk boundaries, replay-first architecture, determinism before headline speed, and OCaml/typed-functional techniques only where they buy real safety or latency. Use when building or debugging market making, pricing, market-data, exchange, or risk infrastructure, especially when users mention OCaml, order books, inventory limits, replay, jitter, queue growth, zero-allocation hot paths, or microsecond latency."
tags: ocaml, trading, market-making, exchange, latency, determinism, replay, risk, queueing, concurrency
---

# Jane Street Functional Trading

This skill is for systems where a few microseconds of slop changes quotes, queue position, or risk exposure.

Load order:
- This skill is intentionally self-contained; do not load extra Jane Street notes for routine design or review work.
- Before changing a hot path, read the project's current benchmark, trace, or queue-growth evidence first.
- Before changing risk or matching logic, read the exact code that defines the risk boundary and recovery/replay path.

## Operating Stance

- Optimize for deterministic behavior before absolute peak speed. A system that is 1ns slower but loses a tail mode can be the better trading system.
- Spend low-level complexity only on the tiny fraction of code that sits on the critical path. Jane Street-style systems keep most code high-level and typed, then quarantine the ugly zero-allocation code where queue math proves it is necessary.
- Treat performance as a distribution and a queue-growth curve, not a single number. Temporary allocation can move the median by wrecking cache locality even when GC pauses look rare.
- Favor architectures whose state can be rebuilt quickly from a sequenced log. Once recovery takes longer than about 30-60 seconds, teams start adding state-transfer shortcuts and the system gets brittle.
- Do not optimize a symbol or service in isolation if the real risk boundary is cross-instrument. ETF and factor overlap often make "parallelize by symbol" the wrong decomposition.

## Before You Design

Before you parallelize matching or risk logic, ask yourself:
- Is the real atomic boundary a symbol, a venue, a portfolio, or an entire correlated market slice?
- What risk limit must be enforced in one indivisible step?
- If I shard this, what exposure becomes temporarily invisible?

Before you flatten a hot loop, ask yourself:
- What is the per-message budget during the worst burst, not the average minute?
- Does backlog stay bounded if processing time drifts from sub-microsecond to several microseconds?
- Is the problem GC pauses, cache churn from short-lived objects, bad speculation, interrupt noise, or cross-core coordination?

Before you add concurrency, ask yourself:
- What is the read/write ratio?
- How long is the synchronized section?
- Do I need linearizability, read-your-own-writes, or merely eventual consistency?
- Can readers tolerate stale data, and if so for how long?

Before you trust a benchmark, ask yourself:
- Am I on bare metal or hiding behind VM jitter?
- Did turbo, interrupts, scheduler ticks, or noisy neighbors dominate the trace?
- Did I measure only medians when the actual loss comes from tails and queue position?

Before you move OCaml work across threads or processes, ask yourself:
- Is the value actually portable, or is a closure smuggling mutable capability across the boundary?
- Am I sharing data, or sharing the ability to mutate data?

## Decision Rules

- If atomic risk must span overlapping instruments, prefer a single sequencer or a central risk gate over symbol-level parallelism.
- If peak-feed queue growth is the failure mode, first remove allocation and coordination from the critical loop before touching fancy algorithms.
- If reads vastly outnumber writes and stale reads are acceptable, use reader-optimized structures only after proving the writer cost and semantics are acceptable.
- If writes are frequent, or if readers must see a single coherent truth, prefer a serial owner or a plain mutex over clever reader-heavy primitives.
- If the OCaml hot path still allocates after flattening data flow, use flatter layouts, stack allocation, or preallocation only if the compiler/runtime supports them cleanly; treat FFI-managed pools as the last resort.
- If tails remain after code cleanup, work outward: bare metal, CPU isolation, timer-tick removal, frequency policy, speculation hints, then hardware.

## Practitioner Heuristics

- In market-data paths, "fast enough" can be shockingly fast. Jane Street engineers have described cases where roughly 750ns per message kept queues tame near the close, while pushing into multi-microsecond handling caused backlog to explode.
- Market data can arrive on the order of gigabytes per second. Ten microseconds of staleness is already enough to make bad trades in some strategies.
- Reader-writer locks are often a trap in short hot loops. Each reader still writes shared metadata, so coherence traffic can dominate the work.
- A single cache-line transfer between cores is on the order of tens of nanoseconds; two transfers to acquire and release a read lock can burn most of a tiny latency budget before useful work starts.
- False sharing can erase an otherwise elegant design. One missing 64-byte alignment on per-thread counters can create order-of-magnitude slowdowns.
- Lock-free does not mean contention-free. If ownership of a hot cache line bounces, the CPU still punishes you.
- In spin loops, bad speculation above roughly 1% is already a red flag. A pause instruction or other speculation barrier can be worth more than another data-structure rewrite.
- Turbo boost can improve raw throughput while worsening determinism. Disable it when tighter tail control matters more than the highest single-sample speed.
- VM benchmarks lie for tail-sensitive systems. Bare metal can remove hundreds of nanoseconds of median latency before you touch application code.

## Anti-Patterns

- NEVER shard matching or pre-trade risk "by symbol" because it parallelizes beautifully on a whiteboard. Correlated products, especially ETFs and shared factors, create exposures that no shard can see atomically. Instead shard only after proving the risk boundary is truly local, or keep one sequencer for that risk domain.
- NEVER chase median latency alone because the seductive story is "GC only hurts the tail." Short-lived allocation also destroys cache locality and can move the median enough to destabilize queue growth. Instead inspect allocation, cache behavior, and queue buildup together.
- NEVER put Deferred-heavy, closure-rich abstractions directly on the packet path because the code looks composable and civilized. Closure allocation, scheduler boundaries, and heap churn destroy determinism long before they show up as dramatic pauses. Instead keep the critical loop synchronous, flat, and preallocated, and move async boundaries to the edges.
- NEVER switch to a reader-writer lock just because the workload is read-heavy on paper. In short sections, readers contend with each other through shared lock metadata and can scale worse than a mutex. Instead choose primitives from read/write ratio, critical-section length, and consistency needs.
- NEVER call a design "fixed" because it is lock-free. The seductive mistake is ignoring cache-coherence traffic, false sharing, and ownership movement. Instead reason about which cache lines move between cores and align or partition data so readers stay local.
- NEVER copy zero-allocation style across the whole codebase because it feels principled. That turns ordinary code into hard-to-change code while buying nothing off the hot path. Instead quarantine the ugly low-level style to the loops whose queue budget demands it.
- NEVER trust aggregate observability alone for pathological latency because percentiles and averages hide when the butterfly actually appears. Instead combine static instrumentation with time-based traces triggered by events that exceed the real trading budget.
- NEVER share arbitrary closures across threads because the closure itself looks immutable. A closure can smuggle access to mutable thread-local state and break race reasoning. Instead pass only portable data and functions, and keep shared mutation behind typed ownership or lock APIs.
- NEVER add a fast recovery shortcut before proving replay is too slow. State transfer looks attractive, but it usually creates two recovery semantics to keep consistent. Instead first see whether faster components let you rebuild from the log inside the operational window.

## How To Think About Specific Tasks

For pricing and risk code:
- Do not let live clocks, session calendars, or venue connectivity leak into the pricing kernel; they make replay disagree with production exactly when you need post-trade forensics.
- Encode the boundaries that actually break desks in practice: session state, marketability, inventory regime, stale-vs-live data, and recovery mode.
- Use types for structural invariants that should never vary mid-flight, and runtime guards for facts that can change between packets.
- If a model is numerically delicate, design the debugging path before the model rewrite; silent wrong answers are worse than loud failures in trading code.

For market-data handlers:
- Budget for the burst, not the average feed, because the economic loss is usually stale-action error rather than outright downtime.
- Measure queue depth and time-in-queue, not only handler duration; handlers can look "fast" while the system is already quoting off buffered data.
- If a loop spins, inspect speculation, interrupts, scheduler ticks, and CPU policy before redesigning the data structure.
- Prefer dropping optional enrichment over delaying the next market-data decision.

For exchange or matching engines:
- Deterministic single-threaded components are often the right primitive because they simplify fairness, replay, and replication.
- End-to-end latency is not always the primary metric; sometimes single-digit microsecond components matter because they let the overall replicated architecture stay simple and robust.
- If an edge connection allows only one unacknowledged transaction with a tiny socket buffer, throughput becomes a direct function of component latency, so design pushback deliberately.

For parallel OCaml work:
- Treat portability, contention, and aliasing as first-class design questions.
- Mutable state is not the only hazard; a function that captures mutable state is also a capability leak.
- Prefer process boundaries or narrowly typed shared-memory APIs when you need guarantees you can explain to a postmortem reviewer.

## Failure-Handling Playbook

- If the hot path is slow and allocation is visible, flatten representations, preallocate, and remove scheduler abstractions from the loop.
- If the hot path is slow and allocation is not visible, inspect cache misses, cross-core traffic, speculation, and interrupts.
- If profilers say "mostly spinning" and give you nothing actionable, switch to time-based control-flow tracing with an "interesting event" threshold just above the real latency budget.
- If a concurrent structure benchmarks well but regresses under scale, suspect false sharing before inventing a new theory.
- If replay is too slow, simplify the event stream or speed up component handlers before adding a second recovery mechanism.
- If a design is fast but hard to reason about, push complexity outward until the critical path is the only ugly code left.

## What Good Output Looks Like

When you use this skill, produce:
- The true risk boundary.
- The replay and recovery story.
- The hot-path allocation policy.
- The coordination strategy and why its semantics fit the workload.
- The measurement plan, including which tail event counts as economically bad.
- The exact place where you intentionally spend low-level complexity, and why it is quarantined there.
