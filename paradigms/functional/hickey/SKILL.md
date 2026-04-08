---
name: hickey-simple-made-easy
description: Use Rich Hickey's simplicity lens to decomplect value, identity, time, coordination, and side effects in Clojure systems. Use when reviewing atoms/refs/agents/volatiles, choosing seq vs transducer vs eduction vs transient pipelines, or redesigning APIs where "functional" code still behaves unpredictably. Triggers: simple vs easy, complecting, value+time, swap! retries, commute/ensure, chunked laziness, transients, agent failure.
---

This skill is for system-shape decisions, not Clojure syntax.
Use it when the bug is "duplicate effects", "mystery retries", "lazy code read too much", or "functional code still feels brittle under load".

## Mandatory loading triggers
- Before redesigning boundaries or critiquing architecture, READ `references/philosophy.md`.
- Before scanning an existing `.clj` file for obvious complexity smells, inspect `scripts/simplicity_check.clj`.
- Do NOT load `scripts/simplicity_check.clj` for concurrency diagnosis, STM tuning, or performance work; it is a smell scanner, not runtime truth.

## Freedom calibration
- High freedom: renaming seams, collapsing abstractions, changing data shapes, splitting orchestration from model transitions.
- Low freedom: transaction bodies, validators, watches, agent actions, transient ownership, and resource-backed lazy pipelines. These have sharp runtime semantics; follow the rules exactly.

## The decomplecting pass
Before adding or keeping an abstraction, ask:
- Which dimensions does this unit know about: value, identity, time, coordination, effect?
- If it knows more than two, what can be pushed to the edge?
- If an invariant spans multiple mutable places, can the places become one value instead?
- If correctness depends on "exactly when input is consumed", should this be a seq at all?
- If a reviewer says "it looks functional", what mutable or temporal fact is still hidden?

Operational rule: every function should primarily do one of these jobs:
- compute a model transition
- coordinate state
- interpret external input
- perform an effect

If one function does two or more, assume it will be harder to replay, test, and change than it looks.

## Decision tree: pick the state primitive
- Need local, single-owner reduction state inside one transducing process or tight loop?
  - Use `volatile!`.
  - Reason: no validators, watches, or CAS overhead.
  - Sharp edge: `vswap!` is explicitly non-atomic.
- Need synchronous, independent shared state?
  - Use `atom`.
  - Only safe when the update function is pure and retry-safe.
  - If you keep reaching for `compare-and-set!`, remember it compares with `identical?`, not `=`.
- Need synchronous, coordinated updates across locations?
  - Use `ref` + `dosync`.
  - Default history is `:min-history 0` and `:max-history 10`; do not tune unless long transactions are faulting on history.
- Need asynchronous, independent state transitions?
  - Use `agent`.
  - `send` is for CPU-bound work; `send-off` is for blocking I/O.
  - Agents fail closed until `restart-agent`; short-lived tools/tests often need `shutdown-agents`.
- Can state be derived from immutable input or an event log?
  - Use no shared mutable cell at all.

## Decision tree: invariant spans more than one thing
1. Can you collapse the invariant into one ref or one append-only event pipeline?
   - Do that first. Most multi-ref designs are accidental normalization.
2. If you must keep multiple refs, is the cross-ref update truly commutative?
   - If yes, `commute` may reduce contention.
   - If no, use `alter`.
3. If you only need to protect a read of another ref, `ensure` is legal but not cheap.
   - In symmetric contention, official Clojure Q&A shows `ensure` can livelock with repeated 100ms timeout retries and seconds of delay.
   - If the path is hot, prefer one coarser ref or explicit serialization.
4. If helper code must never run inside a transaction, wrap it with `io!`.
   - Do not rely on discipline alone.

## Decision tree: seq, transducer, eduction, sequence, transient
- Want the clearest debugging story and over-consumption does not matter?
  - Use ordinary seqs.
- Want reusable transformation logic without caching and without surprise early consumption?
  - Use `eduction`.
  - Non-obvious win: every `reduce`/`iterator` reapplies the xform from source; this is safer for offset-sensitive or stateful producers.
- Need a seq result from a transducer and can tolerate eager probing?
  - Use `sequence`.
  - Sharp edge: current implementation may consume the first chunk before the caller pulls; on chunked sources this is commonly 32 items.
  - This is toxic for parsers, PRNG-driven simulations, and sources where read position matters.
