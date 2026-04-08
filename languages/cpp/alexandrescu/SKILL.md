---
name: alexandrescu-modern-cpp-design
description: Write C++ libraries in Andrei Alexandrescu's Modern C++ Design / Loki style, with the expert trade-offs, failure modes, and C++20/23 replacements practitioners only learn after shipping policy-based code. Use when designing generic templated libraries, deciding whether to add a template parameter, refactoring inheritance into orthogonal compile-time policies, avoiding the type-identity trap that fractures APIs, choosing between CRTP / policy-based / type-erasure / `if constexpr`, replacing SFINAE with concepts, diagnosing EBCO failures on final or duplicate-type policies, or when the user mentions policy-based design, Loki, Modern C++ Design, Alexandrescu, typelists, `[[no_unique_address]]`, or template metaprogramming trade-offs.
tags: policy-based-design, templates, metaprogramming, generic-programming, c++20, c++23, concepts, ebco, type-erasure, crtp
---

# Alexandrescu — Policy-Based Design (C++)

## Prime directive

The value of policy-based design is **not flexibility** — it is turning decisions that are already static into *types*, so the compiler enforces them and the optimizer erases them. Its signature failure mode is turning the types you intended as *vocabulary* into incompatible islands.

If you stop reading here, remember: **every template parameter is a commitment to fragmenting your public API**. Pay that cost only when the variation is real, orthogonal, and worth the fracture.

## Before you parameterize anything, ask these five questions

1. Does this decision ever vary **inside a single build**? If no → hardcode it; stop.
2. Are there **≥2 concrete implementations today** (not hypothetical)? If no → delete the template parameter; add it the day the second implementation arrives.
3. Would `Foo<PolicyA>` and `Foo<PolicyB>` **deserve distinct names** in your public API? If yes → make them separate classes sharing an internal helper. This is Alexandrescu's own retrospective advice and the reason `std::unique_ptr`-with-`Deleter` is considered a cautionary tale.
4. Is the host a **vocabulary type** (optional, variant, expected/result, handle, span-like, string)? If yes → reject the policy parameter. Vocabulary types that vary by policy destroy API composability (see "The type-identity trap" below).
5. Does the host **already pay for heap + indirect call** on the hot path (I/O, allocation, RPC)? If yes → type-erase the policy; your public surface must stay un-parameterized.

If you can't answer confidently, you are over-templatizing. **The most expensive policies are the ones nobody ever specializes.**

## The type-identity trap (the #1 lesson practitioners learn the hard way)

`vector<int, PoolAlloc>` and `vector<int, StackAlloc>` are **different types**. A function taking one cannot take the other. Every policy parameter on a widely-used type fractures the codebase into incompatible silos, and every boundary between silos costs a copy or a template-everywhere epidemic.

Treat the remedy as a tiered decision, not a single trick:

| Tier | When | Remedy |
| --- | --- | --- |
| **Do-not-parameterize** | Vocabulary types (optional, variant, result, handle) | Pick one implementation. Or ship 2–3 *named* aliases (`optional<T>` vs `optional_ref<T>`) and document them as distinct types sharing an interface. |
| **Type-erase** | Hot path already pays for heap/indirection | Public surface takes an erased facade (`any_allocator_reference`, `std::function`, `pmr::memory_resource`). Internal code still uses the template. |
| **Accept-via-view** | Non-mutating API on a parameterized container | Take `span<T>` / `basic_string_view<T>` / iterator pairs, not the container itself. |
| **Automate** | Policy depends purely on a type parameter `T` | Define a trait (`storage_policy_for<T>`) and use `foo_for<T>` so the *same* `T` always resolves to the *same* policy across the codebase. Prevents users from mixing `compact_optional<Foo>` with `regular_optional<Foo>` in the same API. |

`std::shared_ptr` took the type-erase tier for its deleter; `std::unique_ptr` did not. The difference in practical pain is the clearest empirical case for this decision.

## Decomposition rules (how to actually split a class into policies)

Before splitting, list every axis of variation. Reject any axis where: the axes touch each other's state; the correct choice on one axis constrains another; or there is only one non-dummy implementation.

