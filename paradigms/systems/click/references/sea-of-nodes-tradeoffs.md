# Sea of Nodes — Trade-offs They Don't Teach in Class⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​‌‌​‌‌‍​‌‌​‌​‌‌‍‌​​​‌‌​​‍‌‌‌‌‌‌‌​‍​​​​‌​‌​‍‌​‌​‌​​​⁠‍⁠

Load this file ONLY when deciding between Sea of Nodes and CFG IR, designing
a new compiler's middle end, or debugging a SoN implementation (IGVN
non-termination, GCM placement bugs, memory chain explosions). Do NOT load
for HotSpot tuning — see `hotspot-c2-pitfalls.md`.

---

## The Case For Sea of Nodes (Click's Original Pitch)

Data and control are a single graph. Dependencies are *only* the true
dependencies: def-use for data, phi + region nodes for control, and a
*separate memory edge* for ordering side-effects. Consequences:

- **Reordering is free.** Any topological order of the graph is a legal
  schedule. Code motion is a scheduling problem, not an optimization.
- **DCE is a single sweep.** A node with zero uses is dead. Period.
- **GVN is the representation.** Two nodes with identical opcode and inputs
  *are* the same node — there is no "find" pass, the hash-cons IS the IR.
- **Peephole + GVN + constant prop + unreachable code elimination combine
  into one fixed-point ("IGVN" in C2).** Click's 1995 thesis proved the
  combined analysis strictly subsumes any sequential composition: combining
  optimizations finds things sequential passes miss.

This is why C2 quickly produces high-quality code. It is also why you should
not reach for SoN by default in 2026.

---

## The Case Against (Why V8 Left and You Might Too)

V8's Turbofan team ripped out SoN in favor of a CFG IR (Turboshaft) in 2022.
Their published postmortem (`v8.dev/blog/leaving-the-sea-of-nodes`, 2025) is
required reading. The empirical damages:

| Problem | Impact |
|---------|--------|
| Load elimination rewrite on CFG | **up to 190× faster** than the SoN version |
| L1 dcache miss rate across middle end | **3× on average, up to 7× in some phases** |
| End-to-end compile time | **divided by 2** after switching to CFG |
| Engineer onboarding + debuggability | qualitatively "much easier" |

The root causes are architectural, not implementation quality:

### 1. Most interesting nodes can't actually float

For JavaScript (and any language with checked operations), every load, store,
checked-add, division, and call needs *both* an effect input (ordering vs
other memory ops) *and* a control input (the type/bound check that precedes
it). The result is a "soup of nodes" that looks like a graph but behaves
like a CFG with extra steps. SoN's advantage is smallest exactly where the
source language needs optimization most.

### 2. Visitation order is fundamentally backwards for peephole

Peephole optimization wants to process inputs before uses so that one pass
reaches a fixed point. CFG gives you this for free. SoN has no entry point
for pure nodes, so the canonical traversal is `return`-up, which means you
re-process nodes repeatedly as their inputs get simplified. IGVN workloads
need explicit worklists to approximate what a CFG gets by iteration order.

### 3. State tracking is expensive

Load elimination, escape analysis, and similar state-propagating passes need
to know "what's live at this point." In CFG that's per-basic-block. In SoN
that's per-node and you don't know which nodes still need a given state
until you've visited their uses — a circular dependency that forces either
bailouts on large graphs or pessimistic joins.

### 4. Cache unfriendliness from in-place mutation

SoN passes rewrite nodes in place. New nodes get allocated in a bump pool
far from the originals. After N passes, graph traversal is a random walk
through memory. CFG IRs can reserialize basic blocks into fresh arrays
between passes and restore locality.

### 5. "What's inside this loop?" is hard

A node floats until you schedule it. Loop unrolling, loop peeling, and loop
invariant code motion all need to know precisely which nodes belong to the
loop — exactly the information SoN deliberately discards. You end up
computing pseudo-schedules to approximate loop membership and then throwing
them away. CFG just uses dominators.

### 6. SoN destroys any source-level scheduling

For ahead-of-time compiled input (WASM from C++/Emscripten, already scheduled
for register pressure and instruction latency), SoN blows it all up at graph
construction and must rebuild from scratch in GCM with far less information
than the AOT compiler had. Result: measurably worse WASM codegen in Turbofan.

---

## When SoN Is Actually The Right Choice

1. **The input language is statically typed with few side-effect operations.**
   Java is borderline; OCaml or pure functional code wins. Lots of loads,
   arithmetic, and pure functions → more nodes float → real speedup.
