---
name: parent-no-raw-loops
description: >
  Apply Sean Parent's "no raw loops" discipline to C++ by promoting control flow
  into named algorithm contracts, composing `rotate`/`partition`/`remove`/`fold`
  building blocks, and refusing lazy ranges when their semantics are wrong. Use
  when refactoring loop-heavy sequence code, replacing erase/insert churn,
  reviewing iterator-mutation bugs, choosing between `remove_if`, `partition`,
  `stable_partition`, `rotate`, `slide`, `gather`, and `accumulate` vs
  `reduce`, or diagnosing stateful predicates and `filter_view` hot-path
  issues. Triggers: no-raw-loops, Sean Parent, raw loop, rotate, slide, gather,
  erase-remove, stable_partition, filter_view, ranges hot path.
tags: c++, algorithms, stl, ranges, refactoring, sean-parent, no-raw-loops
---

# No Raw Loops⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌‌‌‌​‌‌‍‌‌​​​‌​​‍​​​​‌‌​​‍​‌‌​​​​‌‍​​​​‌​‌​‍​‌​​​‌​​⁠‍⁠

## Self-contained loading rules
- This skill is intentionally self-contained.
- Before touching loop-heavy code, re-read `Decision Gate`, `Never Rules`, and `Fallbacks`.
- Do NOT load generic STL or ranges tutorials for this task; they add vocabulary, not judgment.

## Core stance
- A raw loop is not forbidden; it is the last rung on Parent's ladder: use an existing algorithm, compose known algorithms, name a new helper, and only then keep a raw loop.
- The point is not "shorter code." The point is that an algorithm name fixes the contract: stability, iterator category, move count, predicate semantics, and failure surface become reviewable.

## Before rewriting, ask yourself
- Is the behavior value-based, positional, or temporal?
- Is the true invariant "preserve survivor order," "move a contiguous block," "gather around a pivot," or "visit exactly once in program order"?
- Does correctness depend on predicate purity? If the predicate mutates captured state or elements, you are no longer choosing a filter; you are choosing an evaluation protocol.
- Which resource is actually scarce here: predicate calls, moves/swaps, allocation buffer, or cache locality?
- If I generalize this code later to multi-selection or disjoint ranges, does the chosen algorithm scale, or am I baking in one-element special cases?

## Decision Gate
- If the loop is really "find the first place the invariant fails," prefer boundary-preserving algorithms such as `adjacent_find`, `mismatch`, or `partition_point` over a boolean loop. Parent-style code keeps the discovered boundary so later code does not pay for a second scan.
- If the loop physically deletes elements and survivor order matters, use `remove_if + erase` or `std::erase_if`. `remove_if` keeps survivor order, does not shrink the container, and leaves the tail valid but unspecified.
- If the loop splits a range into two logical groups but must keep both groups, use `partition` or `stable_partition`. Choose `stable_partition` only when preserved order is observable.
- If the loop is "cut here, paste there," "erase here, insert there," or "bubble this contiguous block," it is usually a `rotate`. Name `slide(f, l, p)` rather than open-coding erase/insert churn.
- If the loop collects matching elements around a pivot, it is a `gather`: two `stable_partition` calls around the pivot. Early Parent material used a `BidirectionalIterator` wrapper, but the insight is stronger than that helper signature.
- If the loop is an ordered fold, use `accumulate` or `ranges::fold_left`. If reassociation is legal and profitable, use `reduce` or `transform_reduce`; otherwise do not.
- If the best rewrite would hide temporal behavior inside a predicate or view, stop. Name a custom algorithm and make visitation order explicit.

