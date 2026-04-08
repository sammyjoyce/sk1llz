---
name: ritchie-c-mastery
description: Write portable C in the style of Dennis Ritchie — abstraction with transparent cost, minimal language, trust-the-programmer discipline applied to the real standard, not to folklore. Use when writing C89/C99/C11/C17 systems code, designing C APIs and opaque handles, debugging "works at -O0, miscompiles at -O2" bugs, hunting strict-aliasing / integer-promotion / signed-overflow undefined behavior, porting code across gcc/clang/MSVC or x86/ARM/PPC, reviewing kernel-style C, or deciding between K&R idioms and modern safer replacements. Triggers on .c/.h files, "portable C", "systems programming", "undefined behavior", "strict aliasing", "opaque type", "errno", "restrict", "volatile", "type punning", "endianness", "K&R style", "goto cleanup", "kernel C", "malloc realloc", "short read short write", "memory-mapped I/O".
---

# Ritchie C Mastery

## The Ritchie Worldview (from the C Rationale, not from memes)

Three non-negotiables Ritchie and the committee stated explicitly in the Rationale:

- **"Trust the programmer"** — meaning the language will not second-guess you with runtime checks or hidden allocations. It does NOT mean the programmer can be sloppy; it means every mistake is yours to pay for.
- **"Don't prevent the programmer from doing what needs to be done."**
- **"Keep the language small and simple."** The spec is smaller than the library on purpose.

Ritchie's own public regret list (internalise these; they bite daily):

- `&` / `|` / `^` precedence *below* `==` / `!=`. Artifact of pre-`&&` C where `a==b & c==d` meant logical AND. **Always parenthesise: `(x & MASK) == VAL`.** gcc and clang warn; it is the single most-cited "wrong precedence" in C.
- Null-terminated strings — chosen because the PDP-11 `MOVB` instruction tested for zero in one cycle. Rob Pike called this "the most expensive one-byte mistake." You inherit O(n) `strlen`, no binary-safe strings, and a buffer-overflow design.
- Array decay to pointers — makes `sizeof(arr)` lie the moment you pass it to a function.

None of these is fixable. Your job is to code *around* them, not wish them away.

## Before You Write Any C, Ask Yourself

1. **What does the abstract machine say?** — not "what does x86-64 do." The C abstract machine is stricter than any real CPU. Code that "works" on x86-64 at `-O0` routinely miscompiles at `-O2` or on ARM because the optimiser is reasoning about the abstract machine, not about your hardware.
2. **What type is this expression *actually* evaluated in?** Integer promotion silently widens `unsigned char` / `unsigned short` to **signed** `int`. `(uint16_t)0xFFFF * (uint16_t)0xFFFF` is signed-int overflow (UB) on every 32-bit-`int` platform, which is nearly all of them.
3. **Is this pointer the result of an operation the standard defines?** `(T*)(char_ptr + offset)` is UB the instant `offset` isn't aligned for `T` — *before* you dereference. The conversion itself is the UB (C11 6.3.2.3p7).
4. **Who owns this memory, for how long, and who frees it?** Write the ownership rule in one sentence in the header comment. If you can't, the API is wrong, not the comment.
5. **Will the optimiser assume this never happens?** Every UB is an optimiser assumption. A signed-overflow check `if (a + 1 < a)` will be *deleted* at `-O2` because signed overflow "can't happen."

## Expert Decisions That Took The Industry Decades To Settle

### Type punning: `memcpy`, never a cast, rarely a union.

`*(uint32_t*)float_ptr` is strict-aliasing UB. GCC `-O2` will reorder loads/stores across it and hand you values from the future (see `-fstrict-aliasing`; the Linux kernel builds with `-fno-strict-aliasing` for exactly this reason). Union punning is *defined* in C99 TC3 but breaks under LTO when the union escapes through a pointer. **Use `memcpy(&dst, &src, sizeof dst)` always**; every mainstream compiler lowers it to a register move at `-O1+`. This is the one case where `memcpy` is genuinely free. Before diving into any aliasing discussion, **READ `references/ub-and-aliasing.md`**.

### `errno` is a three-rule minefield.

