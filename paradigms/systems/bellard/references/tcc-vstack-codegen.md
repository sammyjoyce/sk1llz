# TCC value-stack code generation⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​​‌​‌‌‍‌​‌​​‌​‌‍‌​​‌‌‌​​‍‌​‌‌​​‌‌‍​​​​‌​‌​‍​‌​‌​‌‌‌⁠‍⁠

Read this **only** if you are designing a non-optimizing compiler, JIT codegen, or DSL backend. The technique is what makes TCC compile ~9x faster than GCC and fit in ~100KB while still being a real C99 compiler.

## Why no IR

A textbook compiler builds AST → IR (often SSA) → optimization passes → machine code. Each step exists to enable optimizations across basic blocks. **If you are not doing those optimizations, every step except the last is dead weight.** TCC keeps the *minimum* state needed to emit code: a single value stack (`vstack`), and a global "current token" (`tok`). That's it.

The cost: code quality is roughly `-O0` GCC. The benefit: the compiler is small enough to be a library, fast enough to use as a scripting interpreter (`tcc -run script.c`), and simple enough that one person built C99 + linker + ELF loader.

When this trade is **right**: scripting languages, DSL backends, JITs that compile per-request and only need decent code, embedded toolchains, bootstrap compilers.

When this trade is **wrong**: you need register allocation across calls, you need inlining, you need any optimization that touches more than one expression at a time. Use LLVM.

## The value stack: what each entry actually is

Each `SValue` on `vstack` carries:

- **`type`** — the C type (encoded in a single int with tag bits, not a struct, because TCC was small first and never grew up).
- **`r`** — *where* the value currently lives. This is the load-bearing field. Possible locations:
  - A specific CPU register index.
  - `VT_CONST` — it's a compile-time constant; the value lives in `c.i`/`c.ull`/etc.
  - `VT_LOCAL` — it's at stack offset `c.i` (a local variable).
  - `VT_LLOCAL` — it's a *saved lvalue* on the stack (created when an lvalue had to spill).
  - `VT_CMP` — it's currently in the **CPU flags register** as the result of a `cmp` instruction. Subtype tells you which flag.
  - `VT_JMP` / `VT_JMPI` — it's the **side effect of a pending conditional jump** (used for `&&` and `||`).
- **Flags** OR'd into `r`:
  - `VT_LVAL` — what's stored is a *pointer* to the actual value. The same SValue can switch between "lvalue" and "rvalue" by toggling this bit.
  - `VT_LVAL_BYTE`/`SHORT`/`UNSIGNED` — for narrow lvalues, remembers the real width so cast can be lazy.
  - `VT_MUSTCAST` — defer a cast until use.
  - `VT_SYM` — `sym` field must be added to the constant (for unresolved symbol references; the linker will fix up).

## The two functions that matter

Everything else in the codegen is plumbing around two operations:

- **`vsetc()`/`vset()`** — push a new value. If the previous `vtop` was somewhere fragile (like in CPU flags, or part of a `&&`/`||` chain), code is emitted *first* to move it to a safe place.
- **`gv(rc)`** — "get value into register": force `vtop` into a register of class `rc`. This is the **only** place loads, casts, and lvalue dereferences are emitted. Most of TCC's code quality (such as it is) lives in `gv()`.

The architecture-specific codegen (`i386-gen.c`, `x86_64-gen.c`, `arm-gen.c`) only has to provide:
`load`, `store`, `gfunc_call`, `gfunc_prolog`, `gfunc_epilog`, `gen_opi`, `gen_opf`, `gen_cvt_*`. Everything else is shared.

## The lazy-compare trick (the most important non-obvious technique)

```c
if (a < b && c < d) { ... }
```

A textbook compiler emits: cmp → setcc → and-with-previous → cmp → setcc → and → test → branch. ~7 instructions per comparison plus branches.

