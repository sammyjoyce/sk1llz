---
name: thompson-elegant-systems
description: "Apply Ken Thompson's systems style to choices where simplicity is easy to fake but hard to earn: regex engine selection, UTF-8 byte-boundary handling, filename-safe pipelines, Go concurrency ownership, dependency pruning, and compiler trust. Use when building Unix-style CLIs or services, handling untrusted text or regexes, sharding UTF-8 logs by byte offset, deciding mutex vs channel, reducing dependency trees, auditing toolchains, or deciding rewrite vs patch. Trigger keywords: Thompson, RE2, NFA, ReDoS, UTF-8, overlong, CESU-8, self-synchronizing, find -print0, xargs -0, execdir, mutex or channel, RWMutex, trusting trust, DDC, reproducible build, small tools."
tags: unix, regex, utf-8, plan9, go, systems, trust, concurrency
---

# Thompson: Elegant Systems

Thompson simplicity is not "use fewer features." It is removing hidden states until you can predict behavior before running the program.

## Core Stance

- Prefer representations whose failure modes stay visible: text over clever binary, automata over ad hoc backtracking, ownership transfer over shared mutable soup.
- When a system gets harder to reason about, reduce moving parts before adding machinery.
- If you cannot explain the invariant that makes the bug impossible, you are not ready to rewrite.

## Decision Tree

1. Is attacker-controlled input involved?
   - User regex or internet-facing search: use a linear-time engine.
   - External text at a trust boundary: validate UTF-8 bytes before decoding or normalizing.
   - Filenames crossing process boundaries: use NUL-delimited plumbing or `find -execdir`.
2. Is the problem shared state or ownership transfer?
   - Ownership, work distribution, async result: channel.
   - Cache, field, counter, short critical section: `sync.Mutex`.
3. Are you proving build determinism or build trust?
   - Same source, same bits: reproducible build.
   - Binary corresponds to reviewed source: DDC or independent bootstrap.
4. Are you reaching for a rewrite because the code is ugly or because a stronger invariant exists?
   - Ugly only: patch.
   - New invariant makes the failure impossible: rewrite the smallest unit that can enforce it.

## Before Doing X, Ask Yourself

- Before debugging: what exact state do I predict at the failing point? If you cannot state the prediction, you are still sampling, not debugging.
- Before choosing a regex engine: do I need backtracking-only features, or do I just need captures, alternation, and Unicode classes? If user input reaches the pattern, treat look-around and backreferences as disqualifying unless the haystack is strictly bounded.
- Before chunking UTF-8 text: where are the legal restart points? UTF-8 is self-synchronizing, so you can recover from any byte offset by backing up over at most 3 continuation bytes.
- Before inventing a binary format: who debugs the corrupted file at 3 AM? If the answer is "humans with grep, less, or awk", keep it textual or use a boring schema.
- Before adding concurrency: who owns cancellation, who waits for exit, and what state remains shared? If you cannot answer all three, do not add the goroutine yet.
- Before adding a dependency: does it reduce semantic risk or just typing? In Go, any package that contributes to the build can run `init`; count the realized build graph, not the `go.mod` file.
- Before calling a toolchain "verified": am I claiming determinism, correspondence, or correctness? They are different claims.

## Non-Obvious Heuristics

- Safe regex engines are linear but not magic. Go's `regexp` rejects counted repetitions above 1000, limits parse-tree height to 1000, and caps compiled size around 128 MiB; `\pL` alone expands to 1292 runes, so generated Unicode-heavy patterns hit compile limits surprisingly early.
- RE2 chooses strategy by pattern shape: DFA for match location, NFA for submatch boundaries, one-pass execution when branch choice is locally unambiguous, and a bit-state backtracker only while the bitmap stays under 32 KiB. "Linear time" does not mean "same fast path for every pattern."
- A one-pass regex is one where it is always obvious when a repetition ends and which alternation branch wins. If that is not locally obvious, submatch bookkeeping dominates even in a safe engine.
- `\b` in Go/RE2 is ASCII-only. It is usually wrong for multilingual boundary logic even when the engine choice is otherwise right.
- UTF-8 validity is a byte-level contract. Valid 2-byte starters are `C2-DF`; `C0`, `C1`, and `F5-FF` never appear in legal UTF-8. Surrogates `U+D800-U+DFFF` and code points above `U+10FFFF` are invalid even if something "decodes" them.
- UTF-8 works well with byte-oriented tools because ASCII bytes never appear as continuation bytes and character boundaries are recoverable from arbitrary offsets. That is why Boyer-Moore-style search and shard recovery work on UTF-8 text.
- Unix filenames are bytes, not characters. Only slash and NUL are forbidden. Assume spaces, newlines, leading dashes, and invalid UTF-8 are all possible.
- `find -print0 | xargs -0` fixes whitespace parsing, not time-of-check/time-of-use races. If an attacker can rename files while you act on them, prefer `find -delete` or `find -execdir` with a trusted `PATH`.
- Reproducible builds remove accidental inputs. They do not prove the compiler binary matches reviewed source. DDC answers the correspondence question; source review answers the correctness question.
- If the build environment might be hostile, widen "compiler" to include assembler, linker, loader, privileged build helpers, and possibly the kernel. DDC is only as strong as the boundary you choose.
- `RWMutex` is usually a regression. Go's own review guidance says to benchmark it unless reads last hundreds of milliseconds and writes are rare.

