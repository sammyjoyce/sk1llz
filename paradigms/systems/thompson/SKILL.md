---
name: thompson-elegant-systems
description: Write systems code in the style of Ken Thompson — brute-force-first, lock-step NFA regex, self-synchronizing UTF-8, text streams, and deleting code. Use when designing Unix-style CLIs, writing C/Go systems code, choosing a regex engine, validating UTF-8, deciding between a dependency and 50 lines of copying, debugging by reasoning instead of stepping, auditing a toolchain for supply-chain attacks, or deciding whether to rewrite vs. patch. Trigger keywords: Unix philosophy, pipes, filters, grep, regex, NFA, DFA, ReDoS, backtracking, UTF-8, overlong encoding, self-synchronizing, Plan 9, Go, goroutines, channels, GOMAXPROCS, Thompson, brute force, small tools, trusting trust, diverse double compiling, reproducible build, rewrite from scratch, throw away code.
tags: unix, utf-8, grep, regex, plan9, go, systems, encoding, simplicity, trusting-trust
---

# Thompson: Elegant Systems

Thompson's philosophy is not "keep it simple." It is **do the obvious thing, measure, and only escalate when the measurement demands it.** The hard-won knowledge below is the part that isn't in the textbooks.

## The Single Heuristic That Governs Everything

> Build a mental model first. Debug the model, not the code.

Rob Pike's story: Thompson stood behind the keyboard and *thought* while Pike reflexively reached for print statements and stack traces. Thompson usually found the bug first — because the bug was in his model of the program, not in the program's lines. If you cannot state in one sentence what the code **should** do at the failing point, you are not ready to debug it. Reach for the debugger only after the model is exhausted.

This inverts the standard tool order:
1. Read the code and predict the state at the failure point.
2. Run the code and check the actual state against your prediction.
3. The bug is the *delta between prediction and reality* — not the symptom.

## "Before You..." Decision Frameworks

**Before adding a data structure**, try an array with linear search. Linear scan beats `HashMap` up to ~30 elements because of cache locality, no hashing cost, and no pointer chasing. A `std::vector<pair<K,V>>` with `find_if` is faster than `std::unordered_map` for small N on every real CPU. Only upgrade when profiling — not instinct — shows the linear scan on the hot path.

**Before adding a dependency**, count the lines you actually need. Thompson's rule (via Pike): *"A little copying is better than a little dependency."* If the answer is under ~100 lines and the license permits, copy them with attribution. A dependency is not free: it is supply-chain risk + version drift + build-time tax + a transitive closure you cannot audit.

**Before writing a clever algorithm**, write the O(n²) version and measure. For substring search, naive scan beats Boyer–Moore until the haystack is large *and* the pattern is >4 chars *and* the search is repeated — and even then only by a factor of 2–5×. Thompson wrote `ed` and `grep` with a backtracking matcher despite having invented the NFA construction, because real patterns rarely triggered pathological cases. Cleverness costs complexity forever; measurement costs an afternoon.

