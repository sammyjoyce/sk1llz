# Worked example: Pólya's inscribed square (the canonical relaxation)

This is the problem Pólya used to demonstrate his method, and the one Schoenfeld used for decades to teach metacognition. It compresses almost every idea in the skill into one example. **Load this file only when you need to see the method end-to-end on a single problem** — e.g., you have explained the skill to a colleague and they want to see it work.

## The problem

> Given a triangle `T`. Construct, using straightedge and compass only, a square whose four corners all lie on the sides of `T`.

Four conditions:
1. All four corners of the square lie on sides of the triangle.
2. The four corners form a square (right angles, equal sides).
3. The construction must terminate.
4. Only straightedge and compass allowed.

## Where the naïve approach fails

The obvious move is to parameterize: let the bottom side of the square lie on one edge of `T`, let its length be `s`, solve for `s`. This works — but it requires algebra, and algebra is not a straightedge-and-compass construction. You have produced a formula, not a construction. **Pólya's point**: the wrong tool gives a wrong-shaped answer even when the numerical result is right.

## The Pólya move: relaxation

Apply **Family 1, sub-strategy "drop one condition"** from the heuristic families reference. There are four conditions. Which do we drop?

- Drop "four corners" → trivial and useless.
- Drop "square" → any rectangle inscribes easily, but we lose the problem.
- Drop "straightedge and compass" → gives a formula, not a construction.
- **Drop "all four corners on the triangle"** → keep three corners on the triangle, let the fourth float.

The fourth relaxation is the right one because it preserves the hardest structural constraint (being a square) while loosening the easiest (the last corner). **The decision of which condition to drop is the entire problem.** Novices drop any condition; experts drop the one that leaves the core structure intact.

## The relaxed problem and its family of solutions

"Construct a square with three corners on sides of `T` and the fourth floating."

This problem has **infinitely many solutions** — a one-parameter family. Pick one corner of `T` as the base vertex; you can construct squares of any size with two corners on one adjacent side and one corner on the other. The locus of the floating fourth corner, as the square's size varies, is a **straight line** through the base vertex.

This is the insight the relaxation delivers: the fourth corner moves along a line.

## The connection subproblem (Newell's wild subproblem)

Having the relaxed solution is not having the original solution. Newell's term: the **connection subproblem**. We know how to build squares with 3 corners on `T` and we know the 4th corner traces a line — how does that give us a square with all 4 on `T`?

Intersect the locus line with the third side of `T`. Where they meet is the position the floating 4th corner must occupy for it to *also* lie on `T`. Reconstruct the square of the corresponding size, and all four corners land on the triangle.

**This is the move novices never make.** They solve the relaxed problem, shrug, and return to square one. The discipline is: after solving the relaxed problem, *explicitly ask what is now computable about the original problem that wasn't before*.

## Look Back — four things, not one

1. **Verify**: pick any triangle, run the construction, measure the corners. They should hit the sides exactly.

2. **Derive differently**: can we use a homothety (scaling) argument instead? Yes — pick *any* inscribed square (even one with a corner off `T`), then scale/translate it until the floating corner lands on `T`. This is a completely different method, and it agrees with the relaxation method. Two independent derivations agreeing is strong evidence of correctness.

3. **Generalize**: does the method work for inscribing a rectangle of fixed aspect ratio? A regular `k`-gon? A **triangle** inscribed in another triangle? The answer is "yes" for the first, "yes for even `k`", and "requires a different move for triangle-in-triangle." Knowing where the method *breaks* is as valuable as knowing where it works.

4. **Transfer**: where else does "drop one condition → get a locus → intersect the locus with the dropped condition" apply? It is the core move in many geometric construction problems, *and it is the shape of Lagrange multiplier optimization* (drop the constraint, find the unconstrained optima, intersect with the constraint). The method generalizes far beyond geometry.

## What this problem teaches

Every step of Pólya's framework does real work here:

- **Understand**: isolate the four conditions. Without this, you cannot choose which to drop.
- **Plan**: the plan is the single decision of which condition to relax. Everything else follows.
- **Execute**: the execution is mechanical once the plan is right.
- **Look back**: the method generalizes to optimization with constraints — that insight is worth more than the original problem.

The ratio of thinking to doing is roughly 90/10. Novices invert it, then wonder why they're stuck.
