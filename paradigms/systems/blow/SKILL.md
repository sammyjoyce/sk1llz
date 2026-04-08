---
name: blow-compiler-gamedev
description: "Build languages, compilers, asset pipelines, and game-engine subsystems in a Jonathan Blow style: prioritize compile-edit-run latency, explicit costs, data-first transforms, and compile-time power without a second meta-language. Use when designing or refactoring hot engine loops, ECS/data-layout code, build systems, asset tooling, language features, or creative tools where iteration speed and debuggability matter. Triggers: Jonathan Blow, Jai, compile-time execution, data-oriented, ECS, fast compile, no hidden costs, engine architecture, asset pipeline, metaprogramming, game tooling."
tags: compiler, language-design, game-dev, game-engine, data-oriented, metaprogramming, jai, performance, tooling
---

# Jonathan Blow

Use this skill as a compact philosophy/mindset guide. It is self-contained.
Do NOT load generic compiler or renderer background material unless the user
explicitly asks for history, citations, or a broader survey.

## Operate From These Bets

- Iteration latency is a product feature. Blow treats the compile-edit-debug
  loop as part of the programming model; around 3 seconds still feels
  interactive, and once changes take much longer, programmers batch edits,
  stop experimenting, and architecture calcifies around "don't touch it."
- Compile-time execution is only worth it when it deletes whole categories of
  glue code: reflection-based generation, table baking, bindings, serializers,
  layout derivation, and asset transforms with explicit inputs. If it merely
  moves opaque logic from runtime into the build, you made debugging worse.
- Data layout follows the query, not the noun. A game update/render loop should
  be shaped around "what fields do I touch, in what order, at what frequency,"
  not around a taxonomy of world objects.
- Compiler and build complexity belong inside the tool, not in the user's head.
  If callers must manage header order, package order, generator phases, or
  fragile build glue, the language is exporting its implementation debt.
- Hidden control flow is a correctness problem, not just a style issue.
  Exceptions, implicit allocations, virtual dispatch in hot loops, and magic
  cache-invalidation layers are expensive mainly because they destroy the
  programmer's ability to predict behavior under stress.

## Before You Design Anything, Ask

- What daily wait am I removing for the programmer: compile time, rebuild
  invalidation, runtime probing, asset iteration, or debugging latency?
- What concrete transform or query becomes simpler if I do this? If the answer
  is vague, the feature is probably ornamental.
- Am I inventing a second language, hidden lifetime model, or hidden build
  system when one direct primitive would do?
- If this feature becomes popular, what stale-state or partial-rebuild failure
  mode will users hit first?
- Can the compiler/tool absorb the complexity once, or am I about to charge
  every call site forever?

## Decision Heuristics

| Situation | Default move | Seductive wrong move | Fallback if default resists reality |
|---|---|---|---|
| Object graph is in the hot path | Flatten into homogeneous passes or job queues | Micro-optimize methods one object at a time | Keep the OO shell for authoring, but extract hot data into linear working sets |
| Build latency is dominated by generators or glue | Collapse phases into compiler-owned compile-time work with explicit inputs | Add another generator, header phase, or stub layer | Keep it as a standalone build step until inputs and invalidation are obvious |
| API needs callbacks with per-call state | Model callback as code plus payload | Use globals or thread-local tricks to fake captures | Require caller-owned payload storage if allocation becomes the new bottleneck |
| Query is rare but expensive to maintain incrementally | Compute the view on demand | Maintain a live sorted/indexed view every frame | Materialize a cached secondary view only after the query becomes frame-critical |

### If The Task Is A Language Feature

- Default to the smallest primitive that exposes the real power. Blow-style
  design prefers a few orthogonal mechanisms over a gallery of "convenience"
  features that each introduce new rules.
- Add syntax sugar only after the low-level model is solid. If you cannot state
  the storage, control-flow, and compile-time cost model in a paragraph, the
  sugar is premature.
- Treat "everyone else has this feature" as evidence against adding it until a
  shipped use case proves the need.

### If The Task Is A Callback Or Polymorphism API

- Start from "code pointer plus data pointer" as the default mental model.
  Captureless callback APIs look elegant, but they push users toward globals
  and mysteriously break once threading or re-entrancy enters the picture.
- Prefer compile-time specialization when the call happens in a hot loop or
  when hidden dispatch would blur the cost model.
- Prefer runtime dispatch only when heterogeneity is real, rare, and outside
  the frame-critical path.

### If The Task Is A Hot Engine Loop

- First identify the dominant query. "Update transforms," "expand bounds,"
  "run AI proximity tests," and "serialize dirty replication state" usually
  want different layouts; one canonical object layout is often the bug.
- Only choose pure SoA when the loop touches a few columns across many rows or
  when you need predictable SIMD/prefetch streams. If each step needs most
  fields of one entity, pure SoA can turn one good stream into several bad
  gathers; AoS or tiled AoSoA is often the saner compromise.
- Group work by transform type, not by object identity. Pre-parsed homogeneous
  job lists often beat virtual dispatch because the function becomes implied by
  the queue and zero-entry queues become free.

