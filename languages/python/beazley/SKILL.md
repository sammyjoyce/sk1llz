---
name: beazley-deep-python
description: "Protocol-first guidance for advanced Python in David Beazley's style: generator pipelines, send/throw/close semantics, descriptor precedence, metaclass and code-generation trade-offs, GIL behavior, and inspect-driven debugging. Use when working on streaming pipelines, yield-from refactors, generator-based coroutines, descriptors, metaclasses, import hooks, or subtle CPython behavior. Trigger keywords: generator pipeline, yield from, send, throw, close, descriptor, __set_name__, metaclass, __signature__, inspect, GIL, free-threaded."
tags: generators, coroutines, descriptors, metaclasses, inspect, cpython, concurrency, streaming
---

# Beazley Deep Python

Write advanced Python by reasoning from the protocol outward, not from syntax inward. The question is not "can Python do this?" but "which protocol am I binding myself to, and what does that force on shutdown, debugging, introspection, and performance?"

## Working Stance

- Start with the machinery: iterator protocol, generator protocol, descriptor precedence, import hooks, thread scheduling, or frame inspection. Beazley-style code stays readable because the author knows exactly which mechanism is carrying the abstraction.
- Prefer lazy dataflow when the data source is large, unbounded, or composable. In Beazley's 1.3 GB log example, the generator solution ran in 16.7s versus 18.6s for the manual loop because it avoided temporary materialization while keeping the pipeline declarative.
- Treat "clever" metaprogramming as a runtime budget question. If a trick runs on every attribute set or every object construction, benchmark it before admiring it.

## Mandatory Refresh Points

- Before changing `yield from`, `send()`, `throw()`, or `close()` behavior, refresh the current `PEP 380` and `PEP 479` semantics for the target interpreter.
- Before patching descriptor bugs, refresh the current descriptor precedence rules and `inspect` behavior for the target Python version.
- Do not go load metaclass or async material for plain file-streaming work; this skill is meant to stay mostly self-contained.

## Before You Write Code, Ask

- Before building a pipeline: is pull-based exhaustion compatible with the input, or do I actually need push, broadcast, or cancellation from outside the driver loop?
- Before using `yield from`: am I delegating the full generator protocol, including `send`, `throw`, and `close`, or only values?
- Before writing a descriptor: should instance assignment be able to override this attribute, or must the descriptor always win?
- Before blaming the GIL: am I on stock GIL-enabled CPython, a free-threaded build, or an interpreter where an extension may have re-enabled the GIL at runtime?
- Before adding metaclass magic: am I optimizing author ergonomics, runtime behavior, or both, and is that trade actually worth it?

## Decision Rules

### Streaming and Pipelines

- Use generator pipelines when stages are one-pass and the last stage can honestly drive the whole computation. The pipeline is only "simple" while one consumer pulls the stream.
- Do not end an infinite or tailing stream with exhausting sinks such as `sum()`, `min()`, `max()`, `set()`, or `list()`. Those are correct on finite iterables and pathological on live streams.
- The moment you need fan-out or multiplexing, a plain `for`-driven pipeline stops being the right abstraction. Broadcasting moves control flow into the broadcaster; multiplexing usually needs queues, threads, or processes because one loop cannot poll multiple generators fairly.
- If external cancellation matters, design an explicit shutdown channel. Beazley's generator examples work because the caller owns execution; production cancellation usually needs a flag, event, sentinel, or agreed exception.

### Generators, Coroutines, and Delegation

- Treat `yield from` as protocol delegation, not loop sugar. It forwards yielded values, `send()`, `throw()`, and `close()`, and the subgenerator's `return value` becomes the value of the `yield from` expression.
- `yield from` also propagates finalization. If the delegating generator is closed, the subiterator is closed too. That is correct for factored code and wrong for shared subiterators; if the subiterator is shared, wrap it or iterate explicitly.
- Use `return`, not `raise StopIteration`, to terminate a generator. After `PEP 479`, accidental `StopIteration` escaping a generator becomes `RuntimeError`, specifically to make these bugs noisy instead of silently truncating output.
- Bare `next()` inside generator code is suspicious. If exhaustion is expected, catch `StopIteration` in the generator itself; catching it in a helper does not preserve the right generator semantics.
- `close()` is for bail-out and cleanup, not for "return the final answer". `PEP 380` explicitly rejects using `close()` as an end-of-stream result channel; use a sentinel or an agreed exception instead.
- Catch `GeneratorExit` only to release resources and then return immediately. If you keep yielding after `GeneratorExit`, the runtime raises `RuntimeError`.
- Generator-based coroutines need priming with `send(None)` unless you hide it behind a decorator. Do not cargo-cult that rule into `async def`; it applies to generator receivers, not native coroutines.

### Descriptors and Metaprogramming

