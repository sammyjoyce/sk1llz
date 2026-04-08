---
name: turon-api-design
description: "Design Rust public APIs in Aaron Turon's style: borrowed-first async interfaces, zero-cost lower layers, semver-aware trait boundaries, and ecosystem-friendly layering. Use when shaping async I/O crates, public traits, return types, runtime boundaries, re-exports, or extension points. Triggers: async api, futures, tokio, impl Trait, object safety, blanket impl, specialization, backpressure, cancellation, crate facade."
tags: rust, async, api-design, futures, semver, traits, object-safety, backpressure, cancellation, ecosystem
---

# Aaron Turon: Public API Shape

Turon's bar is not "idiomatic Rust." It is harder: the API should feel as direct as sync Rust, preserve zero-cost composition, and still leave room for the ecosystem to evolve around it.

Use this skill for public surface design, not for executor internals or borrow-checker puzzles in isolation.

## Loading Order

- Before changing a public API, read the crate root exports and current rustdoc examples first.
- Before changing an async trait, locate the real spawn/detach boundary in the target crate before touching signatures.
- Before changing blanket impls or specialization, enumerate the current impl set plus one upstream-minor-release world and one downstream-extension world.
- Do NOT start with executor internals, benchmark harnesses, or private module layout. Turon's method fixes the contract boundary first and only then tunes internals.

## Start Here

Before changing a public API, ask yourself:

- Where is the real `'static` boundary? Most async work does **not** need `'static`; only spawned or detached work usually does.
- Where does overload show up? If the caller cannot observe "not ready", you probably hid backpressure inside an internal queue.
- Which layer owns policy? Runtime choice, buffering, retries, and service orchestration belong above the foundational trait layer.
- Which impl territory am I occupying? Every blanket impl or exposed trait contract spends downstream design space.
- Which guarantees are part of the contract but easy to forget? `Send`, `Sync`, `Clone`, `DoubleEndedIterator`, object safety, and cancellation behavior are the usual ones.

## Design Procedure

### 1. Put the spawn boundary last

If the operation lives inside the caller's lexical scope, design the async signature to look like the sync signature plus `async`. Turon's async-borrowing point was not cosmetic; it eliminates the old ownership-threading pattern that forced `Rc<RefCell<_>>`, tuple-returned buffers, and needlessly owned state everywhere.

Use owned, `'static` futures only at the actual detachment boundary:

- Borrowed `async fn` for request/response operations that stay within one task.
- Owned handle or task API when the work must outlive the caller.

If you require `T: Send + 'static` on every public async entrypoint "just in case", you have pushed executor constraints into the wrong layer.

### 2. Protect the zero-cost budget

Turon's original futures design had a concrete cost target:

- no allocation per combinator
- about one allocation per task or connection
- one dynamic dispatch per wakeup/event, not per combinator edge

Use that as a smell test. If composition allocates at every branch, every callback registration, or every adapter hop, the abstraction is too high in the stack or using the wrong representation.

When multiplexing many child operations, preserve targeted wakeups. If one wakeup forces you to re-poll every child to find who made progress, you lost the "epoll for everyone" property and turned scheduler work into latency.

### 3. Separate capability from policy

Turon's futures and Tokio layering kept the lowest level general and cheap, then allowed opinionated layers above it.

At the foundational layer:

- expose capabilities
- preserve backpressure
- avoid choosing a runtime
- avoid choosing a buffer story unless the capability itself requires it

At higher layers:

- add buffering
- add retries and timeouts
- choose service abstractions
- integrate with a specific runtime or protocol stack

If the low-level trait decides buffering format, retry policy, and runtime integration all at once, the ecosystem cannot recombine it.

### 4. Preserve backpressure as part of the contract

Demand-driven async works because "not ready" can cascade backward through the system. That is a feature, not an implementation detail.

Before adding a convenience API, ask:

- Can the producer signal overload?
- Can the caller defer admission?
- Does cancellation stop work, or merely orphan it?

If the answer is "the callee always accepts and queues," expect memory growth, latency cliffs, and misleading benchmarks.

### 5. Make cancellation semantics explicit

Turon's early futures model made cancellation happen by dropping the future. That is powerful but ruthless: any await point may be the last one.

Design rule:

- abort-on-drop must leave local state sound
- required remote cleanup must be explicit, awaited protocol work

Use drop for synchronous cleanup and local state rollback. Use an explicit `close`, `shutdown`, or cancellation handshake when correctness depends on the other side acknowledging termination.

### 6. Think in compatible worlds before publishing traits

Turon's coherence work is the key public-API lesson most Rust authors miss: negative reasoning across crate boundaries is brittle because upstream crates can add impls in semver-compatible releases and downstream crates can add new local types forever.

Before publishing a trait or blanket impl, evaluate three worlds:

1. Current crate graph.
2. Upstream dependency adds a new impl in a minor release.
3. Downstream crate introduces a fresh local type and its own impls.

If your design only works in world 1, it is not a stable public API.

## Public Trait Heuristics

