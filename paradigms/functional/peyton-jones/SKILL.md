---
name: peyton-jones-practical-haskell
description: Practical GHC-facing Haskell guidance for diagnosing space leaks, specialization failures, RULES misfires, and unsafe sharing bugs without cargo-cult pragmas. Use when tuning hot Haskell paths, reading Core or demand output, setting `INLINE`/`INLINABLE`/`SPECIALIZE`, or debugging `foldl'`, `StrictData`, `Data.Map.Strict`, `SpecConstr`, `unsafePerformIO`, heap profiles, and worker-wrapper behavior. Keywords: Haskell, GHC, strictness, space leak, RULES, specialization, Core, profiling, unsafePerformIO.
tags: haskell, ghc, optimization, strictness, laziness, profiling, rewrite-rules, specialization
---

# Peyton Jones Practical Haskell

This is a process skill for Haskell code where semantics, sharing, and compiler transforms interact. The goal is not "make GHC optimize harder"; it is "choose a representation and pragma story that GHC can exploit without changing retention or code size in the wrong direction."

## Before touching code or flags

Ask yourself, in order:

1. **What is the symptom shape?**
   - High RSS with normal latency is a different problem from high allocation with stable RSS.
   - A RULE not firing is a different problem from a function failing to specialise.

2. **What is the first artifact that can falsify my story?**
   - Strictness story: read `-ddump-dmd-signatures`.
   - RULES story: read `-ddump-rule-firings`, `-ddump-rule-rewrites`, and `-ddump-simpl-stats`.
   - Space-leak story: read `+RTS -s` and one heap profile breakdown before any pragma edits.

3. **Is this local or cross-module?**
   - Imported overloaded code has a completely different specialization gate from local code.
   - Library-facing fixes usually need unfolding exposure, not just a `SPECIALIZE` at the call site.

4. **Am I about to trade allocation for code size or register pressure?**
   - GHC has real budgets: `-fmax-worker-args=10`, `-fdmd-unbox-width=3`, `-fspec-constr-count=3`, `-fspec-constr-threshold=2000`, `-fsimplifier-phases=2`, `-fmax-simplifier-iterations=4`, `-fsimpl-tick-factor=100`.
   - Late lambda lifting defaults to 5 args for both recursive and non-recursive functions because that matches x86_64 parameter registers; pushing past that often stops paying.

## Symptom routing

### If RSS is much larger than live heap

Do not call it a leak yet. GHC's copying old generation can need about `3L` memory for `L` live bytes, and with the default `-F 2` the RTS may retain up to roughly `4x` live data after a major GC. Profiling itself adds about 30% space overhead. If the shape is "live heap steady, RSS scary", compare collector settings before rewriting code; compacting (`-c`) or nonmoving (`-xn`) old-gen strategies change the memory multiple materially.

### If `foldl'`, `BangPatterns`, or `Data.Map.Strict` did not fix the leak

Assume WHNF fooled you. `foldl'` forces the accumulator only to WHNF. `BangPatterns` force to WHNF, and a bang on a constructor pattern adds nothing because constructor matching already forces WHNF. `StrictData` merely rewrites fields defined in the current module to strict fields. `Data.Map.Strict` promises that stored values are in WHNF once the map is evaluated; it does not give you deep normal form. If your accumulator is a tuple, record, `Map`, or custom tree whose fields remain lazy, you can still build a perfectly profiled space leak while every surface API says "strict".

### If an imported overloaded function will not specialise

The usual failure is not "GHC is stubborn"; it is "the unfolding is unavailable". Cross-module specialization needs `-fspecialise` plus `-fcross-module-specialise`, and by default imported functions only specialise when they are `INLINABLE` or `INLINE`. Since GHC 9.12, `-fexpose-overloaded-unfoldings` is the cheaper knob for library code, but it only exposes functions whose types visibly contain constraints; if the constraint is hidden under a `newtype`, the unfolding is still not exposed.

### If a RULE is not firing