## NEVER Do These

- NEVER run attacker-controlled patterns on a backtracking engine because feature completeness is seductive and friendly test data hides the bad path. The concrete consequence is trivial ReDoS: Russ Cox measured a 29-byte case where Perl took over 60 seconds while Thompson NFA took about 20 microseconds. Instead use RE2/Go/Rust regex, or split the task into a safe prefilter plus a bounded second-stage parser.
- NEVER assume a "safe" regex engine accepts arbitrarily large generated patterns because linear-time matching makes that feel safe. The concrete consequence is compile-time failure or strategy fallback when counted repeats exceed 1000 or DFA budgets churn. Instead simplify the pattern, pre-tokenize, or replace the regex with a parser when the pattern is machine-generated.
- NEVER treat UTF-8 validation as "the bytes decoded to a rune" because CESU-8, overlong encodings, and surrogate encodings are seductive half-valid forms. The concrete consequence is filter bypass: RFC 3629 explicitly calls out `C0 80` becoming NUL and path-filter evasions using illegal octets. Instead validate byte classes first, reject surrogates and overlong forms, and only then normalize or compare.
- NEVER cut UTF-8 shards on arbitrary byte boundaries because the self-synchronizing property makes repair look optional. The concrete consequence is false corruption at chunk edges or duplicated and dropped code points during parallel scans. Instead back up over continuation bytes or overlap by 3 bytes and discard the partial prefix.
- NEVER feed filesystem paths through blank- or newline-delimited plumbing because most filenames look safe and the bad cases are rare. The concrete consequence is acting on files the producer never meant to name; GNU findutils documents examples where newline-containing names make `xargs` target unrelated paths. Instead use `-print0` and `-0`, or better, `-execdir` when the action mutates files.
- NEVER use a channel to protect an integer, cache, or struct field because "share memory by communicating" is seductive when learning Go. The concrete consequence is slower code plus lifecycle bugs. Instead use channels for ownership transfer and async results; use `sync.Mutex` for state.
- NEVER default to `RWMutex` because read-mostly intuition feels sophisticated. The concrete consequence is extra overhead and worse scalability on ordinary short reads. Instead start with `sync.Mutex` and switch only with a benchmark proving the read hold times are long and writes are rare.
- NEVER call a toolchain "verified" because it is reproducible or because you read the source. The concrete consequence is missing a self-propagating compiler backdoor. Instead use DDC or an independent bootstrap chain when correspondence matters, then review the source for correctness.

## Freedom Calibration

- High freedom: decomposing a tool into smaller text-stream stages, deleting code, replacing a dependency with local code, or moving a boundary from shared state to ownership transfer.
- Low freedom: regex on untrusted input, UTF-8 at trust boundaries, filename plumbing, compiler/bootstrap verification, and concurrency shutdown paths. On these tasks, follow the invariants exactly and do not improvise around them.

## Fallback Moves

- If RE2 or Go rejects a needed construct, first ask whether the construct is only a convenience. Replace look-around with explicit capture and filter logic; replace backreferences with a parser or a second-stage equality check.
- If `-print0 | xargs -0` is still too risky, use `find -execdir ... {} +` or keep the work inside a directory file descriptor instead of stringly-typed paths.
- If DDC is impossible, state that you only have reproducibility evidence, pin the bootstrap compiler chain, and remove nondeterminism such as map iteration and lock-serialized work ordering before claiming anything stronger.
- If a rewrite is tempting but not yet justified, write the narrower invariant-preserving wrapper first. Thompson style prefers proving the new seam before deleting the old core.

## Reference Loading

- Before auditing or designing regex behavior, READ `references/regex-nfa-vs-backtrack.md`.
- Before validating, chunking, or comparing UTF-8 or Unicode text, READ `references/utf8-invariants.md`.
- Before compiler, bootstrap, attestation, or supply-chain work, READ `references/trusting-trust-defense.md`.
- Before Go concurrency changes, READ `references/go-concurrency-gotchas.md`.
- Load `references/philosophy.md` only for historical context or prose. Do NOT load it for technical decisions.
- Do NOT load all references up front. Pull only the one that matches the current failure mode.