### If The Task Is Asset Or Build Tooling

- Prefer explicit inputs and deterministic outputs. Compile-time file, network,
  and OS access is powerful, but if the dependency boundary is not obvious, the
  user will not trust rebuilds.
- Build secondary indexes only for queries that happen on-frame or repeatedly
  in a tight tool loop. For rare queries, a one-shot sort/filter often beats
  paying maintenance cost every frame forever.
- Bias toward one obvious entrypoint over a graph of scripts. "Point the
  compiler at the top file and let it discover the rest" is a Blow-style win.

## Numbers And Thresholds That Actually Matter

- A branch misprediction around 23-24 cycles can be more expensive than the
  work it guards. In a classic game-engine example, a 12-cycle bounding-sphere
  update was cheaper than checking a dirty flag that usually failed.
- A main-memory miss on console-era hardware was measured at roughly 400+
  cycles. That makes instruction-count heroics irrelevant if your data is
  scattered.
- Rearranging data to be contiguous produced a roughly 35% win in that same
  case before any algorithm rewrite; reorganizing both data and passes dropped
  the scan from 19.6ms to 4.8ms, and careful prefetching reached 3.3ms.
- Prefetch only after you have made access predictable. Blind prefetch on
  irregular graphs or across many worker threads can thrash shared cache and
  steal wins from adjacent systems.

## Non-Obvious Working Rules

- If the guarded computation is cheap and the branch is noisy, recompute.
  Dirty flags are attractive because they look like "optimization," but in hot
  loops the branch, load stalls, and cache pollution often cost more than the
  math.
- If queries are rare, stop maintaining live indexes for them. A sorted view
  that is updated every frame is usually a tax unless the query is genuinely
  frame-critical.
- If a transform is currently hard-coded, keep it hard-coded until multiple
  real data shapes force abstraction. Generic containers and meta-frameworks
  are seductive because they look reusable, but they frequently lock you into
  the wrong algorithm and the wrong cache behavior.
- If performance work begins with "how do I optimize this object," you are
  already late. Reframe it as "what stream of facts do I need, and how do I
  walk it linearly?"
- If a feature requires a long apology, it is probably fighting the design.
  Blow-style systems prefer obvious power over impressive complexity.

## NEVER Do These

- NEVER build a "generic asset system" because it feels reusable. That path is
  seductive because it promises one pipeline for every game, but in practice
  you invent a weak language that matches no shipped title's migration path.
  Instead share boring plumbing and keep game-specific transforms concrete.
- NEVER hide hot gameplay, rendering, or simulation behind heterogeneous object
  graphs because OO extensibility feels clean. The concrete result is vtable
  traffic, scattered state, branchy dirtiness checks, and no useful prefetch.
  Instead flatten work into homogeneous passes or job tables.
- NEVER add a language feature because another language has it. The seductive
  part is status: closures, advanced type machinery, and effect systems make a
  language look modern. The consequence is more compile-time state, more
  lifetime rules, and more explanations per call site. Instead add the smallest
  primitive that solves a currently painful real use case.
- NEVER let compile-time execution read ambient machine state "just because it
  can." The seductive part is power; the consequence is non-reproducible
  builds, invalidation bugs, and an unclear trust boundary. Instead make the
  inputs explicit or move the side effect into a deliberate build step.
- NEVER ship captureless callback APIs plus globals because they look C-simple.
  The consequence is hidden shared state that fails under threading,
  re-entrancy, or multiple simultaneous users. Instead model callbacks as code
  plus payload from the start.
- NEVER assume dirty flags, cache invalidation layers, or "smart" laziness are
  free. They are seductive because they look like saved work; the consequence
  is extra branches, stale-state bugs, and debugging sessions spent proving the
  cache is coherent. Instead benchmark unconditional recompute, dirty lists, or
  exception queues.
- NEVER export build ordering, generator phases, or dependency plumbing to the
  user because it looks flexible. The consequence is that every programmer must
  memorize the compiler's internal problems. Instead let the compiler discover
  dependencies and own the rebuild order.

## When The First Move Fails

- No profile yet: do not fake precision. Remove obvious object-graph and
  hidden-dispatch costs first, then measure before changing algorithms.
- Data shape is unstable: resist giant frameworks. Use direct transforms with
  obvious boundaries so you can throw them away when the real data arrives.
- User wants safety or ergonomics over raw speed: keep the cost model explicit
  and wall the slower abstraction off from hot loops instead of pretending the
  cost vanished.
- Compile-time approach feels too magical: implement it once as ordinary code
  or a standalone build step, prove the value, then pull it into compile time.

## Expected Output Style

- Name the dominant constraint first: iteration latency, runtime bandwidth,
  data layout, or build determinism.
- Show the rejected alternative and why it is seductive.
- Make costs visible: hidden allocation, dispatch, branchiness, rebuild
  invalidation, data scattering, or extra mental model.
- Prefer a smaller primitive plus a clear migration path over a feature pile.
