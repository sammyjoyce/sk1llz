---
name: stroustrup-cpp-style
description: Apply Stroustrup's 21st-century ISO C++ style: subset-of-superset design, tool-enforceable safety rules, semantic concepts, RAII ownership, and zero-overhead interface choices. Use when designing or repairing modern C++17/20/23 APIs, signatures, templates, move/noexcept behavior, raw-pointer migrations, virtual-base contracts, or C/ABI boundaries. Triggers: RAII, rule of zero, owner<T*>, not_null, span, string_view, concepts, enable_if, virtual destructor, noexcept move, pimpl, lifetime, invalidation, shared_ptr, unique_ptr.
---

# Stroustrup-Style C++

## Load protocol

- Before touching special members, raw handles, or Pimpl seams, MUST READ `move-and-raii.md`.
- Before changing a public signature or arguing about `T` vs `const T&` vs `T&&`, MUST READ `parameter-passing.md`.
- Before exposing a concept, public template, or overload set, MUST READ `generic-programming.md`.
- Load `philosophy.md` only for teaching material, design docs, or policy debates. Do NOT load it for routine implementation work.
- Do NOT load `references.md` during coding. It is a bibliography, not an execution guide.

## Operating model

Stroustrup's modern style is not "use newer syntax." It is a subset-of-superset strategy: first add a better abstraction (`span`, RAII handle, hardened container, semantic concept), then ban the lower-level escape hatch that made bugs normal. If you try to ban raw techniques before introducing the replacement abstraction, engineers route around the rule and the codebase gets worse, not safer.

Treat guidelines as targets for tools, not as taste. If a rule cannot be checked locally by compiler, static analysis, or a narrow review protocol, it will not scale past a small team. Recent Stroustrup profile work explicitly prefers rejecting hard-to-analyze code over pretending whole-program lifetime reasoning is affordable.

Optimize for semantic coherence, not minimal syntax. The seductive mistake in mature C++ code is making every interface reflect the current implementation detail. Stroustrup's guidance is the opposite: choose the contract that stays correct after the implementation gets faster, safer, or more instrumented.

## Before you edit, ask yourself

- What invariant becomes mechanically checkable after this change?
- After one `push_back`, `erase`, `sort`, reallocation, or thread handoff, which aliases, iterators, references, or views become invalid?
- Am I introducing a shape-only concept because it is easy to type, even though I cannot state the semantics in one sentence?
- Is this really a low-level exception, or did I fail to first build the abstraction that would let me forbid the low-level trick?
- If I am adding `noexcept`, can I prove it under logging, allocation, and future member changes, or am I just silencing a performance discussion?

If any answer depends on "the current implementation happens to...", redesign before editing.

## High-value heuristics

### Ownership and invalidation

- Non-owning types are edge types. `T&`, `T*`, iterators, `string_view`, and `span` are strongest at call boundaries and weakest as stored state.
- Any non-const container operation after taking an alias to an element should be treated as invalidating until proved otherwise. The safe default is: copy out what you need, mutate, reacquire the alias.
- Do not try to be clever with flow-sensitive lifetime proofs in ordinary code. Stroustrup's invalidation work explicitly narrows the initial safe subset to straight-line, locally checkable cases because "smart" non-local analysis becomes unaffordable fast.
- `owner<T*>` is not a smart pointer lite. It is a quarantine marker for code you cannot yet convert because of ABI, migration cost, or handle implementation details. If you introduce `owner<T*>`, also isolate the delete site.
- References are never owners, and they dangle in more ways than most code reviews catch: `vector` growth, storing a reference to an element before mutation, binding to a temporary result such as `std::max(x, y + 1)`, and escaping a local through a longer-lived object.

### Interfaces and performance

- "Cheap to copy" is a machine-word rule, not a vibes rule. Stroustrup/Sutter guidance says two or three machine words is the usual break-even point; beyond that, the extra indirection of `const T&` often wins.
- `string_view` and `span` are meant to be passed by value. `shared_ptr` is the counterexample: it may fit in two machine words, but copying it still performs refcount work, often atomic.
- Do not put smart pointers in a signature unless the function is expressing lifetime semantics. A function that only uses a `widget` should accept a `widget`, not force the caller into `shared_ptr<widget>`.
- `noexcept` on move is a semantic performance knob. Standard containers and algorithms use it to decide whether relocation can use move while preserving guarantees. One throwing move can turn a container-growth path from cheap move into copy fallback or make a move-only type unusable in practice.
- Base destruction is an interface choice, not boilerplate. If destruction through `Base*` is allowed, the destructor must be public and virtual. If it is not allowed, make it protected and non-virtual so accidental `unique_ptr<Base>` deletion cannot compile into UB.

### Generic programming