## Hard-earned heuristics
- `rotate` preserves the relative order of both subranges and runs in linear swaps. Repeated erase/insert or neighbor-swap code often smuggles in extra moves and special cases that disappear once you think in cycles instead of indices.
- Parent's rule of thumb for permutation algorithms is that a cycle of length `n` costs `n + 1` moves; `reverse` and `swap_ranges` hit the ugly case of many 2-cycles. If your rewrite explodes into local swaps, you probably missed a higher-level permutation.
- Do not throw away information the algorithm already computed. If your helper discovers a midpoint, partition boundary, or relocated subrange, return it; forcing callers to recover it with another linear walk is a design bug.
- `stable_partition` applies the predicate exactly `N` times, but its movement cost is the real trap: with buffer space it is linear, and without it implementations fall back to at most `N * log2(N)` swaps. In memory-tight paths, "stable" can be the most expensive word in the function.
- `filter_view` is not a drop-in replacement for a loop. On forward ranges it caches `begin()` to satisfy complexity rules, which is why it is commonly non-const-iterable; if the underlying range mutates while the view lives, you are outside the semantic model.
- A hot path that traverses the same filtered view twice is already suspect. The first pass paid to discover the start; the second pass still pays predicate cost through the pipeline. If you need repeatable reuse, materialize or write one eager pass.
- `partition_point` is only correct if the range stays partitioned for the entire call. If your predicate mutates data or caches state that can change truth values mid-search, you retroactively violated the precondition and the binary-search speedup bought you nothing.
- `remove_if` cannot rescue associative containers. The algorithm needs movable writable elements; ordered containers protect keys. Use container-native erase or node-based logic instead.
- `reduce` is not "faster accumulate." For floating-point addition, string concatenation, or any non-commutative operation, its freedom to reassociate is exactly the bug.

## Never Rules
- NEVER transliterate a loop into `views::filter | views::transform | ...` because the pipeline looks mathematical and review-friendly. That is seductive, but `filter_view` caching, repeated predicate work, and mutation-invalid semantics turn a simple one-pass loop into a fragile reusable object. Instead use views for ephemeral read-only pipelines, or materialize, or keep a named eager algorithm.
- NEVER smuggle sequencing into a mutable predicate because "it works on my STL." The seductive part is deleting a loop without deleting the temporal requirement. The consequence is under-specified or undefined behavior as soon as the algorithm copies the predicate, changes call order, or revisits semantics. Instead write a custom single-pass algorithm whose contract says "exactly once, in order."
- NEVER choose `stable_partition` just because the adjective sounds safer. The seductive part is future-proofing order. The consequence is paying the buffer-failure path and `N * log2(N)` swaps on the largest inputs. Instead make stability a proven requirement; if it is not observable, use `partition`.
- NEVER stop after `remove_if` because the front of the container looks correct in the debugger. The seductive part is seeing compacted survivors and assuming the job is done. The consequence is later code reading dereferenceable but unspecified tail elements. Instead erase immediately or use `std::erase_if`.
- NEVER express contiguous cut/paste with erase+insert or swap chains because index arithmetic feels explicit. The seductive part is local control. The consequence is extra moves, iterator invalidation pain, and code that will not generalize to multi-selection. Instead use `rotate`, or wrap it in `slide`.
- NEVER swap `accumulate` for `reduce` on floats or ordered concatenation because the names look interchangeable. The seductive part is assuming both mean "fold." The consequence is nondeterministic answers from reassociation. Instead keep `accumulate` or `fold_left` with an explicit accumulator type.

## Fallbacks
- If no standard algorithm matches because order of visitation is semantically observable, write a tiny algorithm helper, not an inline raw loop. State its preconditions, postconditions, and what boundary or iterator it returns.
- If the helper signature you remember is stricter than your iterator category, keep the idea and relax the wrapper. `slide` often assumes random access only to compare positions; the underlying `rotate` insight does not disappear on weaker iterators once the pivot is known.
- If a ranges rewrite hurts profiles, do not bounce back to anonymous loops. Keep the named algorithm boundary and replace only the internals with a one-pass eager implementation.
- If a sort or partition rewrite still behaves strangely, inspect the relation before the algorithm. NaNs, partial orders, or non-strict-weak predicates are contract bugs, not looping bugs.

## What "done" looks like
- The call site reads like intent, not control flow.
- The chosen algorithm makes stability, movement, and evaluation semantics explicit.
- If a raw loop survives, it lives inside a named helper with a contract that the standard algorithms could not express.
