---
name: lattner-compiler-infrastructure
description: Expert guidance for LLVM, Clang, Swift, and MLIR compiler infrastructure in Chris Lattner's style: choosing IR boundaries, pass placement, legality strategy, diagnostics, and semantic traps. Use when building or reviewing frontends, dialects, passes, lowerings, canonicalization, dialect conversion, optimizer pipelines, or debug-info-sensitive transforms. Triggers: llvm, mlir, clang, pass manager, canonicalize, fold, dialect conversion, opaque pointers, gep, freeze, debugify.
---

# Lattner Compiler Infrastructure

## Load Only What You Need

Before changing an LLVM pass pipeline, read `Using the New Pass Manager`.

Before touching LLVM IR semantics, read `UndefinedBehavior`, `GetElementPtr`, and `OpaquePointers`.

Before writing MLIR canonicalizations or rewrite patterns, read `Operation Canonicalization` and `Pattern Rewriting`.

Before writing legalization or type-lowering code, read `Dialect Conversion`.

Before verifier or assembly-format work, read `DefiningDialects/Operations` and `Diagnostics`.

Before deleting, RAUWing, or merging IR instructions, read `How to Update Debug Info`.

Do NOT load MLIR dialect-conversion material for a pure LLVM alias-analysis or opaque-pointer bug.

Do NOT load LLVM IR UB docs first for a verifier-formatting bug; use MLIR verifier-ordering docs instead.

## Core Stance

- Separate "what is legal" from "what is profitable". Pipelines rot when canonicalization, legalization, and optimization leak into each other.
- Add a new IR level only when invariants change. If two adjacent IRs allow the same illegal constructs and run the same analyses, one of them is cargo cult architecture.
- Prefer reusable infrastructure over one successful compile. A pass that only works inside one fixed pipeline is a future bug, not an abstraction.
- Make semantics explicit early and target details late. The later you encode target quirks, the less IR you poison with backend accidents.
- Treat diagnostics and debug info as semantic contracts, not polish. Broken locations degrade SamplePGO and make later debugging misleading.

## Before You Change The Architecture, Ask

- What invariant becomes true after this phase that was false before it?
- Could this be a local `fold` instead of a canonicalization pattern, or a canonicalization instead of a conversion?
- Am I encoding profitability into legality? If yes, split them.
- Does this transform need fixpoint revisits, or should it run once with explicit ordering?
- If this pass deletes IR, which cached analyses or debug facts become stale immediately?
- If this introduces a new IR or dialect, what transformation becomes simpler enough to justify its maintenance cost?

## Decision Tree

If the rewrite is single-op, local, and can only reuse existing values or attributes, implement `fold`.

If the rewrite may create ops, erase ops, or depends on non-local shape, use a rewrite pattern.

If the goal is "make later analyses easier", canonicalize.

If the goal is "make previously illegal IR legal", use dialect conversion or explicit lowering.

If the goal is "improve generated code", use an optimization pass after legality is established.

If a backend needs special lowering for an address computation, do not special-case GEP semantics; lower the resulting ADD and MUL tree in the backend.

## LLVM Heuristics That Save Months

- Use immediate UB only when hardware usually traps. Otherwise prefer poison-style semantics so speculation and hoisting remain legal.
- `undef` is effectively deprecated; reserve it for uninitialized loads. If you need a stable arbitrary value, use `freeze`, not `undef`.
- Branching on deferred UB is itself immediate UB. Any transform that hoists a condition, unswitches a loop, or duplicates control flow must ask whether the condition needs `freeze` first.
- `select` is a poison barrier in ways `and` and `or` are not. Replacing `select` with boolean ops is only safe when poison behavior is preserved, not when the truth table merely looks equivalent.
- Under opaque pointers, identical pointer operands no longer imply identical access types. Store-to-load forwarding, AA shortcuts, and memory combining must compare the load and store types explicitly.
- `inbounds` GEP overflow produces poison, not wrapped arithmetic. Add `inbounds` only when you want that semantic commitment.
- GEP is not generic pointer arithmetic across objects. If you cross from one allocated object into another with GEP, alias reasoning becomes unsound. If you truly need cross-object arithmetic, use `ptrtoint` and `inttoptr` and accept weaker optimization.
- Do not infer aliasing from LLVM's type system. Use TBAA metadata or real provenance facts; LLVM IR types alone do not enforce source-language aliasing rules.
- Group function, loop, and CGSCC passes inside the same pass manager. Repeated adaptors look harmless, but they worsen cache locality and can change optimization quality.
- If a module pass deletes functions and you queried inner analyses, clear or invalidate them immediately. Stale cached analyses keyed by dead IR addresses create heisenbugs that only appear in long pipelines.
- Preserve analyses selectively only after you can prove the updater is correct and the compile-time win is measurable. Over-preservation is more dangerous than recomputation.
- When you create, RAUW, or delete instructions, plan the debug-info update at the same time. Use RAUW where possible, `salvageDebugInfo` when not, and run `debugify` or `verify-debuginfo-preserve` before declaring the transform done.

## MLIR Heuristics That Save Months

