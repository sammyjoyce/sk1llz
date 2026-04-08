---
name: ritchie-c-mastery
description: >
  Decision-heavy systems C guidance for code that sits on undefined-behavior,
  ABI, allocator, signal, atomic, or wire-format fault lines. Use when writing
  or reviewing C where optimizer assumptions, packed layouts, hostile inputs,
  or portability can turn "works here" into production-only corruption.
  Trigger on: strict aliasing, effective type, packed structs, reallocarray,
  malloc(0), realloc(p,0), signal handlers, PIPE_BUF, lock-free atomics,
  object-size, string_copying, sanitizer triage, and ABI boundaries.
---

# Ritchie Systems C⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​‌‌​​​​‍‌‌‌​​‌‌‌‍‌​​​​‌‌‌‍‌‌​‌​​‌‌‍​​​​‌​‌​‍‌​​​‌​​‌⁠‍⁠

## Load only what applies
- Before touching byte reinterprets, packed layouts, or `-O0`/`-O2` drift, READ `references/ub-and-aliasing.md`.
- Before changing cleanup flow, syscall loops, fd lifetime, or public C APIs, READ `references/portable-idioms.md`.
- Do NOT load `references/portable-idioms.md` for aliasing-only triage.
- Do NOT load `references/ub-and-aliasing.md` for API-shape-only work.

## Core stance
- Ritchie-style C is not "close to assembly"; it is a contract with two judges: the abstract machine and the target ABI. If either can legally refute your assumption, the code is wrong even when the CPU currently "does what you meant".
- The expensive bugs are proof bugs: the compiler can prove overflow cannot happen, the allocator can legally hand back a zero-sized sentinel, or the kernel can legally interleave a pipe write you thought was atomic.
- Prefer representations that make illegal states hard to express at the edge: byte arrays on the wire, opaque structs at ABI boundaries, explicit `count,size` pairs, and one ownership sentence per resource.

## Before doing X, ask yourself...
- Before reading bytes as a wider type: **Am I relying on aliasing, alignment, and endianness staying friendly at the same time? If yes, I am already in debt.**
- Before growing an allocation: **Is the risky operation the multiplication, the resize, or the caller's bookkeeping? Guard all three separately.**
- Before marking anything `packed`: **Am I optimizing layout, or silently creating misaligned pointers the compiler will not reliably warn about?**
- Before using a signal handler for control flow: **Can this be reduced to "store a flag / write one byte / return"? Anything more is usually a portability bug.**
- Before picking warning levels: **Am I tuning CI noise, or am I trying to surface a specific UB class for one audit? The right knob depends on which goal I mean.**

## Decision trees

### 1) Bytes to typed values
- Wire/file bytes plus unknown alignment: use byte assembly or fixed-size `memcpy`. Do not cast.
- Native in-memory object plus need byte inspection: use `unsigned char *`; char-like access is the sanctioned escape hatch.
- Need bit reinterpretation across types: default to `memcpy`, even if a union "works here". Union punning stops being trustworthy once addresses escape or LTO reasons across translation units.

### 2) Size calculations and resize paths
- Any `count * size`: prefer `reallocarray` or `calloc`; if unavailable, use overflow intrinsics before the allocator call.
- Any request that can exceed `PTRDIFF_MAX`: reject it before allocation. Modern glibc rejects sizes above that threshold, and other allocators fail less uniformly.
- Need zero-length semantics: choose one project rule and document it. Linux man-pages permit a unique freeable pointer for `malloc(0)`; OpenBSD returns an access-protected zero-sized object that faults on use.
- Need `realloc(p, 0)`: do not use it as `free` shorthand. glibc documents it as non-conforming and dangerous; branch explicitly on zero and call `free`.

### 3) Signals and wakeups
- Need "tell the main loop something happened": self-pipe or `eventfd`, with handler writes no larger than `PIPE_BUF`.
- Need shared state from a handler: use `volatile sig_atomic_t` or lock-free atomics only. POSIX.1-2024 allows `<stdatomic.h>` operations in handlers only when the atomic arguments are lock-free.
- Need complex cleanup from a handler: do not. Save `errno`, record intent, restore `errno`, and let ordinary control flow do the unsafe work.

