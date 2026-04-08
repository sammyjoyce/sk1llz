# Undefined Behavior & Strict Aliasing — the deep file

Read this when: you're debugging a "works at -O0, breaks at -O2" bug, reviewing code for portability, or designing a parser/deserialiser that touches raw bytes.

---

## Strict aliasing in one paragraph

C11 6.5p7: an object may be accessed only through an lvalue of its **effective type**, a qualified version thereof, a signed/unsigned variant, an aggregate/union that contains it, **or a character type** (`char`, `signed char`, `unsigned char`). Everything else is UB and the optimiser is allowed to assume it does not happen.

The asymmetry most people get wrong:

```c
uint64_t x = 0;
unsigned char *p = (unsigned char *)&x;   // OK — char* may alias anything
*p = 0xFF;                                // OK — modifies byte 0 of x

unsigned char buf[8];
uint64_t y = *(uint64_t *)buf;            // UB — uint64_t* may NOT alias char*
```

Char-type aliasing is a **one-way escape hatch**: you can read/write any object *through* a `char*`, but you cannot read a `char` buffer *as* a wider type. (In practice `uint8_t` is a typedef for `unsigned char` on every mainstream platform, which is why `uint8_t*` inherits the exception — but `static_assert` it if you care.)

## The `memcpy` discipline

Any time you're tempted to cast pointers to reinterpret bytes, use `memcpy`:

```c
/* WRONG — strict aliasing UB, alignment UB on ARM/SPARC */
uint32_t v = *(uint32_t *)(buf + offset);

/* RIGHT — portable, same codegen at -O1+ on gcc/clang */
uint32_t v;
memcpy(&v, buf + offset, sizeof v);
```

At `-O1` or higher, gcc and clang recognise fixed-size `memcpy` and emit a single `mov` (or an unaligned load on architectures that permit one). You pay nothing. This is the correct way to:

- Reinterpret float/int bit patterns
- Load multi-byte integers from network/file buffers
- Write serialised values to byte buffers
- Implement `bit_cast` in pre-C23

**The only exception** where `memcpy` costs something: freestanding / kernel builds without libc, where you may need `__builtin_memcpy` explicitly, or an inline helper that the compiler can fold.

## Union punning

C99 TC3 (and C11 6.5.2.3 footnote 95) *do* permit reading an inactive union member — with the caveats that (a) the bytes must form a valid representation of the new type, (b) trap representations are still trap representations, and (c) LTO and escape-through-pointers can silently break it. C++ is stricter and forbids this entirely outside the "common initial sequence" rule.

**Rule:** union punning works in practice under gcc/clang in a single translation unit. The moment you take the address of a union member and pass it to another TU, you are back in aliasing-UB land. `memcpy` is safer, shorter, and equally fast.

## Integer promotion traps

