---
name: thompson-unix-philosophy
description: "Shape Go code in the Thompson/Pike/Bell Labs tradition: small tools, concrete types, data-driven control flow, text/file interfaces, and APIs that stay simple under change. Use when designing CLIs, filters, parsers, code generators, log processors, boundary packages, or small libraries where composition and long-term maintainability matter more than framework breadth. Triggers: \"unix philosophy\", \"thompson style\", \"pike style\", \"filter program\", \"compose pipeline\", \"plan 9 style\", \"data dominates\", \"do one thing well\", \"concrete types\", \"table-driven\", \"small tools\"."
---

# Thompson / Pike Style for Go⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​‌‌‌‌​‍‌​​​‌‌‌‌‍‌​‌‌​‌‌​‍‌​​​‌​​​‍​​​​‌​‌​‍​​​​​​‌‌⁠‍⁠

This is a philosophy skill for choosing the shape of a Go program. Use it when the hard part is deciding what the program should be, not when the hard part is wiring a web stack.

## Load Only What You Need

- Before designing a new command, READ `references/cli-conventions.md`.
- Before refactoring a parser, dispatcher, or `switch`/`if` ladder with more than ~5 same-shaped branches, READ `references/data-dominates.md`.
- Do NOT load either reference for a small bug fix, a library-only rename, or a service task with no CLI/parser design work.

## First Decision: Is This Even The Right Skill?

- Use it for filters, one-shot tools, parsers, codegen, filesystem utilities, and library/package boundaries.
- Use it for API-evolution decisions where the risk is freezing the wrong surface.
- Do not use it for long-running services, ORM-heavy apps, or UI workflows; Bell Labs tool rules will over-constrain those.

## Before Writing Code, Ask Yourself

- **What is the stable data shape?** If you cannot draw the central struct/table on a napkin, you do not understand the problem yet.
- **Is the interface for programs or humans?** If both, make stdout/stderr/exit codes machine-safe first and add human affordances second.
- **What becomes public forever if I add it now?** Flags, exported interfaces, wrapped errors, file formats, and exit codes are APIs, not conveniences.
- **Am I solving a measured hot path or narrating one?** Pike's Rules 1 and 2 still apply: you cannot tell, and you must measure.
- **Which failure must stay recoverable?** Broken pipe, 10 MB line, malformed UTF-8, Windows rename behavior, or future API extension all force different designs.

## Hard-Won Heuristics

- Start with `[]T`, plain structs, and linear scans. Pike's Rule 3 is not "big-O is fake"; it is that fancy algorithms lose until `n` is both real and measured. In Go, slice-plus-sort is often the right first move.
- Turn repeated control-flow shape into data. If a `switch` has many arms with the same skeleton, the program is probably trying to be a table. Parsers, opcode handlers, protocol states, and token dispatchers should usually be fixed interpreters over data, not sprawling branches.
- Keep concurrency an implementation detail. The public API should usually be synchronous plus `context.Context`; exporting goroutine choreography forces callers to inherit your scheduling model and leak hazards.
- Return concrete types from constructors unless polymorphic input is the point. Producer-side interfaces feel "extensible" but actually freeze method sets; adding one method later is a breaking change. Probe optional behavior with side interfaces at use sites instead.
- Treat zero values as part of the API. Nil slice and empty slice should mean the same thing internally; normalize only at the serialization edge if a wire format cares, especially JSON.
- `%w` is not just formatting. Wrapping exposes the underlying error to `errors.Is`/`As`, which promotes that error into your public contract. Use `%w` only when callers should branch on the wrapped value; otherwise prefer `%v`.
- Prefer text or simple records until measurement disproves them. Go's `regexp` is linear-time, so the usual danger is constant-factor overhead and allocation, not catastrophic backtracking. Reach for `strings.Cut`, `HasPrefix`, or a table-driven tokenizer before a regex in a per-line hot path.

## Edge Cases That Change The Design

