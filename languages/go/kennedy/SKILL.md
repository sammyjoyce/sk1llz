---
name: kennedy-mechanical-sympathy
description: Apply Bill Kennedy's Go engineering style when correctness, latency, or API design depends on how the runtime actually pays for code. Use when working on escape analysis, value vs pointer semantics, channel/goroutine design, scheduler or GC behavior, profiling, tracing, or container tuning. Triggers: mechanical sympathy, data semantics, escape analysis, pprof, go tool trace, schedtrace, GOMAXPROCS, GOMEMLIMIT, channel buffer, sync.Pool, goroutine leaks.
tags: go, runtime, scheduler, gc, profiling, tracing, escape-analysis, channels, goroutines, semantics
---

# Kennedy Mechanical Sympathy

Bill Kennedy style is not "micro-optimize Go." It is "preserve integrity, then make the runtime bill explicit." If you cannot name the copy, allocation, queueing rule, or scheduler consequence you are buying, you are still guessing.

This skill is self-contained. Do not go load generic Go references until you have identified which runtime bill you are trying to change.

## Start With The Bill

Before editing code, ask yourself:

- Is the problem CPU time, allocation rate, scheduler latency, lock contention, or container throttling?
- Is this type modeling ownership and identity, or copies and replacement?
- Is this channel carrying data, or only a signal?
- Is the service running under cgroup CPU or memory limits?

Then pick the lane:

| Symptom | First move | Why this lane |
| --- | --- | --- |
| Higher `ns/op`, unclear why | CPU profile, then benchmark again | CPU work first; tracing is too wide for hotspot attribution |
| Higher `B/op` or `allocs/op` | `go test -benchmem`, then memory profile, then `-gcflags=-m -m` | `pprof` tells you where; compiler tells you why |
| Good microbenchmarks, bad p95/p99 | `go tool trace`; if prod overhead matters, start with `GODEBUG=schedtrace=1000,scheddetail=1` | This is usually runnable latency, blocking, or syscall imbalance |
| Good bare-metal behavior, bad containers | inspect `GOMAXPROCS`, CPU limit, `GOMEMLIMIT` | The runtime may be fighting cgroup throttling, not your algorithm |
| Too many goroutines or rising memory floor | inspect `/sched/goroutines*`, `/sched/latencies`, `/gc/stack/starting-size:bytes` | Goroutine count hurts twice: scheduler pressure and GC root scanning |

## Data Semantics Before APIs

Bill Kennedy's strongest habit is choosing semantics before method sets.

- Use value semantics when copies are part of the truth of the API: snapshots, replacement, small immutable-ish records, and data you want callers to reason about locally.
- Use pointer semantics when the value represents identity, shared mutable state, external resources, or invariants that would become dangerous if copied.
- Once a type picks a semantic, keep it consistent across receivers, helpers, and interfaces. The method-set rules are there to stop you from smuggling a pointer-semantic type through value copies.

Before changing a receiver, ask yourself:

- If this value were copied in three call paths, would the program still be correct?
- Is the copy cheaper than the extra heap object, pointer chasing, and GC scan work?
- Am I changing semantics because the type truly changed, or because I want a convenient method call?

Two Kennedy-style heuristics matter here:

- Interface values are "valueless" from a design perspective. Do not reason from the implementation detail that interfaces currently store a pointer internally. Reason from the semantic contract: value receivers permit storing copies; pointer receivers preserve no-copy intent.
- Mixed semantics are acceptable only as a conscious exception. If half the code treats a type as a copy and half as shared state, code review loses its ability to catch side effects.

## Channels Are Signaling Contracts

Treat a channel choice as a delivery guarantee decision, not as a vague queue abstraction.

- Unbuffered channel: sender gets a receive guarantee before send completes. Use it for handoff, request/response orchestration, or "you may not proceed until someone took this."
- Buffered channel of `1`: delayed guarantee. The second send cannot complete until the first value has been received. Use it when you need one unit of slack without losing backpressure.
- Buffered channel `>1`: no receive guarantee. Use it only when the system has a real bounded backlog model.

Before assigning channel capacity, ask yourself:

- What physical constraint is this buffer representing: workers, open connections, downstream QPS, bytes on the wire, or tolerated burst?
- What failure do I want when that bound is hit: block, drop, or cancel?
- If I make the buffer larger, what signal becomes invisible?

Expert rule: a channel buffer must come from a named bound. "8 felt good" is not a design.

## Goroutines Are Not Free Once Runnable

Spawning is cheap; runnable goroutines are where the bill arrives.

- Each P has a local run queue of 256 goroutines. If runnable work spills far beyond roughly `GOMAXPROCS * 256`, you are now paying more global queue and steal overhead and less useful work.
- The scheduler has a `runnext` fast path for communicate-and-wait pairs, which is why tight handoff pipelines can feel great at low concurrency and fall apart once you flood them with unrelated runnable work.
- New goroutines no longer simply "start at 2 KB forever." The runtime tracks scanned stack sizes and adapts the starting stack size over time. If your service creates huge goroutine bursts after deep call stacks become normal, stack footprint can climb unexpectedly.

Before adding fan-out, ask yourself:

- Am I bounding concurrency to the scarce resource, or merely matching input cardinality?
- Do I need more concurrency, or less runnable latency?
- Would a fixed worker set preserve cache locality and stack reuse better than one goroutine per item?

