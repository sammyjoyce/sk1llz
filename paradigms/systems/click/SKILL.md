---
name: click-jvm-optimization
description: Expert guidance for designing JIT compilers, intermediate representations, and speculative optimizations for managed runtimes in the tradition of Cliff Click (HotSpot C2 architect, inventor of sea-of-nodes, Global Code Motion/GVN). Use when building or debugging JITs, designing compiler middle ends, deciding between sea-of-nodes and CFG IR, tuning HotSpot C2 (escape analysis failures, inlining thresholds, megamorphic dispatch, deoptimization loops, range check elimination), implementing IGVN / GCM / partial escape analysis / speculative guards, or diagnosing issues like "scalar replacement failing," "bimorphic inline cap," "deopt loop," "C2 bailout," "iterator not eliminated," "counted loop lost," "Graal vs C2," "Turbofan Turboshaft," "V8 leaving sea of nodes," "IGVN non-termination," or "GCM placement."
tags: jit, hotspot, c2, sea-of-nodes, escape-analysis, inlining, deoptimization, igvn, gcm, graal, turbofan, compiler-ir, speculation, jvm-performance
---

# Cliff Click — JIT & IR Design

Cliff Click architected HotSpot C2 and invented the sea-of-nodes IR. His
compiler runs trillions of times per day. This skill captures what took him
decades to learn — and what the V8 team took ten years to *unlearn*.

## Load This Skill's References When

| Task | READ |
|------|------|
| Tuning C2 inlining, debugging EA failures, megamorphic dispatch, deopt loops, range check elimination | **MANDATORY: `references/hotspot-c2-pitfalls.md`** |
| Designing a new IR, evaluating SoN vs CFG, debugging IGVN/GCM, reading V8 Turboshaft rationale | **MANDATORY: `references/sea-of-nodes-tradeoffs.md`** |
| Generic "how does a JIT work" question | Do NOT load either reference — answer from base knowledge |

## The One Sentence

> *"Optimization is proving things don't happen. The fastest code is the code
> you proved you didn't need to run."* — Click

Every paragraph that follows exists because someone shipped a JIT where this
maxim was not the default mental model, and their JIT was slow.

---

## Mindset Checks Before You Write One Line of Compiler Code

Before adding a new optimization pass, ask:

1. **What am I proving, and what breaks if I'm wrong?** An optimization is a
   proof that a transformation preserves semantics. Write down the theorem
   before the code. If you cannot state the theorem, you cannot debug the
   miscompile.
2. **Can I subsume this with an existing pass running to fixed point?** IGVN
   (peephole + GVN + ccp + DCE combined) finds 90% of what people write new
   passes for. Extending IGVN with one more rewrite rule is almost always
   better than adding a new pass over the graph.
3. **What information does the downstream pass need that I'm about to
   destroy?** Lowering discards information. Every early lowering is a
   permanent cap on later optimization quality. Delay lowering until you've
   extracted everything the high-level form gives you.
4. **Who is allowed to deopt, and how do they recover?** Speculation is safe
   only because the interpreter is always a valid target. Before adding a
   guard, trace the deopt path end-to-end. If you cannot rebuild interpreter
   state, you cannot speculate.
5. **Am I optimizing cold code?** Interpreter → C1 → C2 exists because
   optimizing cold code is strictly bad. If the method runs once at startup,
   every C2 cycle you spend is a cycle the user waits.

Before tuning C2 for a Java application, ask:

1. **Is this method actually hot enough to reach C2 (Tier 4)?** Default Tier 4
   threshold is ~15000 invocations + backedge count. Below that, you're
   looking at C1 code and all your C2 advice is wrong.
2. **Did inlining even happen?** Turn on `-XX:+PrintInlining`. 80% of "EA
   failed" diagnoses trace back to an earlier inlining decision that never
   fired. Fix inlining first, EA second.
3. **What's the receiver-type profile at the suspect call site?** Bimorphic
   (≤2) → probably fine. 3+ → megamorphic cliff, every downstream opt dies.