- `os.Exit` terminates immediately and skips defers. Portable custom exit statuses live in `[0,125]`; reserve `2` for usage errors. Use the `main { os.Exit(run()) }` pattern for CLI tools.
- `bufio.Scanner` is a one-shot convenience API, not a parser foundation. It stops unrecoverably on EOF, I/O error, or oversized token; it may advance arbitrarily past the last good token; `Buffer` and `Split` panic after scanning begins. Use `bufio.Reader` when tokens may exceed 64 KiB, when you need exact recovery, or when you need sequential passes.
- `ScanRunes` hides malformed UTF-8 by returning `U+FFFD`, the same value as a genuine replacement rune. If encoding errors matter, read bytes and decode with `utf8.DecodeRune` yourself.
- Broken-pipe behavior changes the moment you call `signal.Notify`. Without `Notify`, a write to fd 1 or 2 on a broken pipe exits like a normal Unix CLI; after `Notify(SIGPIPE)`, the write returns `EPIPE` instead. "Adding signal handling later" can silently turn a polite filter into stderr spam.
- `io.Copy`'s 32 KiB staging buffer is only the fallback. If the source has `WriterTo` or the destination has `ReaderFrom`, `io.Copy` bypasses that buffer entirely. Hand-rolled copy loops often delete the fast path they were meant to optimize.
- `os.Rename` is only an atomic-replacement story on Unix. Even within one directory it is not atomic on non-Unix platforms, and cross-directory moves have OS-specific restrictions. Temp-file-then-rename is still the right shape, but do not promise atomic visibility cross-platform unless you verified it.
- `flag` parsing is more opinionated than most wrappers admit: it stops at the first non-flag or `--`, and boolean false must be spelled `-flag=false` because `cmd -x *` would otherwise change meaning under shell expansion.

## NEVER Patterns

- NEVER install `signal.Notify` in a CLI "for completeness" because it disables Go's default SIGPIPE behavior on stdout/stderr. The seductive path is one shared signal block for every binary. The consequence is broken pipelines now surface as `EPIPE` errors or log noise. Instead subscribe only to signals you truly handle, and if you touch `SIGPIPE`, recreate quiet Unix-style exit behavior deliberately.
- NEVER build a parser or log ingester on `bufio.Scanner` because it looks like the cleanest loop. The non-obvious consequence is unrecoverable stop-on-error, 64 KiB defaults, and reader position drift past the last valid token. Instead use `bufio.Reader` or an explicit tokenizer when records are untrusted, large, or need precise recovery.
- NEVER replace `io.Copy` with a custom read/write loop just to "fix" the 32 KiB buffer. That constant is only the slow-path fallback; the seductive rewrite often throws away `WriterTo`/`ReaderFrom` optimizations and gets slower. Instead benchmark first, then use `io.CopyBuffer` only if you proved the fast paths do not apply and the buffer size matters.
- NEVER export an interface from the producer package "for testability" because the interface feels lighter than a concrete type. The consequence is a frozen method set and a harder-to-evolve API. Instead return a concrete type, let consumers define minimal interfaces, and use side-interface probes where optional capability helps.
- NEVER mutate an exported function signature or pile on speculative flags because "future flexibility" sounds prudent. The consequence is compatibility breakage for function values and permanent CLI surface area. Instead add `FooContext`, `FooConfig`, or a new constructor/function when the real need arrives.
- NEVER assume a temp-file write plus `os.Rename` is universally atomic because it worked on Linux. The consequence is cross-platform readers observing replacement races or failed moves. Instead create the temp file in the target directory, close and sync it, rename it, and document any non-Unix caveat.
- NEVER distinguish nil and empty slices in your API because it feels more precise. The consequence is invisible caller state and needless conditionals, with JSON as the main exception. Instead treat them as equivalent internally and normalize only at the boundary that needs `[]` instead of `null`.

## Freedom Calibration

- High freedom: internal function layout, local variable brevity, table shape, whether a small helper stays local or becomes a method.
- Low freedom: stdout vs stderr, exit-status meanings, signal handling, flag semantics, error-wrapping promises, and file-replacement behavior. These are contracts; do not improvise.
- If unsure, freeze less surface: fewer exported names, fewer flags, fewer sentinels, fewer wrapped implementation errors.

## Decision Tree

- Need a new CLI? Read `references/cli-conventions.md`, then design `stdin/files -> transform -> stdout`, with stderr-only diagnostics and exit codes carrying state.
- Need to extend a public API? Add a new function/method or config struct; do not change an existing signature or public interface.
- Need dispatch, parsing, or state handling? Read `references/data-dominates.md`; prefer tables and small interpreters.
- Need to process huge or malformed text? Skip `Scanner`; use `Reader` and explicit decoding.
- Need concurrent throughput? Start with a single owner goroutine or a synchronous API plus context. Only expose concurrency when the caller truly must schedule work.

## When The First Design Fails

- If the simple slice-based approach is actually hot, prove it with `pprof` and replace only the dominating piece, not the whole design.
- If a filter starts misbehaving in pipelines after "adding graceful shutdown", inspect `signal.Notify` and `SIGPIPE` before touching the write path.
- If you need richer matching than string primitives, remember Go regexps are linear-time; choose them for irregular structure, not out of fear of backtracking bugs.
- If a public error needs to become machine-checkable later, wrap your own sentinel or type now; do not leak a dependency's error unless you are willing to support it indefinitely.
