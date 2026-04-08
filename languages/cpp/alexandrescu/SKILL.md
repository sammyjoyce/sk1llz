---
name: alexandrescu-modern-cpp-design
description: "Write C++ in the Alexandrescu / Modern C++ Design tradition, but with the hard-won trade-offs experts use in C++20/23: when policy-based design is worth the API fracture, when it must be replaced by type erasure or named sibling types, and how to avoid the layout, ABI, propagation, and compile-time traps that policy-heavy libraries create. Use when designing generic libraries, deciding whether a behavior belongs in a template parameter, refactoring inheritance into orthogonal policies, reviewing Loki-style code, choosing between policies / CRTP / type erasure / `if constexpr`, or when the user mentions Alexandrescu, Loki, policy-based design, typelists, custom deleters, `[[no_unique_address]]`, or template metaprogramming trade-offs."
tags: policy-based-design, templates, metaprogramming, generic-programming, c++20, c++23, concepts, ebco, type-erasure, crtp
---

# Alexandrescu — Policy-Based Design (C++)

## Load map

- Stay in this file for: "should this even be a policy?", public-API triage, ABI/boundary review, and code-review comments.
- Before refactoring a real class into policies, READ [`references/decomposing-policies.md`](references/decomposing-policies.md).
- Before translating Loki-era machinery (`TypeList`, `Int2Type`, scatter hierarchies, SFINAE probes) into shipping C++20/23, READ [`references/modern-replacements.md`](references/modern-replacements.md).
- Do NOT load either reference for a boundary question (`DLL`, plugin, module, stable SDK, serialized type, vocabulary type). Those decisions are made here, and the answer is usually "hide the policy."

## Core law

Policy-based design is compile-time dependency injection for decisions that are already static inside one build. The win is not "flexibility"; the win is removing a runtime branch without making the user's type vocabulary explode.

Every policy parameter creates four costs at once:

1. A new type identity.
2. A new instantiation family.
3. A new layout/ABI possibility.
4. A new support matrix entry.

If the generated code gets faster but the public API becomes a set of incompatible islands, you did not get zero-cost abstraction. You moved the cost onto every caller.

## Choose the weapon before you write a template parameter

Use this decision ladder:

- Need the choice to vary at runtime per object or per request? Use runtime configuration, virtual dispatch, or type erasure. Not a policy.
- Need a local compile-time branch inside one implementation, but not a public type distinction? Use a tag + `if constexpr`.
- Need to mix behavior into an internal implementation type where the host never crosses an API boundary? A policy may be appropriate.
- Need open-ended extension across DLLs/plugins/teams? Use type erasure or a stable interface. Template identity is the wrong currency.
- Need self-referential static mixins on a closed hierarchy? CRTP. Do not dress CRTP up as a policy just because both use templates.

Counterintuitive rule: if the hot path already pays for heap allocation, I/O, RPC, or an indirect call, the public template parameter is usually wasted. Erase the variation and keep the type stable.

## Before adding a policy, ask yourself

1. Does this vary inside one shipping binary, or only between builds? If only between builds, hardcode/configure it and stop.
2. Are there at least two implementations in-tree today? If not, delete the parameter. "Someday" is not an axis.
3. Will values of this host cross a public boundary: DLL/plugin, shared library ABI, module interface, serialized format, or team-to-team SDK? If yes, the policy must not leak.
4. Is the host a vocabulary type (`optional`, `variant`, handle, string-like, span-like, result/expected, smart pointer)? If yes, assume policy exposure is a mistake until proven otherwise.
5. Does this axis change layout, exception guarantees, allocator propagation, or move-cost? If yes, it is not "just an implementation detail"; callers will feel it.
6. Do you already have more than 3 orthogonal axes, or more than about 8 supported combinations? If yes, you are designing a product matrix, not a class.
7. Can the same effect be achieved by a trait that maps `T -> canonical policy`? If yes, prefer the trait. It prevents `Foo<A>` and `Foo<B>` from coexisting for the same logical `T`.

## Where policy-based design still pays

- Internal containers, parsers, allocators, and ownership kernels where the host never becomes a shared vocabulary type.
- Storage/layout policies where branch removal changes codegen materially.
- Checking, threading, or error-reporting strategies on types that stay inside one subsystem.
- Fixed, closed sets of combinations that you can benchmark, assert on, and explicitly support.

## Where it usually loses

- Public vocabulary types. `basic_optional<T, Policy>` and `basic_variant<Ts..., Policy>` fracture every API boundary they touch.
- Custom deleter/allocator choices on public smart-pointer or container APIs. `std::unique_ptr<T, D>` is the cautionary example: the deleter is part of the type, unlike `shared_ptr<T>`.
- ABI boundaries. Policy-heavy headers are hostile to stable SDKs and plugin systems.
- Large codebases with many teams. Once callers start templating on your policy-composed type, you have exported your implementation matrix.

## Operating procedure when a policy is justified