1. `errno` is meaningful **only** after a function that documents setting it returns its documented failure sentinel. A successful call is permitted to leave `errno` nonzero. **Never test `errno` to detect that an error happened** — test the return value first.
2. Any libc call between the failure and your check may clobber `errno` — including `printf`, `fprintf(stderr, ...)`, `strerror`, even `free` on some libcs. **Save immediately: `int e = errno;`.**
3. For functions where `-1` is a legitimate success (`getpriority`, `strtol`, `readdir`), you **must** set `errno = 0;` *before* the call and test `errno != 0` after.

### `read(2)` and `write(2)` can — and will — return short.

A `write()` of 8192 bytes may return 4096. A `read()` from a pipe, socket, or terminal returns whatever's available now, not what you asked for. `fread`/`fwrite` loop for you; raw syscalls do not. **Every raw `read`/`write` must be in a loop that handles `EINTR` and partial transfer**, or you have a silent data-loss bug that only shows up under load. Template is in `references/portable-idioms.md`.

### `volatile` is for MMIO and signal handlers. That is the whole list.

`volatile` does **not** establish happens-before, does not emit memory barriers, does not prevent inter-thread reordering. The Linux kernel document `volatile-considered-harmful.rst` bans it from kernel code except for hardware registers. Use `_Atomic` / `<stdatomic.h>` (C11) for threads. The only legitimate uses: (1) reading a memory-mapped hardware register that changes without your code writing it, (2) writing a `sig_atomic_t` from a signal handler.

### `restrict` helps in exactly one place.

`restrict` almost never pays off *inside* a function the compiler can inline — after inlining, the compiler proves non-aliasing itself. Recent GCC/Clang versions even emit *runtime* alias checks for vectorisable loops (see `-fopt-info-vec-all`: `loop versioned for vectorization because of possible aliasing`). `restrict` pays off at **non-inlinable boundaries**: exported library functions, separate translation units, and inner loops the compiler can't version. Lying to `restrict` is silent UB with no diagnostic. Rule: add `restrict` *only* after profiling and reading the generated asm.

### `goto cleanup` is idiomatic C, not a sin.

Multi-acquisition functions (open fd → malloc buf → lock → ...) have *one* correct cleanup path. C has no RAII; the alternative is nested `if`s or a flag jungle that will leak under maintenance. Labels go in **reverse acquisition order**. Never `goto` *forward* past a VLA declaration — C99 6.8.6.1 makes that UB. Full template in `references/portable-idioms.md`.

## NEVER list — every item has bitten real production code

- **NEVER** detect signed overflow with `if (a + 1 < a)`. Signed overflow is UB; the optimiser *deletes* the check. CERT VU#162289 (GCC). Use `if (a > INT_MAX - 1)`, `-fwrapv`, or `__builtin_add_overflow(a, 1, &r)`.
- **NEVER** cast `char*` / `uint8_t*` to `uint16_t*` / `uint32_t*` for "fast parsing." That is alignment-UB on ARMv5/SPARC/MIPS (SIGBUS) and strict-aliasing UB on everything. GCC will cheerfully emit `MOVDQA` against an odd address. Use `memcpy(&x, p, sizeof x)` — identical codegen at `-O1`, zero UB.
- **NEVER** write `p = realloc(p, n)`. If `realloc` fails it returns NULL, you overwrite `p`, and you've leaked the original buffer. Always: `void *tmp = realloc(p, n); if (!tmp) { /* p still valid */ return -1; } p = tmp;`.
- **NEVER** assume `char` is signed. On ARM and PowerPC it defaults to *unsigned*; on x86 it's signed. `char c = getc(f); if (c == EOF)` is silently broken on ARM (EOF is -1; `c` is 0..255). **Always `int c = getc(f);`.**
- **NEVER** call non-async-signal-safe functions from a signal handler. That bans `malloc`, `printf`, `fprintf`, `exit`, almost all of libc. The permitted list is in `man 7 signal-safety` — about 40 functions (`write`, `_exit`, `sig_atomic_t` writes, some POSIX syscalls). Violating this causes deadlocks that hit once per 10⁸ signals and survive every code review.
- **NEVER** use `scanf("%s", buf)` — no length limit, guaranteed overflow. `scanf("%1023s", buf)` is survivable but still mishandles whitespace, EOF, and matching failures. Use `fgets` + explicit parse.
- **NEVER** use `strncpy` as a "safe `strcpy`." It was designed for fixed-width v7 Unix directory entries — if the source is longer than `n`, the result is **not** null-terminated. Use `snprintf(dst, size, "%s", src)` or a bounded copy helper.
- **NEVER** mix signed and unsigned in a comparison: `for (int i = 0; i < v.len; i++)` where `v.len` is `size_t`. If `v.len > INT_MAX` the loop runs forever (or never), and the signed-to-unsigned conversion rule makes the mistake silent. Pick one type end-to-end: `for (size_t i = 0; i < v.len; i++)`.
- **NEVER** rely on `malloc(0)`. Implementation-defined: may return NULL (not an error) or a unique non-null pointer. Either branch hides bugs. Clamp small sizes to a minimum, or check `n == 0` yourself.
- **NEVER** ship `#pragma once` in headers used across build systems. Non-standard, handles symlinks / bind mounts / hardlinks inconsistently across compilers, silently fails when two files resolve to the same inode via different paths. Use `#ifndef MYPROJ_FOO_H` guards.
- **NEVER** use `signal()` in new code; use `sigaction()`. `signal()` semantics (one-shot vs persistent, EINTR behaviour) differ between BSD, System V, and glibc. `sigaction` is portable and explicit.

