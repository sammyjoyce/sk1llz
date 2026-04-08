---
name: click-jvm-optimization
description: "Use for HotSpot JIT triage where C1/C2/Graal behavior, not generic Java code style, is the likely bottleneck. Applies when diagnosing deoptimization storms, code-cache saturation, tiered warmup cliffs, OSR pathologies, unstable inlining, or when deciding whether to use CompilerDirectives, CompileCommand, or source-shape changes. Keywords: click, hotspot, c1, c2, graal, deopt, osr, tiered, code cache, compiler directives, printcompilation, printinlining, logcompilation."
---

# Click JVM Optimization

This skill is for compiler economics and speculation failures. Do not use it for GC tuning, lock contention, allocator issues, socket stalls, or generic "speed up Java" work.

## Mandatory loading

- Before changing inlining, escape-analysis behavior, deopt rate, or receiver shape, READ `references/hotspot-c2-pitfalls.md`.
- Before discussing Sea-of-Nodes, IGVN, scheduling, or "should we build C2-like IR?" questions, READ `references/sea-of-nodes-tradeoffs.md`.
- Do NOT load `references/sea-of-nodes-tradeoffs.md` for production incident triage. It burns context and rarely changes the first operational move.
- Do NOT load `references/hotspot-c2-pitfalls.md` when the task is only "list active directives" or "remove a rollback flag."

## Click mindset

- Treat HotSpot as a market with four scarce resources: stable profile data, IR node budget, code-cache space, and compiler-thread time. Most regressions are one of those resources being converted into another at a loss.
- Assume the first explanation is usually wrong. "Need more inlining" is often actually "receiver profile became megamorphic." "Need more compiler threads" is often actually "same bad speculation is being recompiled faster."
- Optimize for reversibility first. Method-scoped control beats JVM-wide flags because it tells you which hypothesis was right without poisoning the whole process.

## Before touching anything, ask yourself

### Before changing flags

- Is the pain startup-only, OSR-only, or steady-state? The correct knob is different for each, and global warmup tuning often damages steady-state.
- Is there one dominant method, or is the signal spread across many tiny methods? One hot method implies source-shape or method-scoped directives; many tiny methods usually implies code-cache or tiered-policy economics.
- Am I debugging compilation behavior or execution behavior? If the answer is unclear, collect compile evidence first instead of editing source.

### Before changing code shape

- Which speculation is probably failing: receiver type, nullability, range checks, uncommon branch rarity, or allocation non-escape?
- If I "simplify" this code, will I also merge profiles that were previously separate? Manual refactors can accidentally destroy monomorphism.
- Am I making one method hotter and more analyzable, or merely moving the same entropy around?

## Path chooser

- If `PrintCompilation`/JFR shows repeated invalidation or recompilation of the same method:
  - Suspect unstable speculation or profile pollution first.
  - First move: method-scoped logging/inlining directives, not global inlining flags.
- If latency spikes line up with warmup, burst traffic, or loop entry:
  - Suspect tiered/OSR behavior first.
  - First move: determine whether the hot path is entered via normal invocation or backedge compilation.
- If compiler queues back up or sweeper activity rises:
  - Suspect code-cache economics first.
  - First move: inspect cache occupancy, heap split, and sweep pressure before changing thresholds.
- If allocation rate is high but CPU is not:
  - Suspect failed EA or lost inlining chain.
  - First move: verify the specific callsite or merge that killed scalar replacement.

## The numbers that actually matter

- `CompileThreshold` is ignored when tiered compilation is enabled. Old tuning lore that starts with this flag is often targeting a JVM mode you are not running.
- `ReservedCodeCacheSize` defaults to about `240M` with tiered compilation and about `48M` without it on modern server HotSpot builds. Segmented code cache is enabled by default only when tiering is on and the reserved cache is at least `240M`.
- Segmented heaps are fixed-size. A small or constrained cache can shut compilation off even while another heap still has space. This is why "free code cache" is not one number.
- `StartAggressiveSweepingAt` defaults to `10%` free space. If you only notice the issue when sweeping becomes aggressive, you are already late.
- `BackEdgeThreshold` is tuning folklore on modern HotSpot. Oracle's code-cache guidance explicitly says it currently does nothing; if you are in a non-tiered or constrained-cache experiment, `OnStackReplacePercentage` is the OSR lever that actually matters.
- For small codecaches, Oracle's tuning guidance treats `<5M` as a special regime where `CodeCacheMinimumFreeSpace` matters; do not push it below about `100K` unless you enjoy `VirtualMachineError` or rarer crashes.
- `MaxInlineSize=35`, `FreqInlineSize=325`, `MaxTrivialSize=6`, and `MaxInlineLevel=9` are useful smell-test numbers, not portable truths. Confirm version-local reality with `-XX:+PrintFlagsFinal`.
- `InlineSmallCode` is compiled native size, not bytecode size. This is the trap behind "small method, why did C2 stop inlining it on the second recompile?"
- `MaxNodeLimit` is version-sensitive and directive-sensitive. Raising it is not a free lunch; bigger graphs make compile latency and spill behavior worse long before they help code quality.

