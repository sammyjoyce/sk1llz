---
name: stepanov-generic-programming
description: Derive STL-style C++ algorithms from proofs instead of from containers: choose the weakest iterator or concept that preserves correctness, preserve regular and value semantics, and state exact law and operation-count requirements. Use when designing or reviewing generic algorithms, iterators, ranges, comparators, rotate or merge style operations, function objects, or when prompts mention generic programming, regular types, iterator categories, concepts, value semantics, or "make this algorithm truly generic".
tags: generic-programming, algorithms, iterators, concepts, stl, value-semantics, regular-types, ranges
---

# Stepanov Generic Programming

This skill is for algorithm design, not for template cleverness. Treat templates, ranges, and concepts as the final encoding step, not the source of the idea.

This skill is intentionally self-contained. Do NOT load STL primers, template tutorials, or "modern C++ syntax" references unless the task is blocked on syntax rather than design.

## The Working Stance

Before you write a template signature, ask yourself:

- What theorem am I really trying to make true for all models of this algorithm?
- What is the weakest iterator or algebraic structure for which the proof still goes through?
- Which costs dominate here: comparisons, iterator increments, swaps, moves, assignments, or temporary memory?
- Which laws must hold: regularity, strict weak ordering, associativity, stability, valid-range preservation, non-overlap?
- Am I designing around the current container because it is convenient, or around the computation because it is fundamental?

Stepanov's default move is: start from an interesting algorithm, derive its proof obligations, extract the minimal interface, then map that interface into C++. Do not start from inheritance, class taxonomies, or template machinery.

## Derivation Procedure

1. Pick the algorithm first. If you cannot state the operation cleanly without naming a concrete container, you are not abstract enough yet.
2. Write the proof sketch against the weakest plausible model.
3. Extract the required operations and semantic laws from the proof, not from habit.
4. Count dominant operations separately. Big-O is not enough for generic code.
5. Only after the concept is stable should you encode it as iterator requirements, concepts, overload sets, or range constraints.

The quality bar is "general without efficiency loss." A concept is good when stronger models can use it without paying for abstraction, and weaker models are not excluded accidentally.

## Iterator Strength Decision Tree

- If the algorithm is single-pass and may consume the source as it advances, require only input iteration. In the weakest model, incrementing one copy can invalidate peer copies of the same position, so any hidden multi-pass assumption is a bug.
- If the algorithm must revisit positions or the caller must safely retain iterators after traversal, require forward iteration.
- If the proof needs backward motion, require bidirectional iteration and name the law that connects `++` and `--`.
- If the algorithm needs constant-time distance or safe arithmetic like `last - first`, require random access. "Can jump by n" is not enough; the key capability is efficient distance.
- If you need physical adjacency, cache-line reasoning, byte reinterpretation, or `memmove`-class optimizations, ask for contiguous access explicitly. Random-access iterators are not automatically contiguous; `deque` is the classic counterexample.
- If the asymptotic win comes from rewiring topology instead of moving values, model that as an additional node operation. Node-based capability is orthogonal to iterator category.

## Laws That Matter More Than Syntax

- A valid range `[first, last)` is the unit of reasoning. A lone iterator does not tell you whether dereference or increment is legal.
- Container mutation generally invalidates ranges. If your algorithm keeps offsets, save them before growth or reallocation.
- Regular functions preserve substitutability of equal values, except possibly complexity effects such as cache distance. Most worthwhile optimizations depend on this property.
- Sorted algorithms do not reason with `==`; they reason with comparator-induced equivalence: `!comp(a, b) && !comp(b, a)`.
- Whole-part value semantics are usually the right default. A copy should preserve observable meaning; address identity is not semantic identity.
- Return useful byproducts when you can do so without extra work. A well-designed `rotate` returns the image of the middle; forcing callers to rediscover it is wasted traversal.

## Performance Heuristics Experts Actually Use

- `lower_bound`, `upper_bound`, and `binary_search` on forward iterators still use logarithmic comparisons, but they can take linear iterator steps. If traversal cost dominates, prefer a container member that exploits topology, or materialize into a stronger representation.
- `equal_range` costs at most `2 * log2(n) + 1` comparisons; `binary_search` costs at most `log2(n) + 2`; `merge` costs at most `n1 + n2 - 1` comparisons and is stable.
- `merge` and `swap_ranges` are undefined on overlapping output ranges. If overlap is possible, redesign the algorithm instead of "hoping the implementation does the obvious thing."
- `transform` expects the operation to be side-effect free. In-place use is allowed only in the standard aliasing cases; generic code that mutates external state inside the operation quietly destroys substitutability.
- `inplace_merge` is a memory-adaptive algorithm: with enough buffer it needs at most `n - 1` comparisons, without buffer it may degrade to `O(n log n)`. Temporary memory is an algorithmic parameter, not an implementation detail.
- Benchmark loop-unrolling factors `4` and `8` first. Stepanov notes that if loop overhead starts around `30%`, unrolling by `4` drops it to roughly `8%`, and by `8` to roughly `4%`; beyond that the gain is often noise.
- Benchmark adaptive algorithms with tiny extra storage, not just "none" versus "plenty." Stepanov explicitly calls out `1%`, `10%`, and `50%` extra memory as useful checkpoints; even `1%` can flip the winning design.