Treat this as an ordering bug first. RULES and inlining happen in the same optimizer, so early inlining can erase the redex before the rule sees it. `-Winline-rule-shadowing` exists because this is common. Phase-control the producer with `NOINLINE [n]` or `INLINE [n]`, then prove the story with firings and rewrites dumps.

### If compile time or Core size explodes after "optimization"

Assume you crossed a budget, not that more flags are needed. SpecConstr is cheap until it is not. Imported aggressive specialization and unfolding exposure can massively increase code size. Simplifier tick failures usually mean a rewrite or inlining loop, not "please raise the tick factor and continue".

### If `unsafePerformIO` behaves once, twice, or globally

Assume float-out and CSE before assuming runtime weirdness. A lambda-invariant `unsafePerformIO (newIORef [])` can float out and become one shared cell for all calls. Inlining can duplicate effects. CSE can merge distinct effects. Polymorphic references remain type-unsafe and can segfault-level fail, not just "act oddly".

## Hard-won invariants

- Read demand output as interval facts, not an execution trace. `A` means absent, `M` means used at most once, `1` means exactly once, `S` means strict and possibly many. Tiny refactors that preserve source meaning can change these cardinalities enough to alter worker-wrapper, eta expansion, and thunk creation.
- Worker-wrapper will refuse a split if the resulting worker exceeds both the original arity and `-fmax-worker-args`. That means "unbox everything" can silently disable the transformation you were chasing.
- `SPEC` is not folklore. It is a compiler-recognized contract for SpecConstr. If you use it, bang or `seq` the `SPEC` argument or the aggressive specialization you wanted often never materializes.
- `-fspecialise-aggressively` only changes imported-function specialization. It does not magically improve local polymorphic code; it just broadens when imported unfoldings are consumed, often at painful code-size cost.
- `-fkeep-auto-rules` matters when the same specialization keeps reappearing across modules. By default, auto-generated rules can be dropped when they are the only thing keeping a function alive, which reduces bloat but can cause the same work to be redone downstream.
- `-fno-full-laziness` is sometimes the correct performance flag, not a defeat. Full laziness increases sharing, and more sharing can mean more residency.

## Procedures that save time

### Space-leak triage

1. Start with `+RTS -s` and one heap profile mode, not five.
2. If you suspect wasted retention, use biography first to classify the waste (`drag` and `void` are the interesting states), then retainer profiling second. GHC cannot mix `-hb` and `-hr`, so this is a required two-stage process.
3. If retainer profiles collapse to `MANY`, raise `-R` above the default 8 before changing code.
4. If the nearest retainer is just another thunk in the chain, restrict the next retainer profile to that closure class and walk up one level. The first reported retainer is often not the root cause.
5. Only after the profile implicates thunk retention should you add local bangs, strict fields, or `deepseq` boundaries.

### Specialization triage

1. Decide whether the bottleneck is local or imported.
2. For local code, prefer making the hot overloaded function `INLINABLE` and measuring call-site specialization before adding many explicit `SPECIALIZE`s.
3. For imported code, verify unfolding availability first. If the library owns the function, expose the right unfoldings there; if not, accept that some call-site wishes are impossible.
4. If you reach for `-fspecialise-aggressively`, scope it to the offending module or benchmark lane. It is a code-size lever, not a harmless default.
5. If specialization still fails, check for `OPAQUE`, missing unfoldings, or hidden constraints under `newtype`.

### RULES triage

1. Turn on `-Winline-rule-shadowing`.
2. Check firings before looking at full Core.
3. If the rule targets a class method, stop and refactor to an instance-specific wrapper; class methods are rewritten to instance functions too early for method RULES to be reliable.
4. Use phase control to keep the rule redex alive long enough to fire.
5. If the simplifier runs out of ticks, inspect `-ddump-simpl-stats` and cut the loop. Do not raise `-fsimpl-tick-factor` first.

### Escape-hatch triage