When the answer is unclear, measure `/gc/stack/starting-size:bytes`, `/sched/goroutines/runnable:goroutines`, and `/sched/latencies:seconds` before touching code.

## Escape Analysis: Ask Why, Not Just Where

`pprof` tells you where memory is allocated. The compiler tells you why it had no stack option.

Use this sequence:

1. `go test -bench ... -benchmem`
2. For end-of-benchmark churn, inspect `alloc_space` or `alloc_objects`
3. For live footprint, inspect `inuse_space`
4. Only then run `go build -gcflags='-m -m'`

Kennedy's important correction: `make([]byte, n)` with variable `n` often escapes not because the slice is "too large," but because the compiler cannot size that stack frame at compile time. Replacing a value with a pointer does not fix that bill; reusing caller-owned scratch space or making the bound static sometimes does.

Optimization rule:

- Remove needless temporary ownership first.
- Remove unpredictable sizes second.
- Only then consider pooling.

## GC And Memory Limits

Know the trade, or do not tune.

- `GOGC` is a CPU vs memory dial. Doubling it roughly doubles heap overhead and roughly halves GC CPU cost for steady-state workloads.
- `GOMEMLIMIT` is a soft limit, not a safety blanket. Set it too close to the working set and the runtime can thrash trying to stay under it. The runtime intentionally caps GC CPU to about 50% over a `2 * GOMAXPROCS` CPU-second window to avoid total collapse.
- Since Go 1.18, GC pacing includes GC roots such as goroutine stacks. Hundreds of thousands of goroutines are not "just scheduler state"; they distort GC economics too.

Use `GOMEMLIMIT` when you know the memory envelope. Avoid baking it into CLIs or unknown-input tools where working set depends on user data or host memory you do not control.

## Container Reality

Container CPU limits are throughput limits, not parallelism limits.

- On Go 1.25+, default `GOMAXPROCS` becomes container-aware and tracks CPU limits unless you manually set `GOMAXPROCS` via env or `runtime.GOMAXPROCS`.
- On Go versions before 1.25, the runtime defaults to host CPU count, not the container limit. In containers, failing to correct this is often a tail-latency bug, not a throughput win.
- CPU throttling usually happens on a 100 ms period. If `GOMAXPROCS` is much higher than the effective CPU limit, Linux can pause the process for the rest of that period, which shows up as ugly tail latency.
- The new default is better for most services, but bursty workloads can still prefer a different setting because a hard parallelism cap may block short CPU spikes that were previously tolerated.

Before overriding `GOMAXPROCS`, ask yourself:

- Which Go version is this binary built with?
- Is latency coming from kernel throttling, or from too little parallelism?
- Did I just disable runtime auto-adjustment by setting `GOMAXPROCS` manually?

## `sync.Pool` Is Scratch Reuse, Not Ownership

Use `sync.Pool` only for temporary objects reused across independent concurrent clients.

- The runtime may drop pooled items at any time.
- Current implementations keep a victim cache for roughly one GC cycle, but you must code as if every `Get` can miss.
- `Pool.New` should generally return pointer types, since storing non-pointers in the returned interface value commonly adds an allocation you were trying to avoid.
- Pooling large buffers without size caps often keeps oversized backing arrays circulating long after the burst that created them.

Good uses: request-local scratch buffers, temporary encoders, short-lived formatting state.

Bad uses: connection ownership, per-object free lists, caches that must stay warm, or anything whose correctness depends on pool retention.

## NEVER Do These

- NEVER add pointers just to "avoid copies" because the seductive local win often becomes heap promotion, more GC scan work, and worse cache locality. Instead measure the copy cost against allocation and scan cost, and prefer values until identity or mutation truly requires sharing.
- NEVER mix value and pointer semantics for the same type because it feels ergonomic in one call site. The consequence is hidden copies, harder interface reasoning, and side effects code review stops seeing. Instead pick one semantic and make exceptions explicit and rare.
- NEVER choose a channel buffer by folklore because a larger buffer feels like instant throughput. The consequence is erased backpressure and lost delivery guarantees. Instead derive capacity from a named bound or use buffer `1` deliberately for delayed guarantee.
- NEVER use a buffered channel for cancellation-only signaling because it looks symmetric with work channels. The consequence is ambiguous ownership and one more hidden queue. Instead use `context.Context` or a closed done channel.
- NEVER trust `sync.Pool` to retain state because it worked in a benchmark. The consequence is production misses after GC and giant pooled objects lingering after bursts. Instead design for miss-tolerance and cap what you return to the pool.
- NEVER set `GOMEMLIMIT` right above steady-state memory because it feels safer than an OOM. The consequence is soft-limit thrashing and worse latency than a clean crash. Instead leave headroom or lower `GOGC`.
- NEVER pin `GOMAXPROCS` in a container without checking Go version and CPU policy because a manual override now disables adaptive defaults on modern Go. Instead verify cgroup limits, throttling behavior, and p99 latency first.
- NEVER reach for tracing to find hot code because traces are seductive and visual. The consequence is wide data with weak hotspot attribution. Instead use CPU or memory profiles first, then trace only when the problem is scheduling, blocking, or utilization.

## What "Done" Looks Like

A Kennedy-style change is done when you can state, in one sentence each:

- which semantic model the type now uses,
- which runtime bill you reduced,
- which measurement proved it,
- and which guarantee you intentionally kept or gave up.