2. **You need combined analyses (Click's thesis insight).** If your
   optimization space genuinely benefits from conditional-constant-propagation
   interleaved with DCE interleaved with GVN, SoN's fixed-point IGVN finds
   things sequential passes cannot. This is rare in practice but real.
3. **You already have a working GCM and can debug it.** Global Code Motion
   (Click–Paleczny PLDI'95) places floating nodes into basic blocks using
   dominator trees + loop depth. It's subtle. Budget months to get it right.
4. **You control the entire pipeline.** C2 works because HotSpot owns the
   profile format, the deopt infrastructure, the runtime. Bolting SoN onto
   an existing compiler rarely pays off.

---

## If You're Building SoN Anyway: The Traps

### IGVN (Iterative Global Value Numbering) non-termination

IGVN processes a worklist: whenever a node changes, re-queue its uses.
Without care this loops forever. Invariants that must hold:

- **The lattice must be monotonic.** Every rewrite moves nodes *down* the
  type lattice (more specific, not less). A single non-monotonic rewrite
  and IGVN may oscillate between two equivalent forms indefinitely.
- **Hash-cons uniqueness.** Two nodes that become equal after rewriting
  must be merged, not kept as duplicates. Missed merges leave "zombie"
  nodes whose uses never notice updates.
- **Worklist must shrink monotonically in the limit.** If you ever need to
  re-add a node you just processed, you have a bug upstream.

C2's workaround: a "verify" mode that runs IGVN twice and asserts the graph
didn't change. Run this in every debug build.

### Memory phi chains in long basic blocks

Every store creates a new memory state. A basic block with 1000 stores has a
memory chain of length 1000. Load elimination must walk this chain to find
the most recent store to an alias. Naive implementations are O(N²); C2 uses
memory "slice" splitting (one chain per alias class) to get back to O(N).
Budget memory for this — it's not optional once blocks get big.

### Global Code Motion pitfalls

1. **Schedule early for DCE + late for register pressure.** Click's GCM pins
   side-effect nodes, schedules pure nodes as late as possible (smallest
   live range), then hoists loop-invariant work out of loops. Getting the
   "latest" placement wrong wastes registers. Getting it too early bloats
   live ranges across the entire function.
2. **Loop-invariant hoisting must respect deopt points.** Hoisting a load
   out of a loop before a checked operation moves it across a potential
   deopt — the interpreter state at that point no longer matches. C2
   handles this with "anti-dependence" edges; forgetting them = miscompile.
3. **Phi inputs are order-dependent.** SoN blurs "which predecessor did
   this input come from." You need a Region node whose input order matches
   each phi's input order exactly, forever.

### Peephole ordering bug class

In SoN, `x + 0 → x` is a rewrite. So is `(x << 2) + (x << 2) → x << 3`. Applied
in the wrong order, `(x + 0) << 2 + (x + 0) << 2` may not simplify because the
outer rule fires before the inner simplifies. Solution: run rewrites bottom-up
in the worklist, but this still requires you to re-queue parents whenever a
child simplifies. Miss a re-queue = miss an optimization.

---

## Click's Own Retrospective (Coffee Compiler Club discussions + the Simple
## tutorial series)

Click has publicly acknowledged the pain points. His current teaching
compiler, *Simple* (github.com/SeaOfNodes), is a 2024 rewrite explicitly
designed to make SoN approachable. Key concessions in Simple vs C2:

- **Explicit scheduling phase early**, not at the end. You still get the
  fixed-point IGVN benefits, but you get a CFG back for late passes.
- **Memory edges are first-class, not afterthoughts.** C2's memory model was
  bolted on; Simple designs it in from day one.
- **Peephole is combined with GVN at node-creation time**, not a separate
  pass. `new Node(...)` immediately canonicalizes and hash-conses.

If you're learning SoN in 2026, start with *Simple* (Java, Rust, Go, or C++
ports available), not with the C2 source tree.

---

## The Pragmatic Middle Path

Most production compilers in 2026 use **CFG with SSA + a few SoN-inspired
tricks**:

- LLVM: CFG with SSA, hash-consed constants, worklist-driven InstCombine
- Cranelift: CFG with SSA, egg-style e-graph rewrites (recent)
- Turboshaft: CFG with SSA + graph-based rewriter pattern from Turbofan
- Graal: still SoN, but with Partial Escape Analysis bolted on top —
  simultaneously the best EA on the JVM and the slowest compilation

You can have Click's combined-analysis benefits without the scheduling
nightmare by: (1) using SSA on a CFG, (2) running a worklist-driven
iterative GVN pass, (3) hash-consing expressions in a side table, (4) doing
peephole inside the worklist. You lose the "DCE is free" property but gain
debuggability and compile speed.