### 4) Diagnostics when the bug appears only under optimization
- Suspect aliasing but `-Wstrict-aliasing` is quiet: try `-Wstrict-aliasing=1` for a one-off audit; `=3` is the default and trades fewer false negatives for far fewer false positives.
- Suspect overflow reasoning: `-Wstrict-overflow=1` catches the easy landmines; levels `4-5` are audit settings and produce large amounts of noise.
- Suspect subobject overruns in structs: `-Wstringop-overflow=3` surfaces smaller-member overruns that default level 2 can miss, but expect benign warnings.
- Suspect uninitialized-state bugs: compile an optimized warning build. GCC only emits `-Wmaybe-uninitialized` with optimization, and optimization can also erase some evidence by exploiting UB.
- Suspect provenance, alignment, or lifetime bugs: run `-fsanitize=alignment,object-size,undefined,address`. `object-size` is valuable precisely because it uses optimizer knowledge that `-O0` never forms.
- Need the source of an MSan hit, not just the sink: use `-fsanitize-memory-track-origins=2`. If the slowdown is too high, drop to `=1`; Clang documents roughly `1.5x-2x` extra slowdown for origin tracking on top of normal MSan cost.
- Using explicit atomic orders: keep GCC's default `-Winvalid-memory-model` active so nonsense like store plus `memory_order_consume` is rejected early.

## NEVER paths experts learn the hard way
- NEVER take the address of a `packed` member and treat compiler silence as validation because GCC's `-Wno-address-of-packed-member` behavior is enabled by default. Instead copy through bytes or `memcpy`, and turn on `-fsanitize=alignment` when auditing.
- NEVER keep `size += delta; p = realloc(p, size);` because it feels linear and tidy; on failure your bookkeeping lies while the old object still exists, which is how leaks and stale bounds checks ship. Instead compute `new_size` separately and commit it only after success.
- NEVER use `strncpy` as a "safer strcpy" because it is for null-padded fixed-size records, not strings; it may omit the terminator and it zero-fills the tail. Instead use explicit-length byte copies for records, or a real string API when the destination must be a string.
- NEVER default to `strlcpy` or `strlcat` on attacker-controlled input because they feel modern and safe; Linux `string_copying(7)` notes they must read the entire source string, which turns very long hostile inputs into needless latency and DoS surface. Instead bound the read you are willing to inspect, then copy exactly that amount.
- NEVER write more than `PIPE_BUF` bytes to a self-pipe and assume atomic wakeups because POSIX only guarantees atomicity up to `PIPE_BUF` (minimum 512 bytes; 4096 on Linux). Instead keep handler writes tiny and fixed-size.
- NEVER use `realloc(p, 0)` or `reallocarray(p, 0, 0)` as portability tricks because glibc documents both as unsafe/non-conforming and C changed the semantics multiple times. Instead branch on zero before the call and make ownership explicit.
- NEVER `siglongjmp` out of a handler because it masquerades as structured error handling while dropping you into code that may immediately touch unsafe library state. Instead set state and unwind in ordinary execution.
- NEVER assume a clean `-Wstrict-aliasing=3` build proves cast-based punning is safe because GCC explicitly says the warning does not catch all cases. Instead treat cast-plus-dereference on non-character buffers as guilty until rewritten.

## Hardening defaults by consequence
- High freedom: API shape, module boundaries, representation choices. Optimize for explicit ownership and ABI evolution.
- Low freedom: casts, packed layouts, signal handlers, overflow checks, atomics, wire-format loads. Here the cost of creativity is UB, so use the documented idiom rather than taste.

## Exit checklist
- Every pointer reinterpretation survives aliasing, alignment, and endianness scrutiny independently.
- Every growth path has a checked multiplication, a checked resize, and bookkeeping that updates only after success.
- Every handler path fits in one breath: save `errno`, set flag or tiny write, restore `errno`, return.
- Every fixed-size copy states whether the destination is a string, a character sequence, or raw bytes.
- Every "compiler bug" suspicion has already survived the warning and sanitizer ladder above.
