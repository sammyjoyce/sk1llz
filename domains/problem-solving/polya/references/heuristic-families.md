# Pólya Heuristic Families (Schoenfeld decomposition)⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​​‌​‌‌‍​​‌​‌​​‌‍‌‌‌​​​‌​‍​​‌‌​​​‌‍​​​​‌​​‌‍​‌​‌‌‌‌‌⁠‍⁠

When Schoenfeld's research group tried to teach Pólya's heuristics directly, students could not apply them. The reason: each "heuristic" in *How to Solve It* is not a single move — it is a **family** of moves that share a label. "Try a simpler related problem" names at least a dozen different procedures, each triggered by different features of the problem. To use Pólya's method you must commit to a specific family member, not the family name.

This file contains the full decomposition. **Load this file before planning any non-trivial problem.** Use it as a checklist: for each candidate heuristic, locate the row whose trigger feature actually matches your problem.

---

## Family 1: "Solve an easier related problem"

| Sub-strategy | Trigger feature | Canonical example |
|---|---|---|
| Set integer parameter to 1, 2, 3, 4 | Problem has an `n`, explicit or tacit | "Sum of first 97 odd numbers" — the 97 is an `n` |
| Try the lowest non-degenerate dimension | Problem is in 3D or higher | Inscribed cube in tetrahedron → inscribed square in triangle first |
| Drop one condition (relaxation) | Problem has ≥2 constraints | "Square with all 4 corners on the triangle" → "square with 3 corners on the triangle" |
| Replace general object with specific instances | Problem uses a general class | "Any quadrilateral" → square, then rectangle, then parallelogram, then trapezoid |
| Remove a level of nesting | Recursive or layered problem | 3-layer bug → reproduce with 1 layer |
| Substitute concrete numbers for variables | Symbolic problem | Replace `f(x, y)` with `f(2, 3)` first |
| Discretize or continuize | Problem in the "wrong" domain for your intuition | Integer optimization → real-valued relaxation |
| Remove the adversary / worst case | Problem has an adversary | "Any input" → "uniformly random input" first |
| Solve the **degenerate** case separately | Problem has a corner case | Empty list, single element, all-equal elements |

**Trap**: "Easier" does not mean "smaller". `n = 0` is often a degenerate case whose solution teaches you nothing. Pick the smallest `n` for which the problem is still structurally interesting. For sorting, that is `n = 3`, not `n = 1`. Schoenfeld calls this "specialization that preserves the structure."

---

## Family 2: "Work backward"

| Sub-strategy | Trigger feature | Note |
|---|---|---|
| Assume the answer exists, derive its properties | Existence question | "Suppose a solution exists. What must it satisfy?" |
| Start from the goal state, apply inverse moves | State-space search | Planning a recipe from the finished dish |
| Write the test/assertion first | Code or proof | TDD is exactly this heuristic |
| Guess the closed form, then verify by induction | Recurrence or summation | Works only if you can compute a few terms first |
| Invert the data flow | Pipeline problem | "What must stage 2 receive for stage 3 to produce X?" |

**Critical caveat**: working backward produces a chain of *implications*, not *equivalences*. Every backward step must be rechecked for reversibility. **Always finish by rewriting the chain in forward direction** — a step that worked backward but fails forward is the most common source of false proofs.

---

## Family 3: "Use analogy"

| Sub-strategy | Trigger feature | Note |
|---|---|---|
| Map structurally: data↔data, condition↔condition, unknown↔unknown | Rich analog exists | The isomorphism must hold on all three |
| Apply a known method to a new domain | Method is abstract enough | Sweep-line from geometry to interval merging |
| Translate to a different representation | Problem is awkward in its native form | Graph → matrix, recursion → iteration, iterative → generating function |
| Compare two problems *you have already solved* | You have ≥2 analogs | The difference between the two analogs is the clue |
| Cross-domain analogy | Pure stuckness | Physical → informational, continuous → discrete |

**Trap — surface vs. structural analogy**: surface analogies share vocabulary or symptoms; structural analogies share the condition (the relation between data and unknown). If your analogy fits the surface but the condition is different, it will lie to you. The canonical disaster: "this looks like the race condition we had last week" — symptom matches, mechanism differs, two days wasted.

**Diagnostic**: write out the mapping explicitly. If you cannot produce a bijection on data / condition / unknown, you have a surface analogy. Downgrade it to a hint, not a plan.