4. **Are my loops counted?** `int i; i < limit; i++` is a counted loop. Anything
   else forfeits unrolling, range-check elimination, and SIMD.

---

## The Click Decision Table: Sea of Nodes vs CFG

| Situation | Choice | Why |
|-----------|--------|-----|
| Statically typed, mostly-pure functional code (Haskell, OCaml) | **SoN** | Most nodes genuinely float; combined analyses pay off |
| Dynamic language with per-op type checks (JS, Python) | **CFG+SSA** | Every op needs control+effect edges → SoN degenerates to CFG with extra indirection (V8's lesson) |
| AOT input that was already scheduled (WASM, pre-optimized IR) | **CFG+SSA** | SoN discards source schedule at graph build; rebuild is strictly worse |
| Need combined constant-prop + DCE + GVN (Click's thesis insight) | **SoN** *or* CFG with worklist IGVN | Either works; SoN is stricter but harder to debug |
| JIT with tight compile-time budget (<100ms per method) | **CFG+SSA** | Turboshaft cut compile time by 2× vs Turbofan SoN |
| You do not already have working Global Code Motion | **CFG+SSA** | GCM is months of subtle bugs. Do not start there. |

The real question is never "SoN or CFG?" It's "which combined analyses do I
need, and which IR makes them cheapest?"

---

## Hard-Won C2 Numbers You Must Know Cold

These are the thresholds that decide whether your Java code becomes a tight
loop of machine code or a series of virtual calls. Memorize them:

- **`MaxInlineSize=35`** bytecode bytes — always inline if under
- **`FreqInlineSize=325`** bytecode bytes — inline if hot and under
- **`InlineSmallCode=2000`** *native* bytes — don't re-inline callee bigger than this
- **`MaxInlineLevel=9`** — max nesting; reactive-stream chains hit this
- **`DesiredMethodLimit=8000`** — post-inline bytecode cap; compile-time constant
- **Bimorphic inline cap = 2** — third receiver type = megamorphic cliff
- **Interprocedural EA bails at 150 bytes** of callee bytecode
- **Tier 4 threshold ≈ 15000** invocations + backedges (default)
- **EA success rate ≈ 13%** of candidate methods on real workloads (DaCapo/Renaissance)
- **`PerMethodRecompilationCutoff=400`** — then permanently interpreted

For the full diagnosis flow and the rest of the numbers, **READ
`references/hotspot-c2-pitfalls.md`** before touching `-XX:` flags.

---

## NEVER (and what to do instead)

- **NEVER add a new optimization pass when an IGVN rewrite rule will do.**
  *Why it's seductive:* a new pass feels cleaner and is easier to reason
  about in isolation. *Consequence:* sequential passes miss optimizations
  combined analyses would find (Click's 1995 thesis result). *Instead:*
  extend the peephole/IGVN rewrite set and let it run to fixed point.

- **NEVER speculate without a profile that proves the common case.** *Why
  seductive:* "obviously it's usually X." *Consequence:* deopt loops. The
  same wrong bet gets made on every recompile until `PerMethodRecompilationCutoff`
  bans the method from C2 forever. *Instead:* read the actual MethodData,
  and if profile entropy is high, emit a bimorphic type-switch instead of a
  monomorphic guard.

- **NEVER lower early to "simplify" later passes.** *Why seductive:* fewer
  node types downstream. *Consequence:* you discarded the high-level shape
  EA, inlining heuristics, and loop recognition depend on. Once lowered to
  memory ops, you cannot prove non-escape. *Instead:* lower progressively,
  keep high-level ops until every high-level opt has had its chance.

- **NEVER put a side-effect node on both the effect chain AND the control
  chain "to be safe."** *Why seductive:* it looks correct and you stop
  worrying about anti-dependence. *Consequence:* the node can no longer
  float, which was the entire point of SoN — you've rebuilt a CFG inside
  your SoN (V8's Turbofan lesson). *Instead:* effect edges for memory
  ordering, control edges ONLY when a deopt/exception depends on the op.

- **NEVER trust that your `for-each` loop allocates no iterator.** *Why
  seductive:* escape analysis is supposed to handle it. *Consequence:* EA
  needs inlining (3+ frames deep) + monomorphic receiver + no control-flow
  merge + callee <150 bytes. Fail any link and you allocate every iteration.
  *Instead:* measure with `-XX:+PrintEliminateAllocations`; if it fails in a
  hot path, manually hoist the allocation or use a primitive-indexed loop.

- **NEVER write `for (long i = 0; i < arr.length; i++)`.** *Why seductive:*
  "long is safer for big arrays." *Consequence:* C2's counted-loop detector
  requires `int`. You lose range check elimination, unrolling, and SIMD —
  a 4–16× slowdown on the loop body. *Instead:* use `int`; if the array
  truly exceeds 2^31, nest a long outer loop with int inner chunks.

- **NEVER refactor by extracting "helper" methods in the hot path without
  measuring.** *Why seductive:* cleaner code, single responsibility.
  *Consequence:* the extracted method may cross the 35/325/150-byte
  inlining and interprocedural-EA cliffs invisibly. A helper at 151 bytes
  silently kills EA for everything that calls it. *Instead:* measure with
  `-XX:+PrintInlining` before and after; if it breaks, `@ForceInline`
  (JDK-internal) or manually inline back.

- **NEVER assume megamorphic is "only a little slower."** *Why seductive:*
  "vtable calls are cheap, modern CPUs have branch predictors." *Consequence:*
  the call cost itself is ~3× — but the real damage is downstream. No
  inline means no EA, no type flow, no constant propagation into the
  callee. A single megamorphic site can slow the surrounding method 5–10×.
  *Instead:* split the call site (manual unroll, type dispatch), or refactor
  to monomorphic. See `references/hotspot-c2-pitfalls.md` for the
  apangin/Shipilev trick.

- **NEVER reach for Sea of Nodes in a new compiler without a working GCM
  prototype.** *Why seductive:* the Click thesis makes SoN look elegant.
  *Consequence:* months lost to Global Code Motion bugs, IGVN non-termination,
  memory phi explosions, and impossible-to-read graphs. V8 took ten years
  to escape it. *Instead:* start with SSA-on-CFG + a worklist IGVN pass.
  Get the combined-analysis benefits without the scheduling nightmare.

---

## Signature Click Moves (the actually-useful ones)

- **Combine analyses.** Run peephole + GVN + conditional constant propagation +
  unreachable code elimination as one fixed point, not four passes.
- **Speculate with deopt as the insurance.** Bet on the profile, guard the
  bet, deopt when wrong. The interpreter is always a valid fallback.
- **Preserve high-level information as long as possible.** Every lowering is
  permanent. Inline before you lower. EA before you lower. Loop-opts before
  you lower.
- **Profile-guide everything.** Every speculation is backed by MethodData.
  Without profile, don't speculate — fall back to conservative code.
- **Make the common case small and the uncommon case correct.** Uncommon
  traps are allowed to be slow; the fast path must fit in icache.

---

## When Your Cliff Click-Style Optimization Stops Working

A fallback decision tree for "my JIT used to be fast and now it isn't":

1. `-XX:+PrintCompilation` — did the method even reach Tier 4? If not, the
   bug is in tiering/thresholds, not in C2.
2. `-XX:+PrintInlining` — did the critical callees inline? If not, find the
   size/level/type-profile reason and fix *that*, not EA.
3. `-XX:+PrintEliminateAllocations` — did EA succeed? If not, check
   control-flow merges, receiver-type stability, and the 150-byte rule.
4. `-XX:+PrintDeoptimizationDetails` — is the method in a deopt loop? If
   `reason` repeats, the profile is lying; coarsen the speculation.
5. `perf stat -t <C2-tid>` — is C2 actually getting CPU? The compile log's
   "3571 ms" may be wall-clock while C2 only ran 300 ms starved by GC.
6. Still stuck? Enable `-XX:+LogCompilation` and feed the log through
   `java -cp $JAVA_HOME LogCompilation -i logfile.xml` for structured output.

Full command reference and the deeper diagnostic flow:
**READ `references/hotspot-c2-pitfalls.md`**.