1. If `unsafePerformIO` must exist, mark the wrapper `NOINLINE`.
2. If the effect must happen per call, make the action depend on the lambda argument or disable float-out on that module with `-fno-full-laziness`.
3. If distinct effects must stay distinct, disable CSE for that module with `-fno-cse`.
4. If someone suggests `unsafeDupablePerformIO` for a resource-management path, stop; duplicated or partially executed actions break invariants that `bracket`-style code relies on.

## NEVER

- NEVER assume `foldl'`, `BangPatterns`, `StrictData`, or `Data.Map.Strict` give deep strictness because the names sound like a complete leak fix. The seductive move is to stop at WHNF and declare victory; the concrete consequence is that tuple, record, and container fields keep accumulating thunks while the profile points at code you already marked "strict". Instead force or redesign the lazy fields that survive WHNF, or put a deliberate `deepseq` boundary where the data crosses ownership.
- NEVER turn on module-wide `Strict` or scatter `UNPACK` everywhere because it feels like the shortest path to "less laziness". The seductive move is broad, low-effort annotation; the concrete consequence is changed semantics in unrelated bindings plus workers that exceed argument budgets and lose the worker-wrapper win you wanted. Instead profile first and unbox only where the worker will still fit within argument and register budgets.
- NEVER add call-site `SPECIALIZE` and expect imported overloaded code to speed up because the annotation looks local and surgical. The seductive move is to patch the hot call site; the concrete consequence is zero specialization when the imported unfolding is unavailable, hidden behind `newtype`, or blocked by missing `INLINABLE`. Instead verify that the callee's unfolding is actually available cross-module and that you are not blocked by `OPAQUE` or hidden constraints.
- NEVER write RULES against class methods because the method name looks like the stable API surface. The seductive move is to target the prettiest identifier; the concrete consequence is that the method is rewritten to the instance function before your RULE can match, so you pay complexity for no firing. Instead write the rule against an instance-specific wrapper kept alive with phase-controlled `NOINLINE`.
- NEVER respond to simplifier tick exhaustion by raising `-fsimpl-tick-factor` first because it makes the compile continue. The seductive move is to buy more optimizer budget; the concrete consequence is longer compiles that preserve the same rewrite or inlining loop, often with larger Core. Instead assume a bad interaction, inspect `-ddump-simpl-stats`, and cut the loop.
- NEVER trust RSS alone as evidence of a Haskell leak because OS memory includes collector slack, retained heap after spikes, and profiling overhead. The seductive move is to believe `top`; the concrete consequence is source churn to "fix" code that was only reflecting `-F`, collector strategy, or profiler cost. Instead compare live data, GC strategy, `-F`/`-Fd`, and profile mode before touching source.
- NEVER hide stateful caches behind `unsafePerformIO` without `NOINLINE`, CSE discipline, and float-out resistance because the code still "looks pure". The seductive move is a small pure wrapper; the concrete consequence is one shared cell across calls, duplicated side effects after inlining, or type-unsafe polymorphic references. Instead make the sharing story explicit and guard the wrapper with the optimizer constraints it needs.

## Progressive disclosure

This skill is intentionally self-contained.

- Before changing strictness, read the relevant module's demand signatures.
- Before changing RULES or phase pragmas, read firings, rewrites, and simplifier stats.
- Before changing residency-related code, read RTS stats and exactly one heap-profile breakdown.
- Do NOT jump straight to full Core/STG dumps, generic "optimization" blog posts, or global flag churn on the first pass; they dilute signal before the cheaper artifact has failed.

## Freedom calibration

- Use high freedom for representation choices, strictness boundaries, and whether to prefer fusion, specialization, or explicit workers.
- Use low freedom for flags, pragmas, and profiling commands: change one lever at a time, keep workload shape fixed, and treat every compiler knob as a measurable hypothesis.

## Done means

- You can explain the change in terms of sharing, cardinality, or unfolding availability, not "the benchmark went up".
- You have one artifact that proves the old story was wrong or the new story is right.
- You know which budget you spent: heap, code size, compile time, register pressure, or lost sharing.