## Decision Trees

**"I need to read a multi-byte integer from a byte buffer":**
```
Is the buffer guaranteed aligned for the target type?
├── Yes → still use memcpy for strict-aliasing safety
└── No  → memcpy into a local, OR byte-shift assemble:
          v = (uint32_t)p[0]<<24 | (uint32_t)p[1]<<16 | (uint32_t)p[2]<<8 | p[3];
          (byte-shift is endian-independent AND alignment-safe)
```

**"Compiler miscompiles at -O2, fine at -O0":**
```
99% of the time this is UB in YOUR code, not a compiler bug.
1. Rebuild with: -fsanitize=undefined,address -fno-omit-frame-pointer
2. Hunt for: signed overflow, uninitialised reads, strict aliasing,
   misaligned loads, OOB, use-after-free, shifts ≥ type width.
3. Only after UBSan+ASan are clean may you suspect the compiler.
4. Before filing a compiler bug: build with -O2 -fno-strict-aliasing
   and -O2 -fwrapv. If either "fixes" it, you had UB.
```

**"errno is 0 / stale after a failure":**
```
Did any libc call (including printf for logging!) run between the
failure and the errno check?
├── Yes → save errno at the failure site: int e = errno;
└── No  → is the function one where -1 can be valid success
          (strtol, getpriority, readdir)?
          └── Yes → errno = 0; BEFORE the call; test errno != 0 after.
```

**"Do I need `volatile`?":**
```
Is it a memory-mapped hardware register?                  → YES, volatile.
Is it a flag written from a signal handler (sig_atomic_t)? → YES, volatile.
Anything else (threads, "force reload", "prevent opt")?   → NO. Use _Atomic
                                                            or a proper barrier.
```

## References — load only when needed

- **`references/ub-and-aliasing.md`** — Strict aliasing rules and the `char*` / `unsigned char*` exception, integer-promotion traps, signed-overflow optimiser behaviour, real-world miscompilation examples (CERT VU#162289, Linux kernel cases), `-fno-strict-aliasing` / `-fwrapv` / `-ftrapv` trade-offs, union-punning caveats. **READ before debugging any miscompilation or aliasing issue.**
- **`references/portable-idioms.md`** — Full `goto cleanup` templates with reverse-order labels, endian-independent `read_be32` / `write_le64` helpers, short-read/short-write loop with EINTR handling, opaque handle pattern, flexible array members, header design checklist. **READ when designing a new C API or porting to a new platform.**

Do NOT load both references for a single-file bug fix. Do NOT load either for trivial edits to existing code.
