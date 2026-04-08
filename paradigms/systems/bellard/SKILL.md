---
name: bellard-minimalist-wizardry
description: "Design single-author, size-constrained systems in the style of Fabrice Bellard: one-pass compilers, dynamic translators, embeddable runtimes, and hand-tuned codec kernels. Use when translation is cheaper than representation, when self-hosting or booting a proof matters more than portability, or when you need Bellard-specific tradeoffs such as no AST/SSA, value-stack codegen, fixed register mapping, translation blocks, direct block chaining, physically indexed code caches, refcount-plus-cycle GC, qjsc-style bytecode embedding, or scalar-oracle-plus-asm verification. Triggers: TinyCC, TCC, OTCC, QuickJS, QEMU, TCG, dyngen, translation block, direct block chaining, self-modifying code, no yacc, no SSA, no IR, x86inc, checkasm, NaN boxing, JS_SetMemoryLimit, js_std_eval_binary."
---

# Bellard Minimalist Wizardry⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​​‌‌‌​‍​​​‌​​​‌‍​‌‌‌​‌‌​‍​​‌​​‌​‌‍​​​​‌​​‌‍​​​‌​​‌​⁠‍⁠

Bellard style is for problems where regeneration is cheaper than architecture. If you need a long-lived multi-author platform, stable serialized IR, or formal guarantees, stop and use a heavier design on purpose.

## First filter

Before making any structural decision, ask yourself:

- **Can I prove the idea on one host first?** OTCC relied on endianness, unaligned access, in-memory codegen, and `dlsym()` before anyone worried about portability. If you cannot tolerate one ugly host-specific build, your scope is still too large.
- **Is recomputation cheaper than representation?** If the answer is yes, do not add AST, SSA, or a persistent IR. Bellard systems repeatedly win by re-deriving state later instead of storing it now.
- **Is invalidation cheaper than clever retention?** QEMU's original TB cache was 16 MiB and simply flushed when full. That only works when translation is cheap; if that sounds reckless, you are probably not in Bellard territory.

## Load only what matches

Load the smallest matching reference and stop there.

| Situation | MANDATORY read before design | Do NOT load |
|---|---|---|
| You are about to add an AST, SSA, yacc, or a second compiler pass | `references/tcc-vstack-codegen.md` | `qemu-tcg.md`, `handwritten-simd.md` |
| You are dealing with MMU, translation blocks, precise traps, interrupts, or self-modifying code | `references/qemu-tcg.md` | `tcc-vstack-codegen.md`, `handwritten-simd.md` |
| You are writing codec/DSP assembly or benchmarking a "fast path" | `references/handwritten-simd.md` | `tcc-vstack-codegen.md`, `qemu-tcg.md` |
| You are building a tiny embeddable runtime with direct bytecode generation | stay in this file unless you also hit one of the cases above | all three references |

If the task spans two categories, load them sequentially. Do not preload all references "just in case."

## The real heuristics

### 1. Representation tax is usually the first losing move

TCC gets away with one-pass codegen because a value can live as a constant, stack slot, lvalue, register, CPU flags, or deferred jump. The trick is not "use a stack"; it is "delay materialization until a consumer forces it." If your boolean becomes a register the moment you emit a compare, you already paid the wrong tax.

### 2. Fixed register mapping beats a half-built allocator

QEMU's early TCG mapped most guest registers to memory and only a few temporaries to host registers. That looks crude until you notice the payoff: portability stays linear, the generated state is easy to reason about, and the remaining copies are obvious in profiles. Do not build a heroic allocator unless you can point to the copies it removes.

### 3. Lazy condition codes are a design pattern, not an x86 hack

QEMU stores `CC_SRC`, `CC_DST`, and `CC_OP` instead of eagerly materializing flags. That buys two things at once: fewer useless flag writes on the hot path, and enough information to reconstruct precise state later. The non-obvious part is the backward pass over the whole translation block to delete dead condition-code assignments. If you need exact traps, carry reconstructable state; do not eagerly snapshot everything.

### 4. Code caches should track physical reality

For system emulation, QEMU indexes TBs by physical address and only chains direct branches when the destination stays on the same page. That is the real lesson: cache keys must be based on what cannot silently change behind your back. Virtual-PC caches feel convenient until page-table edits turn into global flush storms.

### 5. Self-modifying code is a page problem first, a compiler problem second

The first move is blunt and effective: write-protect code pages, invalidate everything on that page when a store lands, and undo block chaining. Only when mixed code/data pages invalidate too often do you graduate to a bitmap of actual code bytes inside the page so stores can prove whether invalidation is necessary. Skip straight to global dependency graphs and you will spend weeks solving the wrong problem.

### 6. One-pass systems still have honest exceptions

TCC breaks single-pass purity only when reality forces it: counting initializer elements for unknown-size arrays and reversing arguments on targets whose calling convention needs it. That is the Bellard test for a second pass: it must remove a concrete impossibility, not satisfy compiler aesthetics.

### 7. Tiny runtimes win by deleting runtime work

QuickJS compiles straight to stack bytecode, computes max stack depth at compile time, keeps small integers on fast paths, reserves half of atom space for immediate integer literals, and uses refcount plus cycle removal so the C API does not need an explicit root stack. If your embedded runtime needs stable bytecode across versions or hostile bytecode input, you have already left Bellard mode.

