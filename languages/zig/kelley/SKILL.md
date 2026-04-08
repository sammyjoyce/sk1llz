---
name: kelley-zig-philosophy
description: "Write Zig in Andrew Kelley's style: explicit ownership, visible control flow, compile-time used for proof or specialization instead of cleverness, and ABI-aware data layout. Use when writing or reviewing Zig systems code, choosing allocators and lifetime models, debugging comptime or result-location bugs, or designing FFI/MMIO boundaries. Triggers: zig, comptime, allocator, ownership, lifetime, packed struct, extern struct, sentinel slice, OutOfMemory, result location semantics, ArrayList.items."
---

# Andrew Kelley Zig Philosophy⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​‌‌​‌‌‍‌​‌‌​‌​‌‍‌‌‌‌​‌​‌‍​‌​‌‌‌​‌‍​​​​‌​‌​‍‌‌‌‌‌​‌​⁠‍⁠

Kelley-style Zig is not "metaprogram aggressively because the language lets you." It is "make the bytes, control flow, and failure modes obvious enough that someone can predict the machine consequences by reading the source."

## Working stance

- Optimize for legibility under systems pressure: bounded memory, freestanding targets, FFI, MMIO, and postmortem debugging.
- Favor proofs the compiler can enforce over conventions humans can forget.
- Treat compile-time execution as a tool for specialization and validation, not as a second runtime.
- Push policy outward. If an allocator, ownership rule, or layout choice belongs to the caller, do not hide it in the callee.

## Before writing Zig, ask yourself

### Ownership

- Where are the bytes right now: global constant data, stack, allocator-owned heap, caller buffer, or container storage?
- Who frees them, and what mutation invalidates the current view?
- If I return this pointer or slice, can the caller outlive it or resize the backing container?

### Specialization

- Does `comptime` remove branches, types, layout decisions, or whole code paths?
- Or am I only moving ordinary data work from runtime into compile time and making every build pay for it?
- If this is a public API, will specialization improve clarity, or will it multiply instantiations and make errors worse?

### Representation

- Is this normal in-memory data, a C ABI contract, or a bit-exact register or wire image?
- Do I need an owned allocation, or would a caller-provided buffer or returned value keep policy out of the API?
- Am I choosing a sentinel type because the sentinel is semantically required, or just because C is nearby?

## Decision rules that matter in practice

### Allocators are policy, so libraries do not pick one

- Library boundary: accept `Allocator` or write into caller-owned storage.
- Known upper bound at comptime: use `FixedBufferAllocator`.
- One-shot CLI, frame, or request lifetime where everything dies together: use `ArenaAllocator`.
- Tests: use `std.testing.allocator`; when OOM handling matters, add `std.testing.FailingAllocator`.
- Application-level general allocation: keep one allocator near `main`; in debug builds, `DebugAllocator` is the default pressure test. In `ReleaseFast`, `smp_allocator` is a reasonable general fallback.

### Handle OOM even on Linux

- Zig's standard failure mode is `error.OutOfMemory`, not "the process will probably crash anyway."
- Overcommit is not universal, and on Linux it can degrade into OOM-killer roulette rather than a clean failure.
- Kelley-style code treats OOM handling as part of portability: embedded, RT, Windows, tests, and reusable libraries all benefit.

### Use `comptime` to prove or specialize, not to show off

- If compile-time work only computes ordinary data, keep it runtime unless specialization changes generated code or proves a real invariant.
- The default `@setEvalBranchQuota` budget is `1000` backwards branches. Needing to raise it is a design review trigger, not a badge of honor.
- `inline` and `inline for` are for compile-time-known structure. Using them to unroll mundane data paths usually buys code size and compile-time cost, not better design.

### Prefer value semantics until pointer semantics are truly required

- Zig may pass aggregates by value or by reference, whichever is cheaper. Treat parameters as values and any address derived from them as ephemeral.
- Returning structs by value is normal Zig, not "expensive C". Result-location semantics often let Zig construct directly in the destination.
- Before introducing out-pointers, ask whether you are solving a real pinning or mutation need or just importing C habits.

### Result-location semantics are a real design constraint