## Preferred control surface

1. Use CompilerDirectives first for investigation or narrow mitigation.
2. Use `CompileCommand` only when you need old-style compatibility or quick one-off repros.
3. Use global JVM flags only when the problem is clearly process-wide.

Why: JEP 165 makes directives runtime-manageable, method-dependent, and higher priority than `CompileCommand`, which itself overrides command-line flags. The first matching directive wins. If you mix all three layers casually, you can no longer trust the result.

Forced directives are not omnipotent. Even an explicit `inline` directive is still vetoed when IR growth, platform legality, or compiler safety checks say no. If a "forced" inline does not happen, suspect graph size or safety limits before assuming the directive failed to match.

## Procedures experts actually use

### When the signal is noisy

1. Start with method-scoped evidence: directive `Log`, `PrintInlining`, `PrintAssembly`, or replay options on the suspect method only.
2. Compare one warmup window and one steady-state window under the same workload shape.
3. Only after the method stays dominant in both windows do you consider a source rewrite or wider flag.

### When code cache is the suspect

1. Measure unconstrained `max_used` first; do not shrink by vibe.
2. Then check whether pressure is in profiled or non-profiled heaps. With segmented cache, "plenty of cache left" can still mean "the heap this compilation needs is full."
3. If the cache is intentionally small, remember that segmentation may strand space. Size heaps explicitly or reconsider segmentation before inventing new inlining folklore.

### When warmup is the complaint

1. Separate normal-entry compilation from OSR. They are not the same path and they do not respond equally to threshold changes.
2. If the hot work arrives in bursts, be suspicious of background compilation lag rather than assuming the code is "cold."
3. Prefer shaping the hot method or using scoped directives over globally lowering thresholds; otherwise you compile more junk earlier and pay in queue pressure and cache churn.

## Anti-patterns

- NEVER raise global inlining limits first because the seductive story is "more inline means more optimization," but the real outcome is IR growth, longer compile times, and extra code-cache pressure that often turns a local win into a tail-latency loss. Instead do method-scoped investigation and fix the specific monomorphism or EA break.
- NEVER tune `CompileThreshold` on a tiered production JVM because old blog posts make it look like the master heat knob, but with tiered compilation it is not driving the system you think it is. Instead inspect tiered behavior, OSR entry, and method-scoped directives.
- NEVER disable `BackgroundCompilation` globally to "remove queue jitter" because it is seductive during debugging, but it converts compiler time directly into request latency and can manufacture p99 regressions that were not there before. Instead narrow the experiment to one method or capture compile logs.
- NEVER shrink `ReservedCodeCacheSize` before measuring unconstrained `max_used` and heap behavior because the appealing idea is an easy footprint win, but segmented heaps can strand memory and disable compilers while another heap still has room. Instead baseline first, then constrain iteratively.
- NEVER mix CompilerDirectives, `CompileCommand`, and global flags in the same unexplained experiment because the seductive part is "more control," but the first-match and precedence rules make attribution impossible. Instead pick one control layer per experiment and record it.
- NEVER answer a deopt storm with more compiler threads first because queue growth makes it feel like a throughput problem, but recompiling bad speculation faster just burns cache and thread budget. Instead identify the unstable speculation and stop feeding it.

## Freedom calibration

- High freedom: source-shape changes, specialization boundaries, manual profile separation, deciding whether Graal is a better fit than C2 for the workload.
- Low freedom: cache sizing, global flags, compiler-thread changes, disabling background compilation, or anything that changes the entire JVM's economics.
- If the proposed move is global and not trivially reversible, ask for approval after presenting the specific metric that justifies it.

## Decision heuristics that take years to internalize

- If the same method keeps recompiling, do not ask "how do I make HotSpot optimize harder?" Ask "what assumption keeps becoming false?" The answer is usually outside the method body.
- If code-cache pressure appears only after a successful warmup, suspect tiered leftovers or lifetime mismatch between profiled and non-profiled code, not just "too much code."
- If a microbenchmark says an inlining tweak wins but production gets worse, suspect tenant-mix polymorphism or different branch rarity. Production profiles are often broader than lab profiles.
- If C2 keeps losing on allocation-heavy functional code, the right answer may be Graal's partial escape analysis rather than more flag surgery on C2.
- If a loop is hot only through OSR, source-shape cleanup is usually safer than aggressive threshold lowering; OSR wins can mask a poor normal-entry path and create misleading success.

## When to stop tuning and change strategy

- Stop and switch to source-shape work if the issue is one callsite becoming megamorphic or one merge killing EA.
- Stop and switch to cache economics if multiple unrelated methods look fine individually but queues and sweeps stay unhealthy.
- Stop and consider Graal if the dominant loss is flow-sensitive allocation elimination that C2 structurally does not do well.
- Stop and revert if a change improves throughput but worsens recompile rate, queue depth, or p99. Click-style tuning treats those as real costs, not acceptable collateral.
