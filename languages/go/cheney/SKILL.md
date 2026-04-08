---
name: cheney-practical-go
description: Pragmatic Go API and code design in Dave Cheney's style: shrink public surface area, make the default path trivial, treat errors as API, and measure before optimizing. Use when writing or reviewing production Go, especially constructors, package boundaries, interfaces, error flows, logging, goroutine lifecycles, tests, or performance-sensitive code. Triggers: functional options, small interfaces, context misuse, internal packages, goroutine leaks, benchmarks, package-level logger, same-type parameters, streaming APIs.
tags: go, api-design, errors, interfaces, logging, concurrency, testing, performance
---

# Cheney Practical Go

This is a philosophy/process skill, not a Go tutorial. Apply it at package and API boundaries, where small mistakes turn into permanent maintenance cost.

## Operating Stance

- Start at the package boundary, not the implementation. Every exported name is a support contract; if two packages do not need it today, keep it unexported or move it under `internal/`.
- Design the default case first. If the common path still requires `nil`, an empty config, or a dummy callback, the API is wrong before the implementation starts.
- Accept the narrowest behavior you consume, as close as possible to the call site. Small interfaces are stronger because they reveal less implementation detail and are harder to fake badly.
- Return concrete types from constructors and factories. Returning interfaces hides useful behavior, freezes callers behind your abstraction, and makes later evolution harder.
- Prefer streaming and caller-owned buffers on hot or repeated paths. An allocation hidden in the API is a tax every caller pays forever.
- Keep concurrency behind a synchronous surface unless the asynchrony is the feature. A cheap goroutine is still a resource leak if nobody owns its shutdown path.

## Before You Change an API, Ask Yourself

### Package Boundary

- Is this name solving a real cross-package need, or am I exporting for tests and "future reuse"?
- Can `internal/` preserve reuse inside the module without turning helpers into public policy?
- Does the package name describe capability, or is it a junk drawer like `util`, `common`, `base`, or `helpers`?

### Constructor and Options

- Is the default path a single obvious call?
- If there is exactly 1 exceptional mode, would a second constructor read better than options?
- If there are 2 or more orthogonal knobs, should options exist, and can option validation fail during construction?
- Am I about to make callers pass `nil`, zero structs, or shared mutable config pointers just to get defaults?

### Interface Choice

- Which exact behavior is consumed here: 1 method, 2 methods, or a concrete type?
- Am I introducing an interface for real substitution, or only to make mocking easier?
- If this function takes two parameters of the same type, how will a reviewer prove they are in the right order at the call site?

### Error and Logging

- Is this error part of my public contract, or only internal context for operators?
- Who is the first layer that can actually make a recovery decision?
- If I log here, am I handling the error completely, or just creating duplicate noise before returning it anyway?

### Concurrency and Performance

- Who stops this goroutine, and who waits for it to finish?
- Is the bottleneck proven with `go test -bench` or `pprof`, or am I optimizing folklore?
- Will this abstraction allocate, escape to the heap, or cross the cgo boundary in a tight loop?

## Decision Rules

| Situation | Prefer | Avoid | Why |
|---|---|---|---|
| One normal mode, one oddball mode | 2 explicit constructors | Functional options by reflex | Options add indirection; a second constructor keeps the default path obvious |
| Several independent knobs | Functional options that can return errors | `*Config` with `nil` meaning defaults | `nil` and empty config are ambiguous, and pointer configs invite aliasing after construction |
| Repeated data transfer or large payloads | `io.Reader`/`io.Writer` or caller-supplied buffers | APIs that return freshly allocated `[]byte` each call | Hidden allocations become permanent GC pressure and latency variance |
| Boundary only needs write semantics | `io.Writer` or a tiny local interface | `*os.File` or `io.ReadWriteCloser` out of habit | Over-wide parameters leak irrelevant capability and make testing harder |
| Caller must branch on failure | Opaque wrapped errors plus `errors.Is`/`errors.As` at real boundaries | Exporting many sentinel vars and custom types | Errors are API surface; every exported sentinel couples importers to internals |
| Suspected order dependence in tests | Map-backed table tests with subtests | Slice tables only because they are conventional | Randomized map iteration flushes hidden global state and order coupling |
| Performance investigation | `go test -run=^$ -bench=. -benchmem`, then `pprof`, then `-gcflags=all=-m -m` | "defer is slow", "mutexes are slow", or other cargo cult rules | Cheney's rule is measure first; folklore ages faster than the compiler |

## Anti-Patterns Cheney Would Push Back On

- NEVER accept `*Config` plus `nil` for defaults because it feels flexible while actually encoding two states (`nil` and empty) that callers cannot reason about cleanly. Instead make the default constructor obvious and use explicit constructors or validated options for true variations.
- NEVER add an interface only so tests can mock it because that seduces you into designing around a fake instead of the dependency's real behavior. Instead keep the concrete dependency until a consumer proves a narrow interface is needed.
- NEVER return an interface from `New...` because it looks abstract but locks callers behind the weakest possible view and makes additive evolution harder. Instead return the concrete type and let consumers define their own tiny interfaces where they use it.
- NEVER put loggers, metrics clients, or service handles in `context.Context` because it hides required inputs in an untyped bag and fails at runtime on rarely tested error paths. Instead inject dependencies explicitly and reserve context for cancellation, deadlines, and request-scoped metadata that truly crosses API boundaries.
- NEVER keep package-level loggers or other mutable singletons because the convenience is seductive while the real result is transitive compile-time coupling and hard-to-control side effects. Instead pass the minimum dependency each type needs.
- NEVER log and return the same error because logging is already handling it, so returning it creates duplicate, decontextualized noise upstream. Instead either recover locally and log the fallback, or wrap and return so one boundary handles it once.
- NEVER start a goroutine without a stop condition and owner because "goroutines are cheap" hides the fact that leaked goroutines keep stacks, timers, sockets, and reachable heap alive. Instead define who cancels, who closes, and who waits.
- NEVER use cgo in a tight loop because the call boundary consumes a thread like blocking I/O and can erase the point of writing the code in Go. Instead batch the boundary crossing, move the hot path to Go, or admit the workload belongs elsewhere.
- NEVER accept two non-commutative parameters of the same type because the call site reads fine while still being one swap away from a bug reviewers cannot see. Instead introduce a helper type, split constructors, or encode the roles in method names.

## Fallbacks and Edge Cases

- Sentinel errors are acceptable for protocol markers like `io.EOF`; the trap is turning every internal branch into a public sentinel.
- If an interface naturally grows beyond 1 method, place it on the consumer side and name it after the behavior needed there, not after the producer's entire capability set.
- If a background goroutine truly lives until process exit, say so in the type's contract; "runs forever" is fine only when ownership is explicit.
- Use map-backed table tests only when hunting order dependence. For snapshot-heavy tests where deterministic diffs matter more, keep slice-backed tables and add subtest names.
- If benchmark numbers are noisy, fix the environment before changing code: isolate the benchmark, record allocations, compare multiple runs, then inspect escape analysis. Do not tune against laptop fan noise.
- If a profile says the hot path is allocation-heavy, look for API shape first. A better buffer ownership rule usually beats micro-optimizing the loop body.

## Done Means

- The exported surface is smaller or more intentional than before.
- The default path is the shortest and clearest call site.
- Each interface describes behavior actually consumed, usually 1 method.
- Errors are either handled once or returned with context, never both.
- Every goroutine has an owner, shutdown trigger, and wait story.
- Performance claims are backed by measurement, not aphorisms.
