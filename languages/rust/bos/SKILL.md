---
name: bos-concurrency-rust
description: "Design and review Rust concurrency in the style of Mara Bos: atomics, memory ordering, lock internals, wait/notify, false-sharing avoidance, and contention tuning. Use when implementing or auditing custom `Mutex`/`RwLock`/spinlock/condvar code, choosing `Ordering`, reasoning about `Arc`/`Weak` or `OnceLock`, debugging lock-free races, or testing concurrency with Loom. Triggers: `Atomic*`, `Ordering`, `compare_exchange`, lock-free, spinlock, false sharing, `Arc`, `Weak`, `OnceLock`, `wait`, `notify`, Loom, contention, cache line."
tags: rust, concurrency, atomics, memory-ordering, locks, spinlocks, condvar, arc, weak, once-lock, loom, false-sharing, performance
---

# Mara Bos: Prove the Handoff, Not the Intuition

Concurrent Rust is about proving who may touch which memory after which event. The bug is usually not "the value arrived late"; it is "a thread derived permission from an atomic that never established happens-before."

## Use This Skill

Use this for low-level shared-memory concurrency, not generic async workflow code. If the problem disappears when reduced to ownership transfer, channels, or `OnceLock`, take that simpler path.

This skill is intentionally self-contained. Do not pull in large lock-free examples or generic async docs until you have identified which case you are in: one-time init, lock-protected invariants, or a truly custom atomic primitive.

## Before You Touch `Ordering`, Ask Yourself

- Is this atomic only tracking a number, or does reading it grant access to other memory?
- Am I publishing data, transferring exclusivity, or just collecting statistics?
- Does this state machine live in one atomic word, or am I smuggling invariants across multiple atomics?
- Is contention brief enough to spin, or can the holder sleep, allocate, block on I/O, or cross FFI?
- Am I reasoning on x86-64 only? What breaks or slows down on ARM64?

## Choose the Primitive

- If ownership transfer or message passing removes sharing, do that. Bos-style atomics are for cases where shared-memory coordination is unavoidable.
- If initialization happens once, prefer `OnceLock` or `LazyLock`. Use `OnceLock` when initialization needs runtime inputs or retryable control flow.
- If the invariant spans multiple fields, use a `Mutex` or `RwLock`. One atomic per field is usually a disguised lock with worse failure modes.
- If waiters must sleep, use a blocking primitive. Spinning is only an optimistic front-end for microcontention.
- If you are building a custom primitive, compress the state into one machine word first. Multi-atomic protocols multiply proof obligations.

## Ordering Heuristics That Matter

- `Relaxed` is for independent facts, not permissions. A relaxed counter is fine; a relaxed "ready" bit that causes another thread to read non-atomic data is not.
- `Acquire`/`Release` is the default handoff pair: release when publishing work or unlocking; acquire when consuming that publication or locking.
- `compare_exchange` has two orderings because failure is still a load. If the failure path reads the published object, the failure ordering often must be `Acquire`, not `Relaxed`.
- Failure ordering never includes a store, so it cannot be `Release` or `AcqRel`. If you find yourself wanting that, your proof is probably mixing the success and failure paths.
- On x86-64, `Acquire` and `Release` often compile to the same instructions as `Relaxed`; on ARM64 they do not. Do not conclude an ordering is "free" from laptop benchmarks.
- On x86-64, a `SeqCst` store is heavier than a plain release store because it becomes an `xchg`; on ARM64 Bos notes `SeqCst` RMWs are essentially as expensive as acquire/release RMWs. Pick orderings from the proof, then benchmark.
- If multiple atomics define one logical state machine, start with `SeqCst` until the proof is stable. Relax individual edges only when you can name the exact release/acquire pair that replaces the global order.

## Contention and Cache Behavior

- Failed CAS is not "just a read." On most CPUs it still claims exclusive access to the cache line. In spin loops, poll with `load` first and attempt CAS only when the lock looks open.
- A hot atomic can damage unrelated neighbors through false sharing. `#[repr(align(64))]` is a reasonable starting guess, but `crossbeam::CachePadded` pessimistically uses 128-byte padding on x86-64, aarch64, and powerpc64 because adjacent-line prefetch can still hurt you.
- `std::hint::spin_loop()` is only a CPU hint; it does not yield to the OS. Bound the spin and fall back to parking or a blocking lock or you invite priority inversion.
- Bos uses 100 spins as a plausible starting point for optimistic mutex spinning and notes Rust's Linux mutex used 100 in Rust 1.66. Treat that as a benchmark seed, not a truth.
- If you can encode "waiters exist" in the state word, do it. A 3-state mutex (`0` unlocked, `1` locked/no waiters, `2` locked/waiters) avoids unconditional wake syscalls in the uncontended path. Bos measured major Linux wins; macOS and Windows often show much smaller gains because their wake primitives already do bookkeeping.
- Waking everybody for a single-resource handoff is usually a thundering herd bug, not generosity. Wake one waiter or requeue onto the mutex.