Every operand of arithmetic type narrower than `int` is promoted to `int` (or `unsigned int` if `int` can't hold all values of the original type) *before* the operation. This is silent and surprising.

```c
uint16_t a = 0xFFFF, b = 0xFFFF;
uint32_t c = a * b;             // On 32-bit-int platforms: SIGNED overflow UB
                                 // a and b promote to int, int*int overflows
```

On a 64-bit-`int` platform this is fine; on everything else it's UB. The fix:

```c
uint32_t c = (uint32_t)a * (uint32_t)b;   // explicit promote to unsigned
```

Related traps:

```c
unsigned char c = getchar();     // c is 0..255
if (c == -1) { /* ... */ }        // ALWAYS false; -1 promotes to int 0xFFFFFFFF,
                                   // c promotes to int 0x000000FF. Never equal.
```

**Rule:** the moment any operand is narrower than `int`, write out the type you want explicitly with a cast. Don't trust "small" types inside expressions.

## Signed overflow is UB — and the compiler uses that

```c
int process(int size) {
    if (size > size + 1) abort();     // DEAD CODE at -O2
    char *s = malloc(size + 1);
    read(fd, s, size);
    s[size] = 0;
    ...
}
```

The optimiser reasons: "`size + 1` overflowing is UB, so I may assume `size + 1 > size` always, so the `abort` branch is dead, delete it." If `size == INT_MAX` this is an exploitable heap overflow (CERT VU#162289 against GCC; same pattern broke OpenSSL, glibc, and the Linux kernel historically).

**Correct overflow checks:**

```c
/* Unsigned — well-defined wrap */
if (a + b < a)               /* works for unsigned */

/* Signed — test the safe precondition */
if (a > INT_MAX - b) { ... }

/* Best — compiler intrinsic, portable across gcc/clang */
int r;
if (__builtin_add_overflow(a, b, &r)) { ... }
```

Flags that change the rules:

- `-fwrapv` — signed overflow wraps two's-complement. Defined behaviour, *disables* some optimisations (loop-counter hoisting, induction-variable analysis). Linux kernel uses it.
- `-ftrapv` — signed overflow traps. GCC's implementation is historically buggy; prefer UBSan (`-fsanitize=signed-integer-overflow`).
- `-fno-strict-aliasing` — compiler treats all pointers as potentially aliasing. Linux kernel uses this. Costs ~5-15% on FP-heavy numeric code.

## The "works at -O0, breaks at -O2" protocol

In order, without skipping:

1. Build with `-O2 -g -fsanitize=undefined,address -fno-omit-frame-pointer` and run your test suite. Fix every finding. 99% of the time this is where the bug dies.
2. If UBSan/ASan is silent, try `-O2 -fwrapv`. If the bug disappears, you have signed-overflow UB that UBSan missed (rare; usually UBSan catches it).
3. Try `-O2 -fno-strict-aliasing`. If the bug disappears, you have an aliasing violation — find every non-`char*` cast in the code path.
4. Try `-O2 -fno-inline`. If the bug disappears, you likely have a use-after-free or uninitialised-memory bug exposed by inlining.
5. Only after all of the above are clean is it worth suspecting the compiler. Then: minimise to <50 lines, test on both gcc and clang, file upstream.

## Alignment UB is independent of strict aliasing

C11 6.3.2.3p7: converting between two pointer types produces UB if the result is not correctly aligned for the referenced type. **The UB happens at the cast, not at the dereference.**

```c
char buf[16];
uint16_t *p = (uint16_t *)(buf + 1);   // UB right here on 2-byte-aligned arches
                                         // even if you never dereference p
```

`-fsanitize=alignment` catches this at runtime. On x86 unaligned loads are usually fine *except* when the compiler auto-vectorises and emits `MOVDQA` (which requires 16-byte alignment) — at which point you get `SIGSEGV` miles from the actual bug. Use `memcpy`.

## The `uint8_t` ≈ `unsigned char` question

On every mainstream platform, `<stdint.h>` typedefs `uint8_t` to `unsigned char`, which means it inherits the char-type aliasing exception. But the standard does *not* require this. If you are writing a library that must be bulletproof:

```c
_Static_assert(sizeof(uint8_t) == 1, "uint8_t must be 1 byte");
_Static_assert((uint8_t)-1 == UCHAR_MAX, "uint8_t must alias unsigned char");
```

In practice `uint8_t*` is safe to alias anything. `int8_t*` is *not* guaranteed — on some implementations `int8_t` is `signed char`, which does alias, but on exotic platforms it could be a distinct extended integer type. If in doubt, use `unsigned char*` for byte-level access.

## Minimal self-defence flags for any non-trivial C project

```
-std=c11 -Wall -Wextra -Wpedantic
-Wshadow -Wstrict-prototypes -Wold-style-definition
-Wmissing-prototypes -Wconversion -Wsign-conversion
-Wcast-align -Wcast-qual -Wformat=2 -Wnull-dereference
-Wundef -Wdouble-promotion
-fstack-protector-strong
-D_FORTIFY_SOURCE=2
```

For CI: add `-fsanitize=undefined,address` builds. For release: consider `-fno-strict-aliasing` if your code has *any* legacy pointer-cast patterns you haven't fully audited. Linux chose this trade-off; you probably should too unless you measure a real speedup.