- **≤3 policies per class.** Four or more almost always means one axis should have been a trait or a default, or two axes collapse into one.
- **No policy may know another exists.** If `PolicyA`'s interface references `PolicyB`, fuse them. Cross-references are not "composition" — they are hidden coupling with combinatorial fallout.
- **Default to stateless policies.** Stateful policies impose construction ordering, break EBCO, and make copy/move of the host alias or duplicate policy state in surprising ways.
- **Empty tag types often beat one-function "policy classes".** `struct CheckedAccess {};` + `if constexpr (std::is_same_v<P, CheckedAccess>)` is usually clearer than a policy with a `static constexpr bool`.
- **Name the contract, not the implementation.** `ThreadingModel`, not `MutexPolicy`. The concept lives longer than any particular lock type.

**Before refactoring an inheritance-heavy class into policies, READ `references/decomposing-policies.md`** (procedure + worked example).

## Modern C++ pragmatics (what to write in 2020+ instead of classic Loki)

| Classic Loki (2001)          | What to write today                       | Why                                               |
| ---------------------------- | ----------------------------------------- | ------------------------------------------------- |
| Recursive `TypeList<...>`    | Parameter packs + fold expressions, `std::tuple` for storage | Linear error messages, O(1) access, no recursion depth |
| `Int2Type<N>` tag dispatch   | `if constexpr`                            | One function body, dead branches elided, no extra overloads |
| `Select<B, T, F>::Result`    | `std::conditional_t` or `if constexpr`    | Ships in the standard, instant diagnostics        |
| Host *inherits* policies for EBCO | `[[no_unique_address]]` data member   | Works when the policy is `final`; no member-name collisions; no access-specifier games |
| `boost::enable_if` / SFINAE traits | `requires` clause + named `concept`  | Error message names the violated requirement instead of dumping a substitution cascade |
| Hand-rolled `has_member<>` | `requires { t.foo(); }` expression at the call site | No boilerplate; constraint lives with the code that needs it |
| `ScatterHierarchy<Typelist>` | Variadic CRTP `: Base<Ts>...`            | Direct, readable, no typelist machinery           |
| Manual `TypeAt<I, List>`     | `std::tuple_element_t<I, std::tuple<Ts...>>` | Built-in, O(1) compile-time lookup              |

For exact, runnable C++20/23 code for each row, **READ `references/modern-replacements.md`**.

**Portability footgun — fix this or ship broken code:** `[[no_unique_address]]` is silently ignored by MSVC (even in `/std:c++20`), which requires `[[msvc::no_unique_address]]`. A cross-platform class needs both spellings behind a macro; getting it wrong produces different `sizeof` between GCC/Clang and MSVC with identical source. Lay it out once:

```cpp
#if defined(_MSC_VER) && !defined(__clang__)
#  define POLICY_MEMBER [[msvc::no_unique_address]]
#else
#  define POLICY_MEMBER [[no_unique_address]]
#endif
```

## NEVER list (expert anti-patterns with receipts)

- **NEVER expose a policy template parameter on a vocabulary type** (optional, variant, result, handle, string, span). It is seductive because "users might want it someday", but it fractures every API that tries to pass the type across boundaries. Consequence: `basic_variant`-style classes become unusable in public APIs and users work around them with conversions. Instead: pick one implementation, provide 2–3 *named* variants (`optional<T>` vs `optional_ref<T>`), or auto-select via a trait.

- **NEVER inherit from a user-supplied policy without checking `std::is_final_v<P>`.** Seductive because inheritance buys you EBCO. Consequence: the moment a user passes a `final` policy (fully legal for allocators, deleters, comparators), your inheritance-based shrinkage collapses and `sizeof(Foo)` silently grows by a pointer, blowing cache-line-based assumptions. Instead: use `[[no_unique_address]]` (plus the MSVC spelling) on a data member.

- **NEVER let policy A reference policy B's interface.** Seductive because "orchestration". Consequence: the two stop being orthogonal; every combination a user constructs becomes a manual audit. Instead: if two policies must coordinate, they are one policy — fuse them and stop pretending.

- **NEVER give a policy persistent state that the host also touches.** Seductive because it looks like encapsulation. Consequence: construction order becomes a load-bearing invariant, host copy/move ops silently alias or duplicate the state, and reassignment produces Heisenbugs. Instead: make the policy stateless, or make the policy the *sole* owner of that state and route host access through its member functions.