- Descriptor choice starts with precedence, not with API aesthetics. If instance state must not shadow the behavior, make it a data descriptor; if per-instance override is a feature, keep it non-data.
- `__set_name__()` happens during class creation. If you attach a descriptor after the class already exists, Python will not backfill the name for you; call `__set_name__` manually or your descriptor will behave half-configured.
- `__signature__` is useful for REPL ergonomics, documentation tools, and friendlier constructors, but `inspect.signature()` treating it as authoritative is a CPython implementation detail. Use it for human-facing introspection, not as your only portability contract.
- Beazley's metaclass examples are a warning about hidden runtime taxes: a signature-heavy metaclass turned object creation from about 1.07s to 91.8s, and even the code-generated recovery path still cost about 17.6s. If construction or assignment is hot, plain classes, dataclasses, or generated methods beat repeated Python-level dispatch.
- Repeated `super().__set__()` chains inside descriptors look elegant and can be disastrously expensive. On hot paths, flatten the validation steps or generate the combined setter.

### Concurrency and the GIL

- Threads are still a good tool when work overlaps blocking I/O or C/extension code that releases the GIL. The subtle failure mode is mixed CPU and I/O load, not simply "threads bad".
- Measure mixed-load latency, not just CPU throughput. Beazley's GIL traces showed roughly 17,000 ticks between UDP arrival and `recvfrom()` resuming, and more than 34,000 ticks before the loop returned to `recvfrom()` under contention.
- His 3.2-era "better GIL" data is the right mental model for convoy effects: a 10 MB echo test competing with one CPU thread took 12.4s versus 0.57s, and 46.9s with two CPU competitors. The lesson is that fairness changes can improve CPU sharing while destroying I/O latency.
- On Python 3.13+ do not assume old GIL folklore is the whole story. Free-threaded builds exist, but they are not the default, extensions can re-enable the GIL at runtime, and you should check `sysconfig.get_config_var("Py_GIL_DISABLED")` and `sys._is_gil_enabled()` before committing to an architecture.

### Introspection and Debugging

- Use `inspect.getattr_static()` when debugging descriptors, wrappers, or proxy objects. Plain `getattr()` can execute the very code you are trying to inspect and hand you a lie.
- Use `inspect.getgeneratorstate()` to distinguish `GEN_CREATED`, `GEN_RUNNING`, `GEN_SUSPENDED`, and `GEN_CLOSED` before rewriting a scheduler or blaming cancellation.
- If wrapper signatures are confusing you, inspect the callable with `follow_wrapped=False` before touching `__signature__`; otherwise you may debug the wrapped function instead of the wrapper contract.
- If you reach for frame hacks, clean up after yourself. Keeping frame references alive creates reference cycles and stretches object lifetimes in ways that look like leaks.

## NEVER Do These

- NEVER raise `StopIteration` to end a generator because it feels like "the iterator way"; in modern Python it turns subtle truncation bugs into `RuntimeError`, and `yield from` refactors can silently skip cleanup. Instead `return`, and catch expected exhaustion exactly where the generator needs it.
- NEVER treat `yield from` as a prettier `for` loop because the seductive syntax hides that `close()` and `throw()` are forwarded too, which can finalize a shared subiterator or change exception flow. Instead use explicit iteration whenever isolation matters more than elegance.
- NEVER catch `GeneratorExit` and continue yielding because it looks like a convenient shutdown hook; the runtime treats that as refusing finalization and raises `RuntimeError`. Instead release resources and exit the generator immediately.
- NEVER call `.close()` on a generator from another thread because it looks like cancellation; if the generator is already executing you get `ValueError: generator already executing`, and signal handlers have the same trap. Instead feed a shutdown event, sentinel, or cooperative cancellation flag that the owning loop checks.
- NEVER ship descriptor or metaclass machinery just to remove a little boilerplate because the abstraction win is seductive and the runtime tax lands on every instance or attribute access. Instead prove the ergonomics matter, then benchmark construction and assignment before committing.
- NEVER debug descriptor-heavy objects with plain `getattr()` because observing the object can trigger properties, `__getattr__`, or binding logic and hide the bug. Instead start with `inspect.getattr_static()` and only invoke the descriptor deliberately afterward.

## Quick Routing

- Need one-pass streaming over large but finite data: use a generator pipeline.
- Need a live stream or `tail -f` style source: keep stages incremental and end with side-effect consumers, not exhausting reducers.
- Need broadcast or multiplexing: switch to explicit `send()` consumers or queue/thread/process boundaries.
- Need assign-time validation that instances must not override: data descriptor.
- Need method-like behavior or per-instance override: non-data descriptor.
- Need ergonomic constructors around dynamic fields: generate code before reaching for a metaclass with heavy runtime enforcement.
- Need CPU parallelism on stock CPython: use processes or native code that releases the GIL.
- Need I/O overlap: threads or async are both viable; benchmark with mixed CPU and I/O load if latency matters.
