---
name: bellard-minimalist-wizardry
description: Build small, dense systems software in the style of Fabrice Bellard (TinyCC, QEMU, FFmpeg, JSLinux, QuickJS, BPG, LTE). Use when designing a non-optimizing compiler, a dynamic binary translator, a media codec kernel, an embeddable language runtime, a single-file emulator, or any system that must self-host with one author. Triggers include writing a recursive-descent parser without yacc, building a value-stack code generator instead of an SSA IR, translation-block chaining, softmmu TLB fast paths, hand-written SIMD assembly versus intrinsics, NaN boxing, refcount-plus-cycle GC, bump arenas, OTCC-style code-size budgets, replacing autovectorization with `.asm` files, getting a project to compile itself.
---

# Bellard's Way

You are in this skill because size, speed, and "fits in one author's head" all matter at the same time. Bellard's body of work shares a recognizable engineering grammar — distinct from both "move fast" and "industrial software." This skill teaches the grammar, not the trivia.

## The first question (ask before any design decision)

> **"Can I get a working end-to-end demo on ONE platform with ALL the dirty tricks, today?"**

If yes — do that first. Every Bellard project passes a self-test before any portability or cleanup work: TCC self-compiles, JSLinux boots Linux, FFmpeg round-trips a real file, QuickJS runs the test262 suite. Until you hit that bar, do not refactor, do not port, do not generalize.

If no — **your scope is wrong**. Cut features until the answer is yes. This is not a productivity tip; it is the load-bearing decision in every Bellard project.

## Decision tree: which sub-domain are you in?

Load **only** the reference that matches. Each is dense and loading the wrong one wastes context.

| Building... | MANDATORY read before designing | Do NOT load |
|---|---|---|
| Non-optimizing compiler, DSL backend, JIT codegen | `references/tcc-vstack-codegen.md` | the others |
| Dynamic binary translator, emulator, sandbox | `references/qemu-tcg.md` | the others |
| Media/signal kernel where SIMD matters | `references/handwritten-simd.md` | the others |
| Small embeddable language runtime | (inline notes below — no reference needed) | all |

If your task spans two categories (e.g. building a JIT *and* writing SIMD intrinsics for it), read both — but read them sequentially, not preemptively.

### When this skill does NOT apply

- You are building a long-lived multi-author production system where the "fits in one head" criterion is irrelevant. Use industrial patterns.
- You need to ship something that other people will fork heavily — minimalism is a personal trait that doesn't survive contact with a community. (See: TCC's many forks, none of which shipped.)
- You are doing genuine cross-block optimization (SSA, inlining, loop transforms). LLVM exists; use it.
- You need formal correctness guarantees (cryptographic primitives, safety-critical control). Bellard-style code is brilliant but rarely formally verified.

## Six non-obvious heuristics

### 1. An IR is overhead until your optimizer can pay for it
If you are not doing cross-block dataflow optimization (SSA, GVN, global register allocation), do **not** build the textbook `parse → AST → IR → backend` pipeline. TCC keeps a single value stack where each entry knows whether it currently lives in a register, in CPU flags as a pending compare, in memory at a stack offset, or as a constant — and emits code lazily on use. The non-obvious payoff: `if (a<b && c<d)` never materializes a boolean; the compare result chains straight into the branch. Idiomatic short-circuit code becomes free.

### 2. Translate, don't interpret — sooner than you think
A naive instruction-by-instruction translator beats a heavily optimized interpreter the moment a basic block exceeds ~5 guest instructions, because dispatch overhead vanishes. The threshold for hand-rolling a JIT is much lower than people assume. QEMU's specific magic is **direct block chaining**: a translation block patches its own exit jump to point at the next block, so a hot loop becomes straight-line host code with zero dispatcher round trips after the second iteration.

### 3. Index your caches by physical reality, not virtual convenience
QEMU keys translation blocks by `(physical_PC, cs_base, flags, cflags)` — never by guest virtual PC. Sounds like extra work; it is the entire reason guest TLB changes don't blow away the JIT. Apply this anywhere you cache: key by what *cannot* change behind your back, not by what is cheapest to compute at insert time.

### 4. Write the dumb C reference path first — and keep it forever
FFmpeg ships a clean scalar C reference for every routine alongside the AVX-512. Not (only) as a portability fallback. The C version is the **oracle** that `checkasm` runs random inputs against to verify the SIMD is bit-exact. Without it, hand-asm degenerates into unverifiable folklore the moment its author moves on. Same applies to TCC's "naive codegen" comments and QuickJS's interpreter loop relative to its inline-cache fast paths.

### 5. If autovectorization helps you, your hot path isn't actually hot
Compiler autovec wins ~2x on a regular loop. Hand SIMD wins ~8x on the same loop. The two **conflict**: leaving `-ftree-vectorize` on while shipping hand-written `.asm` paths slows the hand path through worse register allocation pressure. FFmpeg disables compiler autovectorization in its build for this reason. Commit fully or not at all — the worst configuration is "we have hand asm AND we let the compiler vectorize."

A calibration note: the famous "94x" and "100x" FFmpeg headlines were measured against `-O0` C. With `-O2 -fno-tree-vectorize`, real speedups are 2–10x. Still enormous, but plan capacity around the honest number.