## Locks, Parking, and Fairness

- `parking_lot` is not just "faster std." It gives adaptive spinning for microcontention, no spurious `Condvar` wakeups, waiter requeueing on `notify_all`, and task-fair `RwLock` behavior.
- Eventual fairness matters when one thread repeatedly reacquires before others run. `parking_lot` forces a fair unlock on average every 0.5 ms, and critical sections longer than 1 ms always unlock fairly. Use `FairMutex` only when starvation guarantees matter more than raw throughput.
- For portable futex-style wait/wake code, prefer a 32-bit state word. Bos's wait/wake examples use `AtomicU32` because 32-bit atomics are the portable denominator.
- Separate correctness from parking. Wait/wake prevents CPU burn; it does not create the happens-before edge that makes shared access safe.

## `Arc`, `Weak`, and One-Time Init

- Custom reference counting is harder than it looks. Separate `Arc` and `Weak` counts can miss a concurrent downgrade/upgrade window unless you deliberately lock the check.
- In Bos's `Arc` reasoning, relaxed increments are sometimes fine, but the transitions that prove uniqueness or final destruction must synchronize with the matching decrements. If you cannot narrate those edges, do not ship the custom refcount.
- Prefer `OnceLock` over racy CAS-based lazy init when the constructor is expensive; duplicate initialization work under contention is often worse than briefly blocking.
- Prefer `OnceLock` over `LazyLock` when you need runtime parameters. `LazyLock` poisoning after a panic is unrecoverable for all future accesses.

## Bos-Style Procedure For A New Primitive

1. Write the state machine as integers and legal transitions before code.
2. Mark which transition publishes data and which transition consumes it.
3. Separate correctness from parking: wait/wake reduces CPU burn, but it does not make unsound sharing safe.
4. Minimize the shared footprint: one state word, padded hot fields, short critical sections.
5. Model the smallest version with Loom before optimizing.
6. Benchmark three regimes separately: uncontended, microcontention, and long-hold contention.

## Testing Without Fooling Yourself

- Loom is mandatory for nontrivial custom primitives, but it is not omniscient. Loom explicitly cannot model all reorderings allowed by `Relaxed` and some cross-atomic weak-order behaviors.
- Use `LOOM_MAX_PREEMPTIONS=2` or `3` as a practical starting bound; Loom's own docs say that catches most bugs without exploding the state space.
- After Loom passes, run stress tests on real weakly ordered hardware if the code matters. ARM64 is much more informative than another x86 laptop.
- Test for wake behavior, not just final values. A lock or condvar that is "correct" only because it accidentally busy-loops is still broken.

## NEVER

- NEVER use `Relaxed` on a flag, pointer, or refcount transition that grants access to other memory because the seductive part is that tests still pass on x86-64. Instead pair the publishing edge with `Release` and the consuming or failure-read edge with `Acquire`.
- NEVER spin on `compare_exchange` alone because it feels like the shortest lock loop. Failed CAS still grabs the cache line exclusively and amplifies false sharing. Instead spin on `load`, then attempt CAS, then park.
- NEVER split one invariant across several atomics because each field looks locally simple. The consequence is a proof obligation across interleavings that is harder than a mutex and easier to get wrong. Instead keep the invariant under one lock or one state word.
- NEVER benchmark synchronization on one architecture and generalize. x86-64 hides ordering costs that ARM64 exposes, while OS wake implementations differ enough to invert "obvious" optimizations. Instead benchmark on the target mix and reason from the memory model first.
- NEVER assume a green Loom run proves relaxed-order code is correct because Loom cannot emulate every relaxed reordering. Instead combine Loom with architecture-aware reasoning and hardware stress.
- NEVER reach for always-lock-free designs because uncontended microbenchmarks look beautiful. The seductive part is avoiding syscalls; the consequence is complex recovery, starvation, and cache-line warfare under real contention. Instead prefer `Mutex`, `RwLock`, or `OnceLock` until measurement proves the simpler design is the bottleneck.

## Freedom Calibration

- High freedom: API shape, data layout, sharding, and whether to use `Mutex` versus `RwLock` versus `OnceLock`.
- Low freedom: memory-order proofs, CAS failure ordering, wake policy, and cache-line layout on hot atomics. Treat those as engineering constraints, not stylistic choices.
