---
name: pike-simplicity-first
description: "Apply Rob Pike's simplicity-first Go heuristics for exported API shape, concurrency structure, package boundaries, and dependency restraint. Use when designing or refactoring Go libraries, public types, channel or goroutine pipelines, naming or package layouts, config or option patterns, or compatibility-sensitive changes. Triggers: rob pike, pike, simplicity, goroutines, channels, interfaces, context, package names, options, exported api, go library design."
---

# Pike Simplicity First

Use this skill when the hard part is not writing Go syntax but keeping the package easy to extend, easy to call, and easy to stop. The goal is not "idiomatic-looking Go." The goal is deleting irreversible complexity before it escapes into the API.

## Load only what matches the task

- Before changing exported functions, methods, interfaces, structs, or constructors, READ `references/api-evolution.md`.
- Before adding goroutines, channels, pipeline stages, cancellation, or shutdown logic, READ `references/concurrency-and-shutdown.md`.
- Before renaming packages, adding dependencies, or choosing config or option shapes, READ `references/surface-area.md`.
- Do NOT load `references/philosophy.md` for normal implementation work; it is background and quotes, not the operational playbook.
- Do NOT load any reference file for leaf-local bug fixes that do not affect exported API, concurrency, or package boundaries.

## Core lens

- Treat simplicity as a systems property, not a minimal-lines trick. Pike's standard is severe: even a 2 percent speedup usually does not justify 0.1 to 2 percent more system complexity.
- Optimize the caller's mental load, not the author's convenience. Fewer decisions at the call site beat abstract machinery hidden behind "flexibility."
- Prefer functions and concrete values over early taxonomies. Go's interfaces worked because algorithms came first and type hierarchies came later.
- Use concurrency to express ownership, cancellation, and composition. Parallel speedup is optional; leak-free structure is not.

## Before you change the design, ask yourself

### Exported API

- What decision can I remove from the caller entirely?
- Am I about to freeze a method set I do not own?
- If I need one more method or parameter next year, can I add it without breaking old code?
- If I add a field later, will its zero value preserve the old behavior?

### Concurrency

- Which goroutine owns this state?
- How does every started goroutine learn to stop?
- If a downstream stage returns early, how are blocked upstream senders released?
- Is a channel expressing ownership transfer, or am I using it as an elaborate mutex?

### Package surface

- Does the package name make selectors shorter and clearer at the call site?
- Would copying 20 lines be cheaper than a permanent dependency and trust edge?
- Is this option or config mechanism helping the user, or just making me feel future-proof?

## Decision rules

### API shape

- Return concrete types from constructors unless you explicitly need third-party implementations. Concrete returns let you add methods later; exported interfaces do not.
- Define interfaces at the consumer boundary. If a package exports an interface only for tests or "future flexibility," it is usually freezing the wrong abstraction too early.
- If a stable API needs cancellation or deadlines later, add `FooContext` or `QueryContext` style siblings and let the old API delegate with `context.Background()`.
- If behavior may gain more knobs, prefer a config struct or receiver-based config type over exploding function signatures. New struct fields are usually compatible if their zero value preserves prior behavior.
- If you export a value type directly, remember comparability is part of the contract. Adding an uncomparable field later breaks `==` and map-key users.
- If you want callers to copy values while hiding representation, return opaque values with unexported fields instead of forcing pointers or interfaces.

### Concurrency shape

- Channels are best when ownership moves with the message or when cancellation is part of the protocol.
- A mutex or atomic is often simpler when you are only protecting shared in-memory state and no ownership transfer exists.
- Pipeline stages should close their outbound channels, keep receiving until inbound closes or senders are unblocked, and accept cancellation on every send path.
- Use channel close as broadcast when the number of blocked senders is unknown. A fixed buffer only works when you can prove the exact outstanding count.
- Treat `select` defaults as a code smell in long-lived loops. An empty default turns a blocking protocol into a spin loop.
- `defer` is function-scoped, not loop-scoped. In workers that may run forever, put cleanup on the actual exit path, not inside the steady-state loop.

### Surface area

- Package names are part of every call site. If the only honest name is `util`, `common`, or `base`, the boundary is probably wrong.
- Avoid stutter in import paths and exported names. Callers should not type the same concept twice.
- Prefer a little copying to a little dependency when the dependency would add long-term trust, review, and upgrade cost for trivial reused code.
- If you use functional options, do it because options will genuinely grow or because temporary reversible state is valuable. Otherwise plain arguments or a config struct are simpler.
- When options must be temporary, Pike's self-referential option pattern is stronger than write-only setters because it can return the inverse for `defer`-based restoration.

### Errors

- Do not equate "errors are values" with "write `if err != nil` after every line." Design APIs so the mainline stays readable and the error is checked at the semantic boundary.
- If a loop's purpose is iteration, consider `Scanner`-style APIs that separate iteration from final error reporting rather than forcing error handling on every step.

## Never do these

- NEVER export producer-owned interfaces "for mocking" because it feels abstract and testable, but it freezes a method set you cannot extend later. Instead return concrete types or opaque values and let consumers define the smallest interface they need.
- NEVER unstick a blocked pipeline with a magic channel buffer because it is the fastest patch and often makes tests green, but the required capacity depends on the exact number of abandoned senders and breaks silently when topology changes. Instead thread `context.Context` or a `done` channel through every stage and use close as broadcast.
- NEVER store `context.Context` in a struct because constructor injection looks tidy, but it smuggles one request lifetime into another and makes per-call deadlines and cancellation impossible. Instead accept `ctx` as the first parameter on request-scoped calls.
- NEVER add a dependency for a one-function convenience because it looks cheaper than writing code, but every dependency is a permanent trust and upgrade edge. Instead copy the tiny, obvious code or stay inside the standard library when the abstraction is not carrying real weight.
- NEVER spend design complexity to win a microbenchmark because the number looks objective, but system complexity compounds faster than local speedups. Instead measure on real workloads and quarantine unavoidable complexity behind a tiny internal boundary.
- NEVER leave `for { select { default: } }` or per-iteration `defer` cleanup in long-lived workers because the code feels responsive and structured, but it spins the scheduler and the defers never fire. Instead block, use a ticker or condition, and place cleanup on the goroutine's real return path.
- NEVER adopt functional options by default because they look extensible and idiomatic, but they inflate package namespace and make duplicate or ordering semantics harder to reason about. Instead start with plain arguments or a config struct; escalate only when growth or reversible configuration is real.

## Fallbacks

- If a bad interface already escaped into the public API, add a new sibling interface or dynamic type-check path instead of mutating the old interface in place.
- If channel ownership is unclear after two passes, simplify: collapse stages, use a worker function plus callback, or use a mutex. Confused ownership is a design smell, not a documentation problem.
- If package naming is contentious, sketch three real call sites and pick the name that makes those selectors shortest and least repetitive.
- If a performance change needs extra machinery, keep the simple version beside the optimized path until benchmarks on production-like inputs prove the complexity earned its keep.