- Prefer local traits plus local wrapper/newtype types when you need future maneuvering room.
- Seal extension traits if you know you will need to add methods or blanket impls later.
- Treat every blanket impl as a semver commitment. The power to write `impl<T> Trait for Wrapper<T>` is a zero-sum game; once you take it, downstream crates lose options.
- Never rely on the current absence of a foreign impl as part of correctness or dispatch behavior.

## Return Type Heuristics

- Use `impl Trait` when the point of the API is behavioral abstraction and the concrete adapter stack is not part of the contract.
- Do not use bare `impl Trait` if callers will materially depend on secondary traits or auto traits. If `Send`, `Sync`, `Clone`, `DoubleEndedIterator`, or object safety matter, state them or use a named type.
- If a returned async abstraction must be stored, erased, or mixed heterogeneously, box once at the boundary. Boxing deep inside every combinator is the wrong cost placement.

## Async I/O Heuristics

- Do not make async traits inherit sync `Read`/`Write`-style traits for the sake of DRYness. Turon's futures 0.2 cleanup explicitly separated them and used adapters instead, because inheritance blurred blocking semantics and polluted the trait surface.
- If adding vectored or generic buffer methods makes the trait cease to be object-safe, that is usually a sign you mixed transport capability with buffering policy. Split them.
- Keep borrowed buffer APIs whenever the operation is task-local. Requiring ownership of buffers just to satisfy a hypothetical executor boundary is a design smell.

## Facade And Namespace Heuristics

Turon's module-system critique matters for library UX: filesystem layout is not the API. If users must trace private submodules and re-exports to discover what is stable, your public surface is under-designed.

- Use `pub(crate)` to state intent inside private modules.
- Keep public entrypoints shallow and deliberate.
- Re-export as an API decision, not as fallout from internal file layout.

## NEVER Do These

- NEVER force `'static + Send` onto every async API because one detached executor path wants it. It is seductive because one signature seems universally spawnable. Instead keep borrowed APIs at the leaf level and convert to owned work only where detachment actually happens, or callers will start cloning buffers, heap-owning state, and wrapping everything in shared ownership just to reach your boundary.
- NEVER hide overload behind unbounded internal queues because the call site looks cleaner. It is seductive because benchmarks on happy-path traffic often improve. Instead expose readiness, bounded admission, or explicit reservation so backpressure can propagate before memory blowups and latency cliffs appear in production.
- NEVER make correctness depend on async cleanup running from `Drop`. It is seductive because cancellation-by-drop feels automatic. Instead treat drop as local cleanup only, and add an explicit awaited shutdown path for remote or protocol-visible cleanup, or you will leave orphaned work and half-closed protocol state behind.
- NEVER specialize on lifetime-sensitive facts, repeated generic parameters, or the current absence of foreign impls because the code happens to compile today. It is seductive because type checking can often "see" more than codegen or future crate graphs will. Instead specialize only on always-applicable facts, or redesign the extension point around local types and traits, or you will create semver traps or typeck/codegen disagreement.
- NEVER publish a blanket impl without first naming the downstream impls it forbids. It is seductive because blanket impls feel like free ergonomics. Instead assume every blanket impl spends ecosystem budget and use sealing or wrappers when you still need room to evolve.
- NEVER let a foundational crate choose runtime, buffering, and retry policy all at once because the integration story becomes easy in the short term. It is seductive because demos come together quickly. Instead keep the base layer runtime-agnostic and put opinionated integrations one layer up.
- NEVER use bare `impl Trait` when hidden secondary guarantees matter. It is seductive because the signature looks elegant. Instead include the needed bounds or a named type, or you will accidentally freeze a weaker contract than the implementation can provide.
- NEVER expose internal module topology as the public namespace because re-exports make it easy to leak out. It is seductive because internal organization appears to define the API "for free." Instead design the namespace explicitly so internal refactors do not become breaking changes.

## Decision Tree

If you are choosing between two public shapes:

- Need background work that outlives the caller? Use an owned handle API.
- Need only task-local concurrency? Keep borrowed async signatures.
- Need runtime portability? Put runtime-specific adapters in a separate integration layer.
- Need heterogeneous storage? Box once at the outer boundary.
- Need downstream extension? Prefer local traits/newtypes; avoid blanket impls on foreign surfaces.
- Need ergonomic sync interop? Add adapters, not trait inheritance.
- Runtime API forces `'static` anyway? Offer a thin spawned wrapper above a borrowed core API rather than poisoning the core signature.
- Object safety fights vectored I/O? Split the trait into an object-safe core plus extension methods or adapters.

## Failure Recovery

If the API already shipped and feels wrong:

- If callers clone or `Arc` everything just to call you, move the `'static` boundary outward.
- If overload causes queue growth instead of `Pending`/readiness behavior, reintroduce admission control.
- If users cannot tell what is stable from the crate root docs, redesign re-exports before adding more modules.
- If semver fears block all changes, you probably exposed too much trait surface; add wrappers or sealing in the next major revision.

Turon's pattern is consistent: design the cheap, general, future-proof layer first; only then add the convenient, opinionated layer on top.