- `.{ ... }` can write directly into the destination. That is why `arr = .{ arr[1], arr[0] }` is not a swap.
- Typed initializers `T{ ... }` do not propagate result locations. Use that fact, or an explicit temporary, when in-place construction would alias the old value.
- If an initializer reads from the object it is overwriting, stop and decide whether you want in-place writes or a temporary. Do not let syntax choose for you.

### Layout choice is semantic, not aesthetic

- Default `struct`: ordinary in-memory data; let the compiler reorder and pad.
- `extern struct`: exact C ABI contract.
- `packed struct`: exact bit layout only when you truly mean "this value is a register or wire image."
- Packed layout still does not erase byte-order concerns. Andrew's bit-field design defines layout, but fields wider than 8 bits behave differently depending on byte alignment, so "packed" is not a substitute for thinking about endianness.

### Slices, sentinels, and borrowed container views have sharp edges

- `[:0]T` guarantees a sentinel at index `len`; it does not promise there are no earlier sentinel bytes.
- Sentinel slicing checks the promised sentinel and traps if the backing data does not actually contain it at that position.
- `std.ArrayList.items` is a borrowed view whose lifetime ends at the next resize. Hand it out only if you freeze mutation or transfer ownership first.

## NEVER do these

- NEVER smuggle an allocator choice into a library because "page allocator is fine for now." That hides policy, breaks bounded-memory and freestanding callers, and makes tests less meaningful. Instead accept an `Allocator` or a caller-owned buffer.
- NEVER paper over comptime blowups with `@setEvalBranchQuota` because the seductive part is that the compile error disappears. The concrete cost is slower builds and more generated code for work that probably belonged at runtime. Instead keep only proof, layout, and codegen at comptime.
- NEVER mutate individual fields through a `*volatile packed struct` because field syntax becomes read-modify-write on bits and is not atomic for MMIO. Instead build a full register value and store it once through the volatile pointer.
- NEVER keep `&param` or a borrowed view such as `ArrayList.items` beyond the immediate scope because Zig is free to pass aggregates by reference or copy, and container resizes invalidate borrowed slices. Instead return values, freeze mutation, or transfer ownership explicitly.
- NEVER use `[:0]const u8` as your default internal string type because C interop makes it feel convenient. The non-obvious consequence is that you import sentinel invariants and runtime sentinel checks into code that only needed a byte slice. Instead keep `[]const u8` internally and convert at the boundary.
- NEVER rewrite an aggregate with `x = .{ ... x ... }` unless you have reasoned about result-location semantics. The seductive part is that it looks like constructing a fresh value; the consequence is silent aliasing and wrong answers in swap-like code. Instead use a temporary or a typed initializer when you need separation.
- NEVER normalize `.?` or `catch unreachable` during bring-up because it feels like a quick way to state intent. The hidden cost is that ordinary absence or failure turns into safety traps and a worse public API. Instead preserve `?T` and `error!T` until the invariant is genuinely local and proven.
- NEVER assume "OOM cannot happen here" because Linux overcommit exists. The consequence is nondeterministic process death on some systems and unusable libraries on others. Instead propagate `error.OutOfMemory` and test that path.
- NEVER reach for recursion on unbounded input because the seductive part is clean code. The concrete consequence is unbounded stack growth, and Zig does not magically rescue you from that today. Instead use an explicit stack or bounded arena when depth is data-dependent.

## Fallbacks when the design is unclear

- If ownership is muddy, redesign the API to return a value or write into caller-provided storage. "Document it better" is not the first fix.
- If packed or bit-cast code behaves strangely, replace reinterpretation with explicit encode or decode first. Reintroduce packed layout only after endian and MMIO behavior are proven.
- If comptime and runtime are both viable, ship the runtime version first and add specialization only where it deletes real work or proves invariants.
- If performance work is forcing `@setRuntimeSafety(false)`, first get the code correct in safety-enabled tests and name the invariant you are removing checks for. Kelley-style Zig turns safety off surgically, not preemptively.

## Scope control

- This skill is intentionally self-contained. Do not load extra material for normal Zig implementation or review work.
- Leave this skill only when the task is primarily about Zig version migration, `std.Build` or package-manager churn, or target-specific ABI or linker behavior; those are toolchain problems, not philosophy problems.
