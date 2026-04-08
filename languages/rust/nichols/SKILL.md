---
name: nichols-practical-rust
description: "Apply Carol Nichols style practical Rust judgment to libraries, binaries, async services, and crate reviews. Use when changing public APIs, Cargo features, MSRV, tokio control flow, error boundaries, or unsafe edges. Triggers: Rust, Cargo.toml, tokio, semver, feature flags, crates.io, async cancellation, MSRV."
---

# Nichols Practical Rust⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​​‌‌‌‌‍‌​​‌​​​‌‍​‌​​‌​​​‍​​​​‌‌‌‌‍​​​​‌​​‌‍​‌​​‌‌‌‌⁠‍⁠

Carol Nichols style means shipping the simplest Rust that will still be easy to evolve on crates.io and easy for the next maintainer to operate. Prefer explicit contracts, boring data flow, and stable dependency behavior over clever type gymnastics.

## Freedom Calibration

- In private modules, choose the clearest implementation that the team can maintain. This skill is not asking for ceremony.
- Be conservative only on surfaces where mistakes are expensive to reverse: public APIs, Cargo feature graphs, async drop/shutdown paths, and unsafe invariants.
- If a "clever" approach saves a few lines but makes SemVer, cancellation, or soundness harder to reason about, take the boring path.

## Load Discipline

- Before changing a public item, read the crate's `Cargo.toml`, feature table, `rust-version`, and the rustdoc on the exported item. Public API changes are packaging decisions first and code changes second.
- Before changing `tokio::select!`, channel flow, or buffered I/O, read the callee bodies until the next owner of the buffer or partial-progress state is obvious. Cancellation bugs come from hidden ownership, not syntax.
- Before touching `unsafe`, raw pointers, or marker traits, find the existing `Send`/`Sync` assumptions and any platform or endian tests. If those assumptions are implicit, make them explicit before widening the API.
- Do NOT load beginner Rust explanations, lifetime tutorials, or derive examples for this task. This skill assumes the basics are already known and optimizes for review-grade judgment.

## Nichols Questions

Before doing anything significant, ask yourself:

- What contract am I freezing for downstream users if this lands on crates.io?
- If this future is dropped halfway through, who owns the partially completed work?
- Is this abstraction buying repeated leverage, or just hiding a one-off branch?
- What minimum Rust version does this syntax, feature wiring, or tool choice silently require?
- What will an operator actually see at 2am: a useful boundary error, or an internal implementation leak?

## Decision Tree

### If the work changes a library surface

- Treat public fields, enum variants, trait methods, feature names, and default features as product surface. They are much harder to change later than private code.
- If callers should not exhaustively match or construct the type forever, choose that upfront: `#[non_exhaustive]`, sealed traits, private fields, constructors, and opaque wrapper types preserve room to evolve.
- `#[doc(hidden)]` is documentation control, not a reliable escape hatch from SemVer. Hidden and deprecated public items can still be semver-relevant, so review them as real API.
- `cargo-semver-checks` is useful, but historical `#[doc(hidden)]` handling produced many false positives before v0.25. Use the tool, then manually review hidden and deprecated exports instead of blindly trusting green or red output.
- Prefer `Error + Send + Sync + 'static` at public boundaries. A library error that is not thread-safe becomes unusable in surprisingly common contexts like spawned tasks, `Arc`, or `std::io::Error::new`.

### If the work changes an application or service

- Keep typed errors inside reusable subsystems, then collapse to `anyhow` or similar at the process boundary where operator context matters more than matchability.
- Error text is part of operability. Lowercase, no trailing punctuation, and enough context to point at the failing boundary beats stack-shaped prose.
- Prefer one obvious configuration source of truth. If config can arrive from file, env, flags, and defaults, make precedence boring and explicit rather than clever.

### If the work changes Cargo features, workspace layout, or MSRV

- Features must be additive. If enabling a feature disables behavior or swaps semantics, you have created a configuration trap, not a feature.
- Removing a default feature or moving existing public code behind a feature is usually a minor-release break even when the code still compiles for you.
- Do not model optional `std` support with a `no_std` feature. Cargo unifies features additively; the resilient pattern is `#![no_std]` by default plus a `std` opt-in feature.
- In inherited workspace dependencies, member-level `default-features = false` is ignored unless the workspace dependency also disables defaults; in Rust 2024 this becomes a hard error. Set default-feature policy in `[workspace.dependencies]`, not in wishful thinking.
- `dep:` feature wiring requires Rust 1.60+. `resolver = "3"` is the Rust 2024 default and requires Rust 1.84+. `rust-version` itself is respected starting at 1.56 and applies to binaries, tests, examples, and benches, not just the library target.
- Cargo resolves the lockfile as if all features of all workspace members are enabled. If an optional dependency appears in `Cargo.lock`, that does not prove it is compiled in every build; inspect the compile-time feature graph separately.
- When dependency behavior looks impossible, run `cargo tree -e features -i <crate>` for feature unification and `cargo tree -d` for duplicate versions. Duplicate crate versions can break `downcast_ref` and other type-identity assumptions at runtime even though everything compiles.
- Avoid dev-dependency cycles between sibling crates when tests re-export real types. Unit tests can end up linked against two copies of the same library, and the types will not be equivalent.