1. First classify the axis as static-per-build, static-per-type, or runtime-per-object. Only the middle bucket is policy territory.
2. If you are decomposing an existing class, READ [`references/decomposing-policies.md`](references/decomposing-policies.md) before you sketch the first template parameter.
3. Write down the public consequences of the axis: type identity, layout, `sizeof`, `noexcept` move/copy, and whether values with different choices must interoperate.
4. Write the concept from the call site outward. If the concept exposes details of your first implementation, you have already overfit the policy.
5. Implement the cheapest stateless policy first. Then assert the invariants you are selling: `sizeof`, alignment, and `noexcept` move if those matter.
6. If the type will be instantiated in many TUs, count actual combinations used in production. For a small fixed set, ship named aliases and consider explicit/`extern template` instantiation for the hot combinations. For an open set, erase the boundary instead.
7. If the task is syntax modernization rather than design triage, READ [`references/modern-replacements.md`](references/modern-replacements.md). Do not spend main-skill tokens on typelist archaeology.

## Advanced heuristics practitioners actually use

- A policy is only "orthogonal" if swapping it does not force a different choice elsewhere. If `ThreadingModel` constrains iterator invalidation or allocator rules, you do not have two axes; you have one larger policy.
- The moment a policy changes observable move behavior, it stops being cosmetic. `std::pmr` is the canonical trap: one static type, but allocator propagation rules can make move-assignment copy/throw and make swap on unequal resources undefined.
- For custom deleters, measure object size before you debate elegance. On 64-bit, a function-pointer deleter typically makes `unique_ptr` two machine words; a `std::function` deleter is commonly around 40 bytes. A stateless function object can stay pointer-sized.
- `[[no_unique_address]]` is a layout hint, not a portable ABI promise. It may reuse tail padding, two same-type empty roles still need distinct identity, and MSVC historically accepts the standard spelling while only the vendor spelling has effect.
- If you can describe one axis only as "implementation detail users might want to override," you are probably looking at a trait or a private helper, not a policy.

## NEVER list

- NEVER expose a policy parameter on a vocabulary type because the seductive "someone may need a custom allocator/checker later" turns one concept into multiple incompatible types. Instead do one of: hardcode the implementation, ship 2-3 explicitly named sibling types, or erase the variation behind a stable facade.

- NEVER ship a policy-composed type across DLL, plugin, or stable-SDK boundaries because template identity and layout are compiler- and flag-dependent, which makes ABI drift a release-management problem. Instead do PIMPL, a C-like handle, or type erasure at the boundary and keep policies inside the implementation.

- NEVER treat `std::pmr` as a free cure for allocator-policy fracture because the static type becomes uniform while allocator propagation semantics still leak into move/swap behavior, so "cheap move" can quietly become copy-and-throw. Instead do `pmr` only when that propagation model matches the subsystem, or define a named allocator-aware type whose static and dynamic rules are explicit.

- NEVER inherit from policies in new C++20/23 code because EBCO-by-inheritance is seductive but fails on `final` policies, collides on duplicate empty roles, and drags base-class semantics into traits and diagnostics. Instead do `[[no_unique_address]]` members behind a portability macro and give duplicate roles distinct wrapper tags.

- NEVER build a policy-based owning pointer that auto-upcasts between unrelated deletion contracts because it feels flexible but recreates the exact `unique_ptr` hazard WG21 is still tightening: deleting through a base without a safe deletion contract, plus incomplete-type edge cases. Instead do conversion only when a delete-safety concept is satisfied, or erase/share ownership at the boundary.

- NEVER add a policy that has only one real implementation because speculative flexibility is seductive and permanent, while the concrete consequence is longer diagnostics, more instantiations, and an API break when you later try to remove it. Instead hardcode it and add the parameter only when implementation #2 ships.

- NEVER assume empty policies are free because same-type empty subobjects must still have distinct identity and "small" runtime state in a policy often destroys the layout guarantees you were counting on. Instead measure `sizeof`, add `static_assert`s for the stateless cases you support, and move real state either into the host or into a runtime-erased strategy object.

- NEVER promise exact object layout when `[[no_unique_address]]` is involved because the attribute is intentionally permissive and implementations differ, especially on MSVC. Instead promise only the properties you verify in CI: size ceilings, alignment, and move/noexcept invariants.

## Failure signature -> likely fix

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| Callers template every function on your host type | Policy leaked into vocabulary/API space | Accept via views, iterators, handles, or erase the boundary |
| `3 x 3 x 3 x 3` axes already exist on paper | You are in combinatorial hell before `T` even enters the picture | Collapse coupled axes, hardcode one, or expose named variants instead of all combinations |
| `sizeof(Foo)` jumps on one compiler or after a user-supplied policy | EBCO collapsed (`final`, duplicate empty role, or ignored `[[no_unique_address]]`) | Use member storage + portability macro + tagged roles + size asserts |
| Move assignment of allocator-aware objects copies or can throw unexpectedly | `pmr`/propagation semantics, not type identity, are now the bottleneck | Pre-seed compatible allocators, or stop using `pmr` as the public customization surface |
| Build times or debug-info size spike after "just one more policy" | Instantiation matrix went nonlinear | Alias the supported combos, explicitly instantiate hot ones, erase the rest |
| Diagnostics are unreadable | Concepts were replaced by SFINAE or the concept names implementation trivia | Rewrite the contract as a small named concept that reflects the axis, not the first policy |

## Final gut check

- If deleting one template parameter keeps the tests green, delete it.
- If two policy choices are never mixed in one process, prefer separate named types over one mega-template.
- If callers need interoperability more than they need branch-free codegen, choose stable type identity over policy purity.