---

## Family 4: "Decomposition"

| Sub-strategy | Trigger feature | Note |
|---|---|---|
| Independent subproblems (divide & conquer) | Problem has disjoint parts | Sorting halves, tree traversal |
| Sequential phases (pipeline) | Output of one phase feeds next | Lex → parse → typecheck → codegen |
| Per-case analysis | Problem has natural categories | "If even, …; if odd, …" |
| Layered abstraction | Problem spans levels | Protocol stacks, algorithm vs. data structure |
| Temporal decomposition | Problem evolves over time | Before / during / after the transition |

**Trap**: decomposition only helps if the subproblems are actually *easier* than the whole. A common failure is to decompose into pieces that share a hidden global state — the interactions between pieces end up harder than the original. Before committing, ask: *can I solve subproblem A without knowing anything about subproblem B?* If no, the decomposition is fake.

---

## Family 5: "Auxiliary elements / introduce a helper"

| Sub-strategy | Trigger feature | Note |
|---|---|---|
| Add a variable | Algebraic problem | Let `x = ...` to make the relation explicit |
| Add a geometric construction | Geometry problem | Draw the perpendicular, extend the line |
| Add a data structure | Algorithm problem | Stack, queue, hash map — converts repeated work to O(1) |
| Add an invariant | Loop or recursion | "At every iteration, property P holds" |
| Add an intermediate representation | Compiler-style problem | AST, IR, normal form |
| Introduce a coordinate system | Spatial problem | Put the origin where it helps |

**Polya's rule**: the auxiliary element should appear nowhere in the original problem statement — that is the sign you have introduced genuinely new structure, not just renamed a given.

---

## Family 6: "Generalization (inventor's paradox)"

| Sub-strategy | Trigger feature | Note |
|---|---|---|
| Replace a constant with a variable | Specific problem | `f(7)` → `f(n)` |
| Remove an assumption | Over-specified problem | "Sorted array" → "any array" |
| Embed in a larger class | Special instance | "This triangle" → "all triangles with this property" |
| Strengthen the induction hypothesis | Induction proof stuck | Prove more to get more |

**Two kinds of generalization — only one works**:
- **Concentration** (good): extract the structural essence. Group theory concentrated algebra + number theory + geometry. Test: the general proof is *shorter* than the specific proof.
- **Dilution** (bad): wrap in abstraction that uses no new property. "Let X be any object with …" followed by arguments that only use the original special case. Test: the general proof is *longer* than the specific proof, or uses the same steps.

When in doubt: if generalization didn't give you a new tool, you diluted. Roll back.

---

## Family 7: "Specialization"

| Sub-strategy | Trigger feature | Note |
|---|---|---|
| Extreme case | Boundary intuition needed | `n → 0`, `n → ∞`, limit cases |
| Typical case | Need a concrete handle | Pick a "boring" representative |
| Symmetric case | Problem has symmetry | Equilateral triangle, balanced tree |
| Degenerate case | Boundary correctness | Empty, single, all-equal |
| Test-by-dimension | Physical problem | Check your equation has consistent units |

**Pólya's warning**: specialization that destroys the structure teaches you nothing. If you set `n = 1` and the problem becomes trivial, you have specialized too far. Pick the smallest `n` for which the problem is still *hard*, not the smallest for which it is possible.

---

## Family 8: "Vary the problem"

| Sub-strategy | Trigger feature | Note |
|---|---|---|
| Look at the unknown differently | Stuck on the unknown | "What is `x`?" → "What relations must `x` satisfy?" |
| Look at the data differently | Data seems irrelevant | Reorder, regroup, reinterpret |
| Look at the condition differently | Condition is opaque | Restate the relation in set / function / geometric terms |
| Solve the *opposite* problem | Yes/no or existence | Prove it is impossible — the obstacles become the proof |
| Perturb the problem slightly | Want to test robustness | "What if I change this datum by ε?" |

---

## Meta-rule: "One trick or a method?"

Pólya's distinction: **an idea used once is a trick; an idea used twice is a method**. After solving, ask: *where else does this specific move apply?* If you can name a second problem (not just a problem type — a specific problem) it solves, you have extracted a method. If not, you have a trick, and the next time you need it you will not find it.

This is the actual mechanism by which Pólya's "Look Back" turns problems into skill. Skip this step and the problem leaves no trace.