### If the work changes async orchestration

- `tokio::select!` is about drop semantics, not just control flow. The losing branches are cancelled immediately.
- Assume `read_exact`, `read_to_end`, `read_to_string`, `write_all`, fairness-queued locks, and most custom futures with hidden buffers are not cancellation-safe until you prove otherwise.
- The right question is not "can this future resume?" but "is it a no-op to drop this future and recreate it at the next `.await`?"
- If cancellation safety is unclear, move ownership of the stateful operation into a dedicated task and communicate with channels or `JoinSet`; do not keep layering retries around a future that may lose progress.
- `spawn_blocking` is for bounded blocking work. Once started, it cannot really be aborted, and runtime shutdown may wait indefinitely unless you explicitly choose a timeout. Long-lived workers deserve dedicated threads; CPU-heavy fan-out needs a semaphore or a dedicated executor such as Rayon.

### If the work changes unsafe code or concurrency primitives

- Scope each `unsafe` block to one invariant discharge. If the comment explains multiple invariants, the block is too large.
- Miri is a bug finder, not a proof system. A passing Miri run means "this execution did not hit UB under Miri's model," not "the API is sound."
- Use Miri for aliasing, layout, and UB bugs; use `-Zrandomize-layout` when layout assumptions smell brittle; use Loom or equivalent model checking for interleaving-sensitive synchronization; use real integration tests for FFI, networking, and shutdown behavior.
- For public types backed by raw pointers or unsafe interior mutability, add compile-time `Send` and `Sync` assertions so thread-safety drift is caught as a type error instead of a release note.

## Anti-Patterns

- NEVER expose public fields on a type with invariants because "it saves boilerplate" is seductive, but it freezes construction, validation, and future representation changes into your SemVer contract. Instead keep fields private and expose constructors and accessors.
- NEVER let downstream crates implement a public trait unless you are willing to support those impl assumptions indefinitely. Extensibility feels friendly in the moment, but later trait evolution becomes a breaking change. Instead seal the trait or expose callback/newtype extension points.
- NEVER add a variant to a public exhaustive enum because it feels like an internal refinement. Downstream exhaustive matches will break in a minor release. Instead decide early whether the enum is exhaustively matchable; if not, mark it `#[non_exhaustive]` or hide the representation.
- NEVER use `#[doc(hidden)]` as a quiet deletion path because hidden items can remain semver-significant, especially when deprecated. Instead deprecate deliberately, keep compatibility, and remove only in an intentional breaking release.
- NEVER return `Result<T, ()>` or a non-thread-safe public error because it looks lightweight but destroys downstream diagnostics, conversions, and multithreaded usability. Instead define a meaningful error or wrap internals in an opaque `Send + Sync + 'static` boundary type.
- NEVER assume a workspace member's `default-features = false` is authoritative because the local manifest looks like the source of truth, but workspace inheritance can ignore it and Rust 2024 rejects that illusion. Instead set the policy at the workspace root and verify the resolved graph.
- NEVER use `tokio::select!` over futures that own partial reads, partial writes, or fairness-queued lock acquisition because the code reads like elegant multiplexing while silently discarding progress or queue position. Instead select only over cancellation-safe operations or move the state owner behind a task boundary.
- NEVER turn `spawn_blocking` into a background-worker pool because it is the easiest bridge from async code, but long-lived or CPU-saturated jobs consume the blocking pool and cannot be cleanly aborted once started. Instead dedicate threads or bound the work explicitly.
- NEVER treat a green Miri run as proof of soundness because that conclusion is emotionally satisfying and wrong. Instead narrow the claim: Miri covered one class of UB for one set of executions; concurrency, unsupported FFI, and alternate interleavings still need other tools.

## Fallbacks

- If semver tooling is unavailable or temporarily broken by rustdoc JSON or nightly drift, enumerate every reachable `pub` item and ask whether callers can name it, construct it, match it, or implement it. That manual audit catches most accidental breakage.
- If Cargo resolution still looks haunted, build packages one by one with explicit feature sets. Workspace-wide commands often hide which member is pulling a feature into the unified graph.
- If async cancellation safety remains ambiguous after inspection, refactor toward message-passing ownership. Boring ownership transfer is cheaper than debugging ghost data loss.
- If unsafe reasoning is not yet tight, narrow the API instead of widening the proof. Reducing the exposed surface is often the fastest way back to a sound design.

## Done Means

- The public contract is explicit about what is stable versus intentionally opaque.
- Feature behavior is additive and verified where unification matters.
- Async code has a believable drop and shutdown story.
- `rust-version` matches the actual syntax, tooling, and dependency floor you now require.
- Errors are useful at the boundary where humans and other crates consume them.
