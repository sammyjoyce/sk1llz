# Hand-written SIMD: the FFmpeg / x264 / dav1d convention⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​‌‌‌‌‌‍‌​​‌‌​​‌‍‌​‌‌​‌​​‍‌​​‌‌‌‌​‍​​​​‌​‌​‍​​‌​​​‌​⁠‍⁠

Read this **only** if you are writing a media or signal-processing kernel where SIMD speed determines the project's reason for existing. The conventions here come from FFmpeg, x264, dav1d (and predate Bellard's involvement in some cases) — they represent the most field-tested style for portable hand-written SIMD in open source.

## The five rules, up front

1. Write `.asm` files, not intrinsics, not inline asm.
2. Use the `x86inc.asm` macro layer; don't write raw register names.
3. **Always** ship a clean scalar C reference path next to the asm.
4. **Always** verify the asm against the C reference using `checkasm` on random inputs (bit-exact).
5. **Disable** compiler autovectorization in your build.

If you can't commit to all five, don't start. Half-measures are slower than either extreme.

## Why not intrinsics

Intrinsics look like C functions but compile to specific SIMD instructions. They are seductive because:

- They're "almost as fast" as hand asm.
- They look like C, so reviewers can read them.
- They benefit from compiler register allocation.

The reasons FFmpeg, x264, and dav1d **all** rejected intrinsics after experiments:

- **Compiler register allocation is the problem, not the solution.** Intrinsics let the compiler choose which xmm/ymm/zmm registers to use, when to spill, when to reorder. Compilers are bad at this for SIMD code: they over-spill across loop iterations, they fail to keep values live in vector registers when scalar code touches them, they reorder loads in ways that break manual prefetch placement. Measured cost: 10–15% slower than hand asm on the same algorithm.
- **Brittleness across compiler versions.** A clang upgrade can change which registers your hot loop uses, which changes spill counts, which changes throughput. Hand asm has no version skew.
- **You can't see the instruction stream.** Reading intrinsics tells you what *should* happen; reading hand asm tells you what *will* happen. For perf-critical code the difference is everything.
- **Hungarian-notation naming.** `_mm256_permute2x128_si256` vs `vperm2i128`. Hand asm reads faster once you've learned the mnemonics.

The FFmpeg position, stated bluntly by their devs: "register allocator sucks on compilers."

## Why not inline asm

Inline asm (`__asm__ volatile (...)` inside a C function) sounds like a compromise, but it's worse than either pure asm or pure intrinsics:

- The compiler still owns register allocation around the asm block.
- Constraint syntax (`"r"`, `"=&r"`, `"+m"`, clobber lists) is fragile across GCC/clang/MSVC.
- Debug info is broken inside the asm block.
- It's not portable across compilers (MSVC disagrees with GCC syntax entirely).

The Linux kernel uses inline asm for very specific reasons (calling instructions the C ABI doesn't expose, tiny atomic primitives). Media codecs do not have those reasons. **Either commit to a `.asm` file or don't use SIMD.**

## `x86inc.asm`: the abstraction layer that makes hand asm maintainable

`x86inc.asm` is a NASM/YASM macro library, originally from x264, now used by FFmpeg, dav1d, vpx, and many others. It does three things that are individually small and collectively transformative:

### 1. Abstract register names

Instead of writing `xmm0`, `ymm0`, or `zmm0` (which are different mnemonics in the assembler), you write `m0`. The macros `INIT_XMM`, `INIT_YMM`, `INIT_ZMM` rebind `m0..mN` to the actual register width. **You write the function once and instantiate it for SSE2, AVX, AVX2, and AVX-512 from the same source.**

```nasm
INIT_XMM sse2
cglobal add_values, 2, 2, 2, src, src2
    movu  m0, [srcq]
    movu  m1, [src2q]
    paddb m0, m1
    movu  [srcq], m0
    RET
```

The same body, with `INIT_YMM avx2` at the top, becomes a 256-bit AVX2 function. Same source, different vector width.

### 2. Abstract calling convention

`cglobal funcname, num_args, num_regs, num_xmm_regs, arg1, arg2, ...` declares a global function with a specific number of integer arguments and the named arguments become register aliases (`srcq`, `src2q`). The macro layer handles SysV vs Win64 calling conventions, callee-saved register preservation, and stack alignment automatically. You never write `push rbp` / `mov rbp, rsp` / etc. — the prologue and `RET` are macro-generated.

The `q` suffix on argument names (`srcq`) is a width hint: 64-bit on 64-bit builds, 32-bit on 32-bit builds. Pointer math and address generation work correctly on both without `#ifdef`.

### 3. Macro-driven instruction selection

`x86inc.asm` knows that `vpaddb m0, m1` requires AVX (3-operand form), while `paddb m0, m1` is the SSE 2-operand form. By having you write `paddb m0, m1`, the macro emits the right form for whichever instruction set was selected at the top of the file. **You write SSE syntax; you get AVX speed where available.** Same source, multiple ISAs.

## The mandatory C reference path

For every hand-written asm function, FFmpeg ships a clean scalar C version. This is **non-negotiable** in code review. The C version:

- Acts as the baseline for `checkasm`, the test harness that runs the asm and the C path on the same random inputs and asserts bit-exact equality.
- Documents the algorithm in a form humans can read.
- Is the fallback for architectures where you haven't (yet) written asm.
- Is the starting point when porting the function to ARM NEON, RISC-V V, Apple AMX, etc.

The C version is **not** an afterthought. It is written first, tested first, and remains the source of truth. If the asm and the C disagree, the asm is wrong by definition.

A subtle point: write the C reference with the **same algorithm** as the asm, not a "more elegant" version. The asm is line-by-line traceable to the C; if you rewrite the C to be cleaner, you've lost the equivalence and `checkasm` becomes useless as a regression detector.

## `checkasm`: how the bit-exact verification actually works

`checkasm` is FFmpeg's (and dav1d's) test harness. The pattern:

1. Generate random inputs in a buffer (with controlled distribution — edge values, signed/unsigned extremes, all-zeros, all-ones, and uniform random).
2. Call the C reference path on a copy. Save outputs.
3. Call the asm path on a copy.
4. Compare byte-by-byte. **Any difference is a test failure.**
5. Run thousands of iterations across hundreds of input distributions per CI run.

Two non-obvious wins from this discipline:

- **Most asm bugs are off-by-one in pointer arithmetic or wrong-width loads.** Random testing catches these in seconds; manual tests miss them for years.
- **It catches compiler-induced regressions in the C reference**, too. If a compiler upgrade changes the C reference's output (because of strict-aliasing or signed-overflow assumptions), the test fails and you find out before users do.

`checkasm` also benchmarks: it reports cycles per call for both paths and computes the speedup ratio. **This** is the number that should appear in your commit message — not the marketing-friendly "94x" headline.

## The "94x" / "100x" honesty problem

FFmpeg got significant press in 2024–2025 for AVX-512 patches advertising 94x and 100x speedups. The actual story:

- The 94x number was measured against `-O0` C (no optimization at all).
- With `-O2`, the same C compiles to roughly half the asm's speed for that specific function (so ~2x, not 94x).
- The "100x" filter was a 6-tap convolution where the C version was 8-tap (different algorithm). Apples-to-apples, the speedup was 5–10x.
- **FFmpeg's build disables compiler autovectorization by default.** This is a deliberate choice (see below) — but it means the C path you measure against in tree is *intentionally* not what `gcc -O3 -ftree-vectorize` would produce.

The honest range for hand-written SIMD vs autovectorized C on a regular loop is **2–10x**, occasionally 15x. If someone claims more, ask what they compiled the C against.

**When reporting your own speedups**: always state the C compiler flags. `-O2 -fno-tree-vectorize` is the fair baseline if you've replaced what the autovectorizer would have done with hand asm.

## Why FFmpeg disables `-ftree-vectorize`

It is not because autovectorization "doesn't work." It works fine — at maybe 50% of hand-asm efficiency on the hot paths.

The problem is **interaction**. With autovec on:

- The compiler vectorizes the C reference path. The C path becomes 2x faster.
- The compiler also tries to vectorize the *runtime dispatch* code that picks between C and asm paths. This shuffles register state in ways that hurt the asm path's prologue.
- The compiler may inline a vectorized C path into a context that *also* calls a hand-asm path, doubling SIMD register pressure unnecessarily.
- Worst: when a function has both a hand-asm version and a fallback C version, the autovectorizer's output for the C version may be *faster* than the hand-asm on certain new microarchitectures. Now your dispatch logic is wrong but you don't notice because you weren't testing on that microarch yet.

The cleanest configuration is "the C path is intentionally scalar; the asm path is the fast one." Then the dispatch is unambiguous: if the host has SIMD, use asm; otherwise, accept the slow scalar path. No overlapping responsibilities.

## When to start writing SIMD by hand

The cost of an asm function is real: 50–500 lines of code per function, hours of cycle-counting per architecture, ongoing maintenance per ISA you support. Don't pay it unless:

- The function shows up in profiles for **users** (not just one benchmark you care about).
- The autovectorized C path is at least 3x slower than what you expect a hand asm version to achieve.
- The function is **stable** — if the algorithm is in flux, every algorithm change forces an asm rewrite.
- You will write the C reference and the `checkasm` test in the same commit.

For everything else, write good C with `restrict`, alignment hints, and short loop bodies, and let the autovectorizer do its job.

## ARM NEON / SVE / RISC-V V — does the same approach work?

Yes, but with caveats. dav1d demonstrates `x86inc.asm`-style abstractions for AArch64 NEON. The macro layer is different (`aarch64-asm.S` conventions), but the same five rules apply: hand asm in a separate file, abstract register naming, scalar C reference, `checkasm`, autovec disabled.

SVE and RVV (vector-length-agnostic ISAs) are harder — the register-width abstraction in `x86inc.asm` doesn't translate directly because SVE can be any width from 128 to 2048 bits at runtime. For SVE, intrinsics actually become more attractive because the compiler must know the runtime vector length anyway. For RVV, the ecosystem is still settling and there's no clear winner. **For x86 and AArch64, hand asm with macros is settled best practice. For SVE/RVV, watch the dav1d and FFmpeg mailing lists before committing.**