## Rotation, Insert, and Permutation Thinking

When two adjacent ranges have sizes `a` and `b`, think in permutation cycles, not in "move the left half, then the right half."

- Bidirectional case: the three-reverse rotate costs `3n` assignments for `n = a + b`. It is not minimal, but it is often the best trade-off when the iterator model is weaker or when simplicity matters.
- Random-access case: the best rotate can reach `n + gcd(a, b)` assignments, which is the minimum because the permutation decomposes into `gcd(a, b)` cycles.
- Input-range insertion into `vector` is a classic trap. Naively inserting one element at a time costs `m * (tail + 1)` assignments for `m` inserted elements. Appending all `m` elements and then rotating reduces the work to roughly the linear sum of inserted length plus tail length.
- When growing a vector during such an insert, save `position` and `end` as offsets before the first `push_back`; iterator identity will not survive reallocation.

## Function-Object Heuristics

- Generic algorithms should usually take their dependent function objects explicitly unless the type can be derived unambiguously from other parameters.
- Passing small function objects by value is often the right default. The seductive "`const&` avoids copies" instinct can cost extra indirection and blocks the common zero-state or tiny-state case that compilers optimize well.
- If a comparator needs state, that is not a design smell; it is often the entire point of the abstraction. Hiding that state in globals destroys genericity and testability.

## Regular and Swap-Regular Types

- A type is not "regular" because it compiles with the standard concepts; it is regular when equal values remain substitutable under the operations your algorithm uses.
- Fast, non-throwing swap is not a micro-optimization. For resource-owning types it is what makes copy-then-swap assignment preserve the old value on failure.
- If your swap performs linear copies or allocation, the representation is probably wrong for generic programming. Stepanov's bias is to redesign the representation so swap becomes memberwise pointer exchange.

## NEVER

- NEVER start from the container or class hierarchy because the concrete type in front of you feels like the "real" design. That bakes accidental power into the interface and overconstrains every later algorithm. Instead derive the interface from the proof of the algorithm you actually want.
- NEVER strengthen an iterator requirement just to get `last - first`, `<`, or indexing because the stronger syntax is convenient. That silently excludes streams, linked structures, and other valid models, and it often hides accidental quadratic traversals. Instead keep the public contract weak and add stronger specializations only where they buy measurable benefit.
- NEVER treat comparator-based algorithms as if they were `==`-based because that mental shortcut feels harmless. Partial orders, NaNs, and inconsistent equivalence classes will break binary-search and set-operation semantics in ways that look nondeterministic. Instead define the ordering law explicitly and reason in terms of `!comp(a, b) && !comp(b, a)`.
- NEVER assume random access implies contiguous storage because pointers and `vector` make that seem natural. `deque` and segmented iterators invalidate pointer arithmetic, aliasing assumptions, and raw-memory optimizations. Instead ask for contiguous access explicitly when the algorithm needs physical adjacency.
- NEVER implement swap or assignment by copy-destroy-reconstruct for resource-owning types because it looks like the most generic code. It turns swap into linear work, can allocate, and can leave the object destroyed if construction fails. Instead design for swap-regularity and use copy-then-swap when you need the strong guarantee.
- NEVER pass comparators and predicates by reference purely out of copy-aversion because "references are cheaper" is seductive folklore. For the small stateless or tiny-state functors that dominate generic code, the extra indirection is pure loss. Instead pass by value unless the object is genuinely large or noncopyable.
- NEVER insert an input range into a vector one element at a time because the loop is easy to write and looks obviously correct. Repeated tail shifting makes the algorithm quadratic and invalidates the very iterators you are tempted to reuse. Instead append, save offsets, and rotate.

## Failure Recovery

- If the weakest correct concept is too slow, keep the weak API and dispatch internally to stronger iterator categories or container-specific fast paths.
- If you cannot state the law set precisely, stop generalizing. Most "generic" bugs are unstated semantic obligations, not template syntax mistakes.
- If a node-based algorithm would avoid value movement, expose that operation separately rather than pretending it is the same as a value algorithm.
- If performance depends on temporary memory, treat memory as an explicit tuning dimension and benchmark across no-buffer, tiny-buffer, and ample-buffer regimes before changing the abstraction.

## Modern C++ Mapping

Use `concept` clauses and ranges to encode the result of the derivation, not to discover it. A good `requires` clause is the shadow of a proof. If the proof is weak and the clause is strong, the interface is wrong.