- Need exact stopping semantics or deterministic consumption?
  - Use `reduce`/`transduce` with `reduced`, or `loop/recur`.
- Need a collection output?
  - Use `into`; it already uses `reduce` and transients when possible.
- Need speed in a large builder loop?
  - Try transients only after removing unnecessary intermediate seqs.
  - Official Clojure 1.12 docs show a 1M-element vector builder moving from about `8.4 ms` to `5.5 ms`; good, but not magic.

## Runtime truths people learn the hard way
- `swap!` may call the update function multiple times. Treat the function like a pure reducer, never like a command.
- `commute` replays its function at commit against the most recently committed value, not the in-transaction snapshot. If order matters, you already chose the wrong primitive.
- Watches are synchronous, may run on multiple threads, and an atom/ref may have changed again before the watch runs. Use the `old-state`/`new-state` args; do not re-deref for truth.
- Sends from inside an agent action are held until that action completes. `release-pending-sends` is a rare escape hatch, not a design pattern.
- Sends made inside `dosync` are held until commit and discarded on retry or abort. This makes agents a good post-commit handoff, but only if you keep the transaction pure.
- `seq` over Java `Iterable`s and arrays is not a true immutable value boundary; later mutation can be observed, and iterator-backed seqs can still surface `ConcurrentModificationException`.
- Since Clojure 1.7, transients no longer auto-detect wrong-thread use. Frameworks like `core.async` may still make them safe, but your own cross-thread leak may fail only as "weird behavior".

## Anti-pattern ledger
NEVER put logging, HTTP calls, UUID generation, or metrics emission inside `swap!`, `alter`, `commute`, validators, or `dosync` bodies because "the state change is atomic" is seductive but false for effects; retries duplicate work and desynchronize the outside world. Instead compute pure state first and emit effects from a committed boundary.

NEVER use `commute` for order-sensitive updates because the extra concurrency looks free, but commit-time replay turns the code into last-one-in-wins behavior. Instead use `alter`, or remodel the update as a truly commutative aggregate.

NEVER use `ensure` as a default "cheap read lock" because it feels lighter than changing another ref, but under cross-ensuring writers it can livelock and burn whole seconds in retry storms. Instead collapse the invariant into one ref, or serialize through one reducer/event stream.

NEVER use `sequence` on offset-sensitive, resource-backed, or PRNG-backed sources because "it returns a lazy seq" sounds safe, yet it may consume an initial chunk before anyone asks and it fully realizes intermediate transducer steps. Instead use `eduction` or an explicit `reduce`/`loop`.

NEVER spread a transient across helper boundaries or threads because "it still looks like a collection" hides mutable shared structure and aliasing. Instead keep one owner, capture every returned transient value, and call `persistent!` as soon as the local construction phase ends.

NEVER deref inside a watch and treat that as current truth because watches are synchronous but not snapshot-stable, and they may run concurrently. Instead use `old-state` and `new-state`, then hand off slow work elsewhere.

NEVER use agents as a generic queue when callers need acknowledgements or strict end-to-end ordering because per-agent serialization is seductive but failure mode matters: a failed agent holds queued actions until `restart-agent`. Instead use agents only for independent async state, or choose a queue/stream abstraction with explicit delivery semantics.

NEVER use `compare-and-set!` as if it were value equality because it is easy to assume "same logical value" is enough; the actual test is `identical?`. Instead reserve it for identity-sensitive handshakes and use `swap!` for logical state transitions.

## Recovery moves when the design is already tangled
- Duplicate effects under contention:
  - Move effects out of retry regions.
  - Store intent as data, then consume it after commit.
- STM retry storms:
  - Replace cross-ref invariants with one coarser ref or one ordered reducer.
  - Tune ref history only after proving read faults, not as a first response.
- Lazy pipeline leaks memory or over-reads:
  - Replace the hot slice with `transduce`/`reduce`.
  - Use `eduction` when you still want reusable transforms.
- "Functional" API is still hard to test:
  - Split pure model transition functions from orchestration and side-effect code.

## Review checkpoints
- If a component mixes business rules with clocks, retries, or backpressure, it is not simple yet.
- If replaying yesterday's inputs cannot reproduce today's behavior, time leaked into the model.
- If changing the scheduler changes business meaning, coordination leaked into the model.
- If the only reason a mutable cell exists is "it was convenient to thread through here", delete it or push it outward.