### 6. Bump arenas eliminate categories of bugs, not just allocations
In TCC the symbol/code/data sections are bump-allocated; in QuickJS the per-function compile arena is. The wins compound: `free()` becomes a single pointer reset, use-after-free within an arena is statistically impossible (nothing moves), cache locality is automatic, and Valgrind is mostly unnecessary because there is no per-object lifetime to mismanage. The pattern applies any time a group of allocations shares a phase boundary — a parse, a compile, a frame, a request, a translation block.

## Embeddable language runtime notes (QuickJS pattern)

If you are building one of these and don't need a separate reference file, internalize these specifics:

- **Compile straight to bytecode with no parse tree.** A stack-based bytecode plus a hand-written single-pass compiler is smaller and faster than any AST-walking compiler. Compute max stack depth at compile time so the interpreter loop needs no runtime overflow check.
- **Reference counting + a separate cycle collector** beats tracing GC for embedded use: deterministic free, no GC root annotations leaking into your C API, embedders can reason about lifetimes. The cycle collector only walks reference counts and object contents — no roots.
- **NaN-box only on 32-bit; use a two-register struct on 64-bit.** On 64-bit, memory is cheaper than the bit-twiddling tax. Two-CPU-register `JSValue` returns through registers via the SysV/Win64 ABI for free.
- **Half your atom table is reserved for immediate small-int literals.** An "atom" is then either an interned string pointer or an integer in `[0, 2^31)` distinguished by tag bits — one path handles both.
- **Backtracking regexp uses an explicit stack**, never host stack recursion. A 15 KiB regexp engine is achievable; ICU is not your friend.

## NEVER list (each item: the seductive wrong path → why it's wrong → what to do instead)

- **NEVER add an IR to a compiler that doesn't optimize across basic blocks.** Seductive because every textbook draws the diagram. Consequence: ~5x more code, slower compile times, identical output quality. **Instead** use a value stack with lazy code emission (read `references/tcc-vstack-codegen.md`).

- **NEVER use yacc/bison/ANTLR for a language whose semantics you control.** Seductive because they "generate the parser for free." Consequence: you lose error recovery, debug control, and the ability to interleave parsing with codegen — which is the *only* reason single-pass compilers stay small. **Instead** write hand-recursive-descent; it will be ~30% the size of the generated output and you can step through it.

- **NEVER use intrinsics if you have decided you need hand-tuned SIMD.** Seductive because they look like C and are "almost as fast." Consequence: ~10–15% perf loss from worse register allocation, brittleness across compiler upgrades, the actual instruction stream is hidden from you. **Instead** write real `.asm` files using `x86inc.asm` macros (read `references/handwritten-simd.md`) — or commit fully to the auto-vectorizer. The middle is the worst place.

- **NEVER cache JIT-compiled or translated code by virtual address alone.** Seductive because virtual addresses are what your frontend already has. Consequence: any guest TLB or page-table change forces a full cache flush, killing throughput. **Instead** key by physical address (or content hash) so MMU events become precise per-page invalidations.

- **NEVER use tracing GC in an embeddable interpreter.** Seductive because "modern languages use GC." Consequence: nondeterministic latency, GC root annotations leak into every C function that touches a value, embedders can't reason about lifetimes. **Instead** use refcount + periodic cycle collection.

- **NEVER port before self-hosting.** Seductive because porting is mechanical and feels like progress. Consequence: you lock in design decisions before you know which ones cost you. **Instead** depend freely on endianness, unaligned access, and a specific ABI until your project passes its own self-test; then generalize.

- **NEVER make the build system a feature matrix in v0.1.** Seductive because "users will ask for X." Consequence: the implementation gets shaped by the build system instead of by the algorithm. **Instead** ship one configuration. TCC's first release was Linux/i386 only. FFmpeg 0.1 supported one container and two codecs.

- **NEVER intern global state in singletons or `atexit` handlers in code you want to be embeddable.** Seductive because "it's the simple version." Consequence: nobody can run two instances in one process, sandboxing is impossible. **Instead** pass a runtime pointer everywhere — QuickJS has `JSRuntime` and `JSContext`, QEMU has `CPUState`, FFmpeg has `AVCodecContext`.

- **NEVER trust a hand-asm-vs-C benchmark that wasn't compiled with `-O2 -fno-tree-vectorize`.** Seductive because the headline 94x number looks great in a talk. Consequence: you size your roadmap around fiction. **Instead** insist on apples-to-apples C with vectorization explicitly disabled (since that's what the asm replaces) before accepting any speedup claim.

- **NEVER reach for `malloc()` in a hot path when the lifetime is bounded by a phase.** Seductive because it's the default. Consequence: per-allocation overhead, fragmentation, lifetime bugs. **Instead** bump-allocate from a per-phase arena; release with one pointer reset.

## Procedure: bringing up a new Bellard-style project (this order is not negotiable)

1. **Define the smallest end-to-end demo that proves the concept.** TCC's was `compile and run printf("hello")`. QEMU's was booting a tiny boot sector. Don't write architecture docs first; write that demo first.
2. **Lock in one host and one target platform.** Use endianness, ABI, and unaligned-access assumptions freely.
3. **Single `.c` file until it doesn't fit in your head.** "Fits in head" is the splitting criterion — not file size, not module-count fashion.
4. **Reach the self-test.** Compiler self-compiles. Emulator boots. Codec round-trips. *No optional features may be added before this step.*
5. **Add `checkasm`-style golden tests** comparing every fast path to a clean reference path on random inputs. Forever.
6. **Only now:** portability, configurability, additional architectures, optional features, the build matrix.

If you feel pressure to skip step 4, the answer is always "scope is too big — cut more." Never reorder.