TCC emits: `cmp a, b` → push `VT_CMP`(jl). When `&&` is parsed, TCC sees the LHS is `VT_CMP`, converts it directly to `VT_JMP` (a forward jump on the *opposite* condition to a "fail" label). Now generate the RHS — *that* compare also stays as `VT_CMP` until the `if` consumes it as a final conditional branch. Final code: two `cmp`/`jcc` pairs and one fixup. Roughly 4 instructions where the textbook compiler emitted 12, with **no optimization pass** — purely from refusing to materialize a boolean that nobody asked for.

The same pattern eliminates booleans in `while`, `for`, `?:`, `!`, and any combination thereof. This is why TCC's `-O0` output isn't actually as bad as you'd expect.

## Single-pass with two narrow exceptions

The TCC parser is hand-written recursive descent — no yacc, no parser generator. It runs in exactly one pass over the source, with two specific exceptions where a *local* second pass is unavoidable:

1. **Initialized arrays with no explicit size** (`int a[] = {1,2,3,4};`): a first pass counts elements so the type can be completed before the storage is reserved.
2. **Reverse-order argument evaluation on some ABIs:** when the calling convention requires args pushed right-to-left and the language requires evaluation left-to-right, a first pass walks the args to record locations.

Note what is **not** an exception: forward references to functions, struct member resolution, typedef chasing, macro expansion. All of these work in one pass because TCC processes the file top-to-bottom and any forward reference becomes a `VT_SYM` slot that the linker (also part of TCC, also single-pass) patches at the end.

## Register allocation: there isn't any

On x86, TCC uses **only** EAX, ECX, EDX. EBX, ESI, EDI are callee-saved, which means tracking them across calls would require knowing what each function clobbers — that requires either an interprocedural pass or function-local liveness, both of which TCC refuses to do. Instead: when more than three registers are needed in an expression, one is **spilled to the stack as a temporary local variable** and reloaded later. The spill slots are bump-allocated from the same arena as the real locals.

Wikipedia notes a measured cost: TCC writes register values back to the stack at the end of every statement and re-reads them on the next. This is wasteful, but the saved `cmp`/`mov` pairs from the lazy-compare trick partially compensate, and the compile-speed win (~9x faster than GCC) is the whole point.

## Bump-allocate everything

TCC's symbol table, code section, data section, and string section are all linear bump allocators. The default size limits (100 KB each, configurable at the top of `tcc.h`) are deliberately just enough for the largest realistic single compilation unit. When a section overflows, TCC `realloc`s — but in practice this rarely happens because compilation units are bounded.

`free()` is essentially never called in TCC's hot path. End-of-compilation is one pointer reset per section. This is why Valgrind reports almost nothing: there is no per-object lifetime to get wrong.

## OTCC budget lessons (the 2048-byte ancestor of TCC)

OTCC (the IOCCC 2001 winner) compiled a useful subset of C in **2048 source bytes** (counting rules excluded `;`, `{`, `}`, and whitespace). Specific tricks worth stealing in modern code:

- **Use the host dynamic linker as your linker.** OTCC resolves all external symbols via `dlsym(RTLD_DEFAULT, name)`. Any libc function — `printf`, `malloc`, `open` — is callable from OTCC-compiled code without OTCC implementing relocations or a symbol table format. TCC inherits this trick via `tcc -run`.
- **Generate code directly into a `mmap`'d page and `jmp` to it.** No object file format, no ELF emission. Useful for any one-shot JIT or compile-and-run pattern.
- **Pick one ABI and exploit it ruthlessly.** OTCC assumes i386 SysV — left-to-right argument push, EAX return — and never abstracts. The result fits in 2KB.

OTCC's lesson is not "always be obfuscated"; it is "the budget exists; what you remove tells you what was actually load-bearing."

## When this approach breaks down

You will outgrow the value-stack approach when **any** of these become true:

- You need to inline functions (requires keeping a function body around after parsing).
- You need real register allocation across basic blocks (requires building a CFG).
- You need to optimize loops (requires recognizing them as structures, not as fall-through patterns).
- You need precise debug info that maps back to source-level expressions (requires keeping AST or equivalent).

At that point you have a real compiler problem and you should build (or use) an SSA IR. Don't try to retrofit one onto a vstack codegen — start over with the IR up front. TCC has been forked many times by people who tried; none of the forks shipped.