- Avoid single-property public concepts. `Addable` looks elegant and quietly accepts `std::string` concatenation and pointer arithmetic. That is how generic libraries become semantically incoherent.
- Do not minimize concept requirements to the current body of one algorithm. Stroustrup's warning is that this freezes interface requirements to today's implementation details, so a future optimization becomes an API break.
- Public concepts should correspond to a domain idea with semantics or axioms. If you cannot state the meaning, keep the requirement local as a private constraint or `static_assert`, not as part of the public vocabulary.
- Prefer concept-based overload selection over handcrafted `enable_if` hierarchies when the semantics are real. If the compiler can compute strictness, do not encode the dispatch lattice yourself.

### Errors and boundaries

- Inside ordinary C++ domains, pair RAII with exceptions so failure unwinds complete invariants instead of leaking partial state.
- At C, hard-real-time, or fixed-latency boundaries, translate once at the edge. Do not spread "error-code style everywhere" because one subsystem cannot tolerate exceptions or general free store.
- Resource safety is not just "no leaks." Holding locks, memory, or handles twice as long as necessary directly increases contention and can force more hardware/energy for the same work.

## Decision trees

### Choosing the ownership form

```
Need shared lifetime after this call?
|- No -> use value, T&, T*, span, or string_view as a borrow
|- Yes, but only one owner at a time -> value or unique_ptr
`- Yes, multiple owners truly outlive one another
   |- Can you name the cycle breaker now? -> shared_ptr + weak_ptr
   `- No -> redesign; hidden shared ownership is architecture debt
```

### Deciding whether a view may be stored

```
Need to keep data after the call returns?
|- No -> pass span/string_view by value
|- Yes, and this object owns the storage -> store the owner, derive the view on demand
`- Yes, but ownership is external
   |- Can the lifetime contract be stated and enforced locally? -> store a borrow with proof
   `- No -> copy the data or redesign the boundary
```

### Designing a public template contract

```
Is the requirement part of the public API?
|- No -> local requires/static_assert is enough
`- Yes
   |- Can you state semantics or an axiom? -> define/reuse a concept
   `- No -> do not publish a concept; keep it as a private constraint
```

### Migrating legacy raw-pointer code

```
Can the seam be wrapped without breaking ABI?
|- Yes -> add RAII/value wrapper first, then ban raw ownership internally
`- No
   |- Is the pointer owning? -> mark owner<T*>, isolate delete, use not_null/span around it
   `- Borrow only -> keep raw borrow form but tighten lifetime scope and invalidation points
```

## NEVER rules

- NEVER use `owner<T*>` as the final design because it is seductive "partial modernization" that preserves manual delete and lifetime ambiguity. Instead quarantine it at ABI or migration seams and convert the interior to RAII handles.
- NEVER store `string_view`, `span`, iterators, or references as members just because they are cheap, because their failure mode is delayed invalidation after unrelated container growth, reassignment, or background work. Instead store ownership or recompute the view at the edge.
- NEVER declare one special member in isolation because the language quietly changes which copy/move operations are generated, and a "tiny destructor" can silently turn moves into copies. Instead use the rule of zero or spell out all five deliberately.
- NEVER publish a one-operator concept because the easy syntax hides accidental matches and locks clients to a fake abstraction. Instead constrain with a semantic domain concept or keep the check private.
- NEVER take `shared_ptr<T>` or `const shared_ptr<T>&` for read-only access because it couples callers to one lifetime policy and adds refcount traffic for no semantic gain. Instead take `T&`, `T*`, `span`, or `string_view`, and use smart pointers only when ownership is the contract.
- NEVER add `std::move(local)` on return from a normal function because it is seductive cargo-cult optimization that can block elision and obscures the ownership story. Instead return the local directly and let copy elision do its job.
- NEVER promise broad `noexcept` on public operations because the reward is often speculative while the failure mode is `std::terminate` after a later member or logging change. Instead reserve `noexcept` for moves, swaps, destructors, and other paths you can actually prove.
- NEVER rely on comments such as "this iterator remains valid" after mutation because the next refactor will not read the comment, only the code. Instead structure the code so aliases are reacquired after invalidating operations.
- NEVER make a base destructor public and non-virtual because it feels cheaper than deciding the abstraction boundary, but it leaves deletion through `Base*` as a latent UB trap. Instead choose explicitly: public virtual or protected non-virtual.

## Fallbacks and edge cases

- If invalidation is difficult to prove, replace alias-based logic with index/key-based logic, perform the mutation, then reacquire references.
- If concepts are unavailable on the toolchain, keep the semantic contract in a named comment and add `static_assert` checks at the type or call boundary rather than rebuilding an `enable_if` maze.
- If a move cannot honestly be `noexcept`, do not fake it for container performance. Consider a stable indirection strategy instead of lying to the type system.
- If a real-time or firmware profile bans exceptions or general free store, confine that ban to the profile boundary. Do not downgrade the whole codebase to C-with-classes.
- If a C interface forces raw pointers, pair them with `not_null`, `span`, `zstring`, or explicit size/lifetime comments at the seam and keep the unsafeness from spreading inward.

The Stroustrup target is not "code that looks modern." It is code whose safety and performance story survives tooling, refactoring, and scale.