- Default region scoping is more permissive than many dialect authors expect. If a region must not capture outer values, mark it `IsolatedFromAbove` or verify it explicitly.
- Canonicalization is not a performance pass. Put only cheap, semantics-preserving, convergence-friendly rewrites there; O(n) matchers inside a greedy fixed-point pass turn into compile-time cliffs.
- Implement a canonicalization as `fold` whenever possible. `fold` is reusable through `createOrFold` and dialect conversion; a pattern is only justified when locality is insufficient.
- Pattern benefit must be effectively static. If you want dynamic cost, instantiate multiple patterns with predicates rather than sneaking dynamic profitability into the driver.
- Give rewrite patterns explicit root op names whenever possible. Match-any patterns feel flexible, but they cripple cost-model reasoning and make debugging filters less useful.
- Recursive rewrites must call `setHasBoundedRewriteRecursion`. Otherwise the driver is correct to treat self-application as a likely bug.
- Use the Walk driver when you want one cheap traversal and do not need revisits. Use Greedy when you need transitive cleanup to a fixed point. Do not pay Greedy overhead for one-shot simplifications.
- Greedy canonicalization defaults are already opinionated: `top-down=true`, `max-iterations=10`, `region-simplify=normal`, `max-num-rewrites=-1`. If you hit non-convergence, use `test-convergence` to surface cyclic patterns instead of hiding them with looser limits.
- Dialect conversion operands lie in a useful way: the matched op still has original types, while the adaptor values may already have legalized types. Mixing them casually creates "why did verification suddenly fail" bugs.
- In rollback mode, replacement and erasure can be delayed. That makes legalization safer but IR dumps harder to trust because old and new IR coexist. Switch to no-rollback when debugging conversion mechanics, then switch back if you need search or backtracking.
- `unrealized_conversion_cast` is a debugging clue, not a target state. If it remains after a conversion, your type and materialization story is incomplete.
- Region argument type changes are never automatic enough. If you changed region signatures, call the explicit region conversion hook and audit every surviving user.
- Verifier ordering matters. Put checks that only depend on op-local invariants in `hasVerifier`; put nested-op checks in `hasRegionVerifier`. Otherwise you will inspect malformed children and debug ghosts.
- Never rely on custom printers inside verifier errors. Verifiers run before the printer can assume the op is well-formed; use generic printing and `-mlir-print-op-generic`.

## NEVER Do These

- NEVER add a new IR or dialect because the current one feels "messy". That path is seductive because it postpones hard invariant design. Instead define the invariant you need and prove it cannot live in the current IR.
- NEVER put profitability-heavy rewrites into canonicalization because canonicalization runs everywhere. It is seductive because it avoids choosing a pipeline slot. Instead keep canonicalization cheap and move profitable transforms into explicit passes.
- NEVER replace `select` with boolean algebra just because the truth table matches. The seductive part is local simplification; the consequence is poison miscompiles. Instead re-check poison propagation or insert `freeze` when required.
- NEVER assume pointer equality implies memory-type equality under opaque pointers because that used to be mostly true with typed-pointer-era bitcasts. The consequence is invalid forwarding and AA conclusions. Instead compare access types explicitly.
- NEVER use GEP to model cross-object pointer arithmetic because the integer result looks correct in dumps. The consequence is broken alias and provenance assumptions. Instead use integer arithmetic plus `inttoptr` only when you intentionally want to leave LLVM's object-based alias model.
- NEVER mutate MLIR IR directly inside a pattern because it seems faster than plumbing the rewriter. The consequence is invalid driver state, missed revisits, and non-reproducible bugs. Instead route every mutation through `PatternRewriter`.
- NEVER preserve analysis proxies after deleting IR unless you manually cleared every dead key and measured the compile-time gain. The seductive part is avoiding recomputation; the consequence is stale analysis state that surfaces far from the bug. Instead invalidate conservatively first.
- NEVER drop or fake debug locations as cleanup because "it only hurts debugging". The non-obvious consequence is worse SamplePGO mapping and misleading stepping. Instead preserve, merge, remap, or mark locations as compiler-generated, dropped, or unknown according to the transform.

## Fallback Playbooks

If an LLVM transform starts miscompiling after an "obvious" simplification, re-check UB, poison, `freeze`, opaque-pointer typed-access assumptions, and GEP provenance before debugging the algorithm.

If an MLIR canonicalization oscillates, enable `test-convergence`, add debug labels, and inspect whether the rewrite should have been a `fold`.

If dialect-conversion dumps look nonsensical, switch to no-rollback, inspect adaptor versus original operand types, and trace where `unrealized_conversion_cast` enters the graph.

If verifier errors are unreadable, rerun with `-mlir-print-op-generic`, `-mlir-print-ir-after-failure`, and `-mlir-print-stacktrace-on-diagnostic`.

If debug-info regressions appear after an IR transform, start with `opt -debugify -pass-to-test -check-debugify sample.ll`; on large pipelines cap original-DI verification with `-debugify-func-limit=100` before widening the run.

## What Good Output Looks Like

- Every phase has one dominant invariant and one clear exit contract.
- Canonicalization reduces variation, conversion changes legality, and optimization changes cost.
- Passes declare invalidation honestly.
- IR printing and diagnostics help you localize failure without depending on already-valid IR.
- Removing debug info is rarer than fixing it.
