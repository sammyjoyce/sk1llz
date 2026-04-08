# QEMU TCG: dynamic binary translation that survived 20+ years⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​​​‌​‌‍​‌​‌​​‌‌‍‌​​‌​‌‌​‍‌‌‌​‌​‌​‍​​​​‌​‌​‍​​‌‌​​​​⁠‍⁠

Read this **only** if you are building a dynamic binary translator, an emulator, a sandbox-via-translation, or any system that consumes one instruction set and emits another at runtime. The techniques here are how QEMU achieves "good enough" performance to be practical (~10–25% of native for system emulation) while remaining portable across host architectures.

## What TCG actually is

TCG (Tiny Code Generator) is **not** an SSA optimizing compiler. It is a two-stage portable lowerer:

1. **Frontend** (`target/<arch>/translate.c`): one function per guest instruction class. Each emits TCG IR ops via `tcg_gen_*` macros. The IR is roughly RISC-y three-address with explicit temp lifetimes.
2. **Backend** (`tcg/<host-arch>/tcg-target.c.inc`): pattern-matches TCG IR ops onto host machine instructions. Mostly one-to-one or short macro expansions; the backend does *local* register allocation per translation block but no global optimization.

The key claim: this two-stage approach is portable across host arches (x86, ARM, RISC-V, PPC, ...) without per-host frontend rewrites, while still being fast enough that the dispatch overhead doesn't dominate.

**Do not** confuse TCG with LLVM. There is no SSA, no global value numbering, no inlining. The optimizations are: constant folding, dead-code elimination within a block, copy propagation. That's it. Anything more would slow down translation more than it speeds up execution.

## The Translation Block (TB) — the unit of everything

A TB is one basic block of guest code, translated to one chunk of host code, identified by its **execution context**, not just its address:

```
TB key = (physical PC, cs_base, flags, cflags)
```

`flags` includes the privilege level, mode bits (real/protected/long mode on x86), endianness, FPU state, and any other CPU bit that changes how guest instructions are decoded or executed. **Two TBs at the same guest PC under different CPU modes are different TBs.** Without this, every mode switch would force re-translation.

### Why physical PC, not virtual

Guest virtual address → guest physical address goes through the guest's MMU. If the guest changes its page tables, the same virtual PC may point at different physical code. By keying TBs on physical PC, a guest TLB flush invalidates *only the TBs that actually share a physical page with the changed mapping* — not the entire JIT cache. This is the single decision that makes system emulation viable.

### What's stored

A `TranslationBlock` struct holds:
- The host code pointer (`tc_ptr`).
- The TB key fields above.
- **Two patchable jump slots** (slot 0 and slot 1) — see chaining below.
- A linked-list pointer chaining all TBs in the same guest physical page (so page-level invalidation can walk one list).
- Host-PC-to-guest-PC mapping data, **only** for instructions whose CPU state isn't fully flushed at every step (x86 condition codes, SPARC delay slots, ARM conditional execution). For most instructions there is no mapping — exceptions are recovered by re-translating from the TB start.

## Direct block chaining (the trick that makes JIT actually fast)

The naive flow per TB is: enter TB → execute → return to dispatcher → dispatcher hashes next PC → dispatcher finds (or generates) next TB → enter next TB. The dispatcher round trip costs 50–200 host cycles per basic block, which dwarfs the work in short blocks.

**Direct block chaining** replaces this with: at the *end* of each TB, instead of returning to the dispatcher, the TB jumps to the start of the next TB *directly*. The transition is one host `jmp` — zero overhead.

### How the patch happens

The first time a TB executes, its exit slot is unpatched and points back to the dispatcher. The dispatcher finds the destination TB, then walks back and **patches the previous TB's exit jump** with the destination's host address. Next iteration: zero dispatcher visits.