- **NEVER add a policy parameter that has exactly one implementation today.** Seductive because "we might add another". Consequence: every TU pays full instantiation cost, diagnostics double in length, and removing the parameter later is an API break. Instead: hardcode now, add the parameter the day the second implementation materializes.

- **NEVER use SFINAE traits (`enable_if_t`, `void_t` probing) in new C++20 code.** Seductive because it still works. Consequence: a single typo at a policy call site produces a 200-line substitution cascade that nobody can read, and users cannot tell which requirement they violated. Instead: write a named `concept`. The error becomes `"PolicyX does not satisfy ThreadingModel because 'lock_type' is not a type"`.

- **NEVER put a virtual destructor on a policy-composed value type like `SmartPtr<T, Policies...>`.** Seductive because "it's safer." Consequence: you have just added a vtable to a type whose entire purpose is zero overhead, and invited users to `delete` through a base pointer — the exact failure mode policy-based design exists to avoid. Loki's `SmartPtr` destructor is **deliberately** non-virtual; the library documents that polymorphic destruction of a smart pointer is a design error at a higher level. Instead: design value types as non-polymorphic. If users need polymorphism, they hold one by value in a type-erased wrapper.

- **NEVER assume EBCO shrinks your class just because a policy is empty.** Two empty bases of the **same type** are required by the standard to occupy distinct addresses (the unique-identity rule), and the same applies to two `[[no_unique_address]]` members of the same type. Stack two `SingleThreaded` bases and you pay padding. Instead: if you need multiple "empty" roles of the same underlying type, give each role a distinct tag (`struct LockTag : SingleThreaded {}; struct CheckTag : SingleThreaded {};`) so the types differ.

- **NEVER make a policy do two things.** Seductive because "fewer template parameters". Consequence: the policy is no longer substitutable — to change one aspect users must reimplement the other. Instead: split until each policy answers exactly one question. If that gives you >3 policies, something else is wrong (see decomposition rules).

## Decision tree: "should this be a policy?"

```
Decision known at compile time for a given build?
├── no  → Not a policy. Runtime configuration or virtual dispatch.
└── yes → ≥2 implementations exist in this codebase today?
         ├── no  → Not a policy yet. Hardcode. Revisit the day #2 arrives.
         └── yes → Is the host a vocabulary type (optional / variant / handle / span / string)?
                  ├── yes → Not a policy. Provide 2+ named classes OR auto-select via a trait.
                  └── no  → Does the host already pay for heap + indirect call on the hot path?
                           ├── yes → Type-erase. Public surface must not carry the policy.
                           └── no  → Policy is justified. Apply the decomposition rules above.
```

## Failure-to-diagnosis table

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| `error: ambiguous call to ...` across policies | Two policies define a member with the same name | Rename, or wrap each policy in a disambiguating subobject |
| `sizeof(Foo)` suddenly grew after a user upgrade | A user passed a `final` policy; EBCO inheritance collapsed | Switch to `[[no_unique_address]]` member storage |
| `sizeof(Foo)` differs GCC vs MSVC with identical source | Missing `[[msvc::no_unique_address]]` spelling | Apply the portability macro above |
| Debug binary size exploded after adding a policy | Combinatorial instantiation | Count actual combinations used; consider `extern template` for the top N |
| Compile times doubled after a refactor | Deep recursive templates (typelists, nested SFINAE) | Replace with parameter packs + `if constexpr` + concepts |
| Error message is a 200-line substitution cascade | SFINAE | Convert the constraint to a named `concept` with a human-readable name |
| `constexpr` policy branch is compiled into the binary anyway | Dependent type bug — the branch isn't fully discarded | Put the branch behind `if constexpr`, not `if`, and ensure the discarded branch does not reference names requiring the other instantiation |

## Final gut-check (apply before committing)

If you can delete a template parameter and keep all your tests green, **delete it**. If a second template parameter only has a default value and no user has ever overridden it, delete it. If you cannot explain in one sentence why each remaining parameter earns its cost in API fragmentation, you have more work to do.