### 8. Feature stripping only counts if the linker can erase it

QuickJS's `-fno-*` feature toggles shrink binaries because `qjsc` relies on link-time optimization to dead-strip unused engine pieces. If you copy the surface idea without the LTO assumption, you keep the complexity and miss the size win.

## Before doing X, ask yourself

- **Before adding an abstraction layer:** what hot-path check disappears because this layer exists? If the answer is "none yet," delete the layer.
- **Before porting:** which host-specific cheat is currently buying the demo? Preserve that cheat until the proof artifact self-hosts, boots, or round-trips.
- **Before writing hand assembly:** do I already have a scalar C oracle, an honest benchmark baseline, and a fixed policy for compiler vectorization? If not, the asm path is premature.
- **Before embedding a runtime:** what are the three hostile boundaries: memory limit, stack limit, and execution timeout? QuickJS exposes `JS_SetMemoryLimit()`, `JS_SetMaxStackSize()`, and `JS_SetInterruptHandler()` because tiny runtimes still need kill switches.

## Use Bellard mode only while this remains true

| If your problem looks like... | Bellard move | Switch away when... |
|---|---|---|
| Expression-level codegen, REPL compiler, JIT backend | direct parse-to-bytecode or value-stack emission | you need cross-basic-block optimization, stable analysis passes, or non-local rewrites |
| Emulator / DBT | fixed register mapping, TB chaining, physical indexing, retranslation for precise traps | translation is no longer cheap enough to justify full-cache flushes or simple invalidation |
| Embeddable runtime | stack bytecode, refcount + cycle removal, compile-time stack sizing | you need untrusted bytecode loading, moving GC, or a stable serialized format |
| Codec / DSP hot kernel | scalar oracle + standalone asm path | performance is diffuse across the program or the algorithm is still changing weekly |

## NEVER do these things

- **NEVER add SSA, a persistent AST, or a generic optimizer to a compiler that only optimizes expressions**, because the seductive "proper compiler architecture" move adds representation cost without creating any optimization that can pay for it. Instead keep values in the cheapest recoverable state and emit only when use forces you.
- **NEVER use yacc/bison/ANTLR for a language whose semantics you still want to bend around code generation**, because parser generators feel like free leverage but they sever the tight parse/type/codegen interleave that keeps one-pass compilers tiny. Instead write recursive descent and spend complexity on emitted states, not grammar tooling.
- **NEVER key translated blocks by virtual PC alone**, because it matches frontend intuition but every remap or aliasing event turns into unnecessary flush pressure and broken chains. Instead key by physical address plus static CPU state and chain only when the destination page is safe.
- **NEVER materialize flags or booleans earlier than their first consumer**, because it looks explicit and debuggable but kills lazy-condition-code elimination and makes exact exception recovery more expensive. Instead carry reconstructable tuples or flag-backed values until something truly needs a register.
- **NEVER trust TCC-style bounds checks across unchecked ABI boundaries**, because unchecked pointers are explicitly assumed valid. That is seductive because compatibility stays high, but it means the safety boundary disappears the moment foreign code hands you a pointer. Instead treat incoming pointers as tainted and validate or copy at the boundary.
- **NEVER let a dyngen-style backend float across compiler versions**, because the compiler is part of the generator: relocation shapes, function prologues, and copied stub layouts are all load-bearing. Instead pin the toolchain and diff generated stubs before blaming runtime logic.
- **NEVER mix inline asm, compiler vectorization, and handwritten SIMD in the same hot kernel**, because the halfway approach is seductive but produces impossible-constraint build failures, silent miscompiles, or tiny regressions that are miserable to attribute. Instead choose pure C plus compiler, or pure C oracle plus standalone asm.
- **NEVER load QuickJS bytecode or Binary JSON from untrusted or cross-version sources**, because execution happens without a security check and the format can change without notice. Instead ship source or a trusted same-version embedded blob.
- **NEVER execute JS from a QuickJS finalizer**, because it feels convenient for cleanup but re-enters the runtime from GC context. Instead free only C resources there and expose explicit JS-level close/dispose paths.
- **NEVER port before the self-test exists**, because portability feels like progress while it quietly freezes the wrong abstractions. Instead exploit one host, one ABI, and one compiler until the proof artifact works.

## Fallbacks when the primary move breaks

- **Your JIT/emulator thrashes on mixed code/data pages:** first unchain aggressively, then add the per-page code bitmap, and only then consider a more complex invalidation graph.
- **Your one-pass compiler keeps asking for "just one more" backpatch pass:** audit whether the problem is genuinely local. If it is not, stop pretending and add the smallest targeted IR at the exact seam that forced the second pass.
- **Your QuickJS embedding leaks or crashes:** audit `JS_DupValue()` / `JS_FreeValue()` balance first, then verify every custom class exposes `gc_mark` for outgoing references, then set memory/stack/interrupt limits before debugging anything else.
- **Your asm path benchmarks great but breaks in production:** treat the scalar C path as the oracle and re-run with adversarial widths, tails, alignment, saturation edges, and randomized buffers. If the asm is not bit-exact, delete or gate it until it is.