**Before a rewrite**, ask: do I have a better *specification*, or just distaste for the current code? Thompson famously threw away 1000 lines in a productive day — but only because he had understood the problem better, not because the old code was ugly. Rewrites without a better spec lose a decade of embedded bug fixes (Joel's rule). The legitimate trigger is: the new design makes the current bug *impossible*, not *unlikely*.

**Before launching a goroutine**, name its termination condition. Unbounded or un-cancellable goroutines are memory leaks with a slow fuse. Every `go f()` needs either a bounded lifetime, a `context.Context`, or a done channel. If you cannot explain in one sentence when it stops, you are writing a leak.

## NEVER (with consequence and alternative)

- **NEVER run user-supplied regex on a backtracking engine.** Seductive because stdlib regex (Python `re`, Java `Pattern`, PCRE, JS) "just works" on friendly input. One adversarial `^(a+)+$` against `"aaaa...aaaaX"` (30 chars) takes minutes; 40 chars takes hours. This is ReDoS — a documented DoS vector in every language that ships a backtracker. **Do instead:** Go's `regexp` (RE2), Rust's `regex`, or Hyperscan — all guaranteed linear-time via Thompson's NFA lock-step simulation. Accept losing backreferences; you never needed them.

- **NEVER validate UTF-8 by counting high bits without range-checking.** Seductive because a naive decoder that just masks off prefix bits "works" on valid input. The consequence is **overlong encoding smuggling**: `C0 80` decodes to U+0000, which passes your null-byte check but embeds a NUL inside a C string; `C0 AF` decodes to `/` and bypassed IIS path filtering in 2001 (CVE-2000-0884). **Do instead:** reject any encoding longer than the minimum for its codepoint, reject surrogates (U+D800–U+DFFF), reject anything above U+10FFFF. Or just call a trusted validator — never roll your own.

- **NEVER design a new binary file format.** Seductive because "it'll be faster and smaller." The consequence is endianness bugs, alignment bugs, version-skew bugs, a debugging workflow that needs custom tooling, and a format that outlives the single program that reads it. Thompson's actual rule: *"When you feel the urge to design a complex binary file format, lie down until the feeling passes."* **Do instead:** length-prefixed ASCII records, or one of {CBOR, MessagePack, protobuf} with a schema — pick the boring one your language already has.

- **NEVER use a channel where you need a mutex.** Seductive because "share memory by communicating." Channels are ~10× slower than `sync.Mutex` for pure critical sections and 50–100ns of overhead per send/receive. **Do instead:** channels transfer *ownership* of data between goroutines; mutexes protect *shared state*. A counter is shared state. A work item handoff is ownership.

- **NEVER trust a compiler just because you read its source.** Seductive because "I audited it." Thompson's 1984 Turing Lecture proved a 99-line patch to a C compiler can insert an invisible backdoor in `login` *and reinsert itself every time the compiler recompiles* — leaving the source code pristine. The only known defense is **Diverse Double Compiling (DDC)**: compile the suspect source with an independent compiler, then use the result to recompile the original source; bit-for-bit compare against the suspect binary. Go's toolchain avoids self-compilation for exactly this reason — Go 1.N is always built by Go 1.(N−1), bottoming out in the C-written Go 1.4. See `references/trusting-trust-defense.md` before doing supply-chain work.

- **NEVER parallelize by adding goroutines and hoping.** Seductive because "Go makes concurrency easy." Pike's own admission: most programmers who parallelized with goroutines got *slower* code. Concurrency is not parallelism. Goroutines help when the underlying work is I/O-bound or embarrassingly parallel. CPU-bound work with contention gets slower from scheduling overhead. **Do instead:** benchmark the serial version first; parallelize only if measurement shows CPU headroom *and* independent work.

- **NEVER use `grep -r`, `find`, and `xargs` without `-0` / `-print0`.** Seductive because it usually works. One filename with a newline or space breaks the pipeline silently. **Do instead:** `find … -print0 | xargs -0 …`, or just loop in the shell with proper quoting.

## Thompson Thresholds (the numbers)

| Decision                                  | Rule                                                                     |
| ----------------------------------------- | ------------------------------------------------------------------------ |
| Linear scan → hash map                    | > 30 elements *and* on a hot path                                        |
| Naive substring → Boyer–Moore             | Haystack > 1 MB *and* same pattern reused                                |
| Copy code → add dependency                | Copy under ~100 lines; depend when upstream is actively maintained       |
| Keep fighting a bug → rewrite the module  | > 30% of lines change for the fix; have a new spec in hand               |
| Channel → mutex                           | Critical section < 1 µs or high contention → mutex                       |
| Goroutine pool size                       | I/O-bound: ~hundreds–thousands; CPU-bound: `GOMAXPROCS`, no more         |
| Inline in SKILL.md → reference file       | Content > ~40 lines of prose or examples → move to `references/`         |

## When to Load References

The references hold the expert material you only need sometimes. **Do not load them for ordinary code questions.**

- **Writing or auditing regex for untrusted input, or debugging catastrophic backtracking** → READ `references/regex-nfa-vs-backtrack.md` completely before acting.
- **Validating, decoding, or storing UTF-8 / Unicode text at a trust boundary** → READ `references/utf8-invariants.md` completely before acting.
- **Auditing a build toolchain, signing a release, or any supply-chain work** → READ `references/trusting-trust-defense.md` completely before acting.
- **Reviewing or designing Go concurrency** → READ `references/go-concurrency-gotchas.md` completely before acting.
- **For quotes, historical context, or the philosophy itself** → `references/philosophy.md` (optional; skip for technical tasks).

**Do NOT load** all references up front. **Do NOT load** `philosophy.md` when the question is technical — it has no decision content.

## Signature Thompson Moves (Quick Reference)

- Build the mental model, then debug the model.
- Brute force first, measure, escalate only on evidence.
- Copy 50 lines rather than take a dependency.
- Text streams unless a profiler says otherwise.
- Linear-time regex (NFA) on any untrusted input, always.
- Delete code when you have a better specification, never just to tidy up.
- Trust the toolchain you bootstrapped yourself, or don't trust it.