A TB has **two** patchable slots because most basic blocks end in a conditional branch (taken or not-taken), and you want to chain *both* edges separately. Indirect branches and function returns still go through the dispatcher (because the destination isn't statically known), but those are a small fraction of the dynamic instruction stream.

### The two conditions for chaining

QEMU only patches the chain if **both** are true:

1. The control transfer is **direct** (constant target). Indirect calls/jumps/returns can't be chained because the target may differ each visit.
2. **Source and destination TBs share a physical page.** This guarantees that any MMU change affecting the destination also invalidates the source page (and the chain), so chains can't outlive the validity of their endpoints.

This second condition is non-obvious and important: it's how QEMU avoids needing a separate "is this chain still valid" check at every patched jump.

## Two-level TB lookup (the cache hierarchy)

When the dispatcher does run, the lookup is two-tiered:

1. **Per-CPU `CPUJumpCache`** — a small direct-mapped table indexed by `hash(pc)`. One lookup, one comparison, hot in L1 cache. Hits cost a few cycles.
2. **Global `qht` hash table** — keyed on the full TB tuple. Used only on `CPUJumpCache` miss. Much larger, slower, but exhaustive.

After a `qht` hit, the result is written back into `CPUJumpCache` so the next visit at the same PC takes the fast path. This is a textbook two-level cache design but the specific point worth stealing is: **don't try to make one cache do both jobs.** A small fast cache plus a big slow cache beats a single medium-sized cache for skewed access distributions (which guest code always is).

## Self-modifying code: the SIGSEGV trick

x86 (and many other ISAs) does not require the program to flush instruction caches when it modifies code. So when a guest writes to a page that QEMU has already translated TBs from, the writes must invalidate those TBs.

QEMU's mechanism (for user-mode emulation):

1. When a TB is generated, the host pages backing the guest code are **`mprotect`'d to read-only**.
2. If guest code writes to such a page, the host CPU raises `SIGSEGV`.
3. QEMU's `SIGSEGV` handler checks: is this fault inside a guest-code page that we've translated? If yes, walk the per-page TB linked list, invalidate every TB on the page, undo all the chain patches that point into them, restore write permission, and resume the guest write.
4. The next time guest execution reaches that PC, it re-translates.

For system emulation (where there's no real `mprotect` because guest pages are inside the QEMU process's RAM), the same effect is achieved through the softmmu's dirty-tracking — every store through softmmu marks the destination page dirty, and the dirty bit is checked against the "code page" set.

The lesson: **let the hardware (or the OS) tell you about state changes you can't cheaply track yourself.** Page protection plus a signal handler is far cheaper than instrumenting every guest store.

## Softmmu fast path

Every memory access in system emulation has to translate guest virtual → guest physical → host pointer. Doing this in C for every guest load/store would kill performance.

QEMU emits the TLB lookup **inline in the generated host code**:

```
index = (guest_addr >> PAGE_BITS) & (TLB_SIZE - 1)
if tlb[index].vaddr_read == (guest_addr & PAGE_MASK):
    host_addr = guest_addr + tlb[index].addend   # one add
    do the load/store
else:
    call C slow path (helper)
```

Five host instructions on the fast path (compare, branch, add, load, return-to-fall-through). The slow path is a real C function (`helper_ld*`/`helper_st*`) that walks the guest's actual MMU and refills the TLB entry. Because the TLB is per-CPU and physically indexed, a guest TLB flush is just a memset of the TLB array.

The `addend` trick is worth highlighting: instead of storing the host address separately, the TLB entry stores `(host_addr - guest_addr)`. The fast path then computes `host_addr = guest_addr + addend`, which is one instruction and works for any access size at any offset within the page. The MMU concept of "translation" disappears into a single addition.

## Helper functions: the escape hatch

Some guest instructions are too complex for inline TCG IR — x86's `MTMSR`, segment register loads with side effects, complex flag computations after BCD instructions, FP exception handling. Rather than open-coding 50+ TCG ops, the frontend emits a `tcg_gen_call` to a regular C function (`helper_*`).

The trade-off: helpers cost a host function call (~10 cycles) but remove huge amounts of IR generation, IR optimization, and host code emission. Use helpers for:

- Instructions with rare, complex side effects (system instructions, floating-point exceptions).
- Instructions that need to call into device emulation anyway (MMIO regions).
- Anything where the IR would exceed ~30 ops.

**Don't** use helpers for hot paths: every load/store on the fast path is inline IR; every ALU op is inline IR. The helper threshold is "would inlining this make TBs too big to fit in i-cache?"

## Exception recovery: re-translate, don't track

When a guest instruction faults (page fault, divide by zero, illegal instruction), QEMU needs to know **which guest PC** the host SIGSEGV/SIGFPE/etc. happened at. The naive approach is to maintain a host-PC → guest-PC table for every instruction in every TB — but that's huge.

QEMU does it cheaper: **store a host-PC → guest-PC map only for the small set of instructions that didn't flush their state at the start of the next instruction.** For everything else, when an exception fires, walk the host PC backward to the start of the containing TB, then **re-translate the TB from scratch in "instruction at a time" mode** until the host PC matches. The exception's guest PC is whatever the partial re-translation arrived at.

This sounds expensive, but exceptions are rare (microseconds-scale), and the savings on the common path (no per-instruction PC mapping) more than pay for it. **Pay the cost on the rare path, not the hot path** is a recurring Bellard pattern.

## When TCG-style translation is the wrong tool

- **You need single-instruction-step debugging** of every guest instruction. TCG's per-block granularity makes this awkward; some debug builds disable chaining and break by instruction, but it's slow.
- **You need actual hardware-level virtualization** (KVM, VT-x). TCG is software emulation; for the same architecture as the host, KVM is 100x faster.
- **The guest and host share the same ISA.** Use a sandbox or `seccomp-bpf` instead.
- **You're translating very short, very rare code** (e.g. a single shader). Bytecode interpretation may beat the translation cost.

## The portability claim, in concrete terms

QEMU runs as a host on: x86_64, i386, ARM, AArch64, PowerPC, RISC-V, MIPS, s390x, SPARC. It runs as a guest: even more arches. The cost of supporting a new host is **one TCG backend** (~3000 lines of C in `tcg/<arch>/`). The cost of supporting a new guest is **one TCG frontend** (~5000–20000 lines depending on ISA complexity).

That ratio — one backend per host, one frontend per guest, shared IR in the middle — is the engineering insight. If your dynamic translator only ever targets one host, you don't need the IR; just emit host code directly like TCC does. The IR exists *only* to factor host backends out of guest frontends. Don't add an IR you don't need.
