---
name: meyers-effective-cpp
description: Apply Scott Meyers-style itemized review heuristics to C++ code where overload resolution, initialization syntax, special-member generation, perfect forwarding, smart-pointer ownership, lambda capture, and container behavior hide non-obvious failure modes. Use when designing or reviewing constructors, sinks, factories, pImpl types, move/copy operations, forwarding helpers, smart-pointer APIs, or performance-sensitive STL code. Triggers: meyers, effective-cpp, forwarding-reference, universal-reference, brace-init, initializer_list, noexcept-move, pimpl, make_shared, shared_ptr, unique_ptr, lambda-capture, std-async.
---

# Meyers Effective C++

Use this skill as a semantic trap detector, not a style checklist. Meyers' method is to find the place where C++ silently generates behavior, prefers the overload you did not expect, or turns a performance claim into folklore.

## Mandatory loading

- Before changing forwarding constructors, `T&&` APIs, brace-init call sites, or pImpl special members, READ `references.md`.
- Before touching `std::make_shared`/`std::make_unique`, lambda captures that may escape, or `std::async` code, READ `references.md`.
- Do NOT load `references.md` for routine `const`, `override`, or RAII cleanup; this file is enough.

## Operating stance

- Treat every public API as an overload-resolution problem first and an implementation problem second.
- Assume move is not present, not cheap, and not used until you verify the exact type and context.
- When old Meyers advice and modern folklore conflict, keep his method: ask what the compiler generates, what overload resolution prefers, what exception guarantees containers depend on, and what ownership the interface actually communicates.
- Prefer guidance you can defend in review with a concrete failure mode, not slogans.

## Freedom calibration

- Use judgment on interface shape, naming, and whether a helper should be a member, free function, or factory.
- Be rigid on forwarding references, brace initialization, `noexcept` on moves, pImpl completeness, and escaping captures; these fail by language rule, not taste.

## Before you touch an interface, ask

- Am I expressing observation, shared ownership, a true sink, or perfect forwarding? One signature rarely communicates all four correctly.
- Could this overload accidentally bind to the class itself, a derived type, `std::initializer_list`, or a braced temporary?
- If move becomes copy here, does correctness or big-O change?
- If this code is used in a container, does `noexcept` change whether reallocation moves or copies?
- If this lambda or async object escapes the current scope, what exactly got captured: a value snapshot, a `this` pointer, or a future whose destructor may block?
- If I hide data behind pImpl, am I trading compile-time wins for runtime indirection, extra memory, and null moved-from states I now must police?

## Decision rules

- If the function merely observes an object, prefer `const T&`. Do not pass `std::shared_ptr` by value unless taking part in refcount ownership is part of the contract.
- If the parameter is a true sink of a move-only type, default to `T&&`, not by-value. By-value adds another move, moves at the call site, and can worsen sequencing and strong-guarantee behavior.
- If the parameter is copyable and the function always keeps an owned copy, by-value is acceptable only when move is verified cheap and the call path benefits from collapsing `const&`/`&&` overloads into one body.
- If you need perfect forwarding plus special cases, do not overload directly on universal references. Dispatch through helper class templates or constrained overloads after stripping cv/ref qualifiers.
- If a call may hit an `initializer_list` overload or `auto` deduction is involved, default to `()` unless you explicitly want `initializer_list` semantics or narrowing diagnostics.
- If perfect forwarding must accept braced initializers, add a dedicated `std::initializer_list` overload. Forwarding alone is not enough.
- If you want build isolation through pImpl, define destructor and relevant special members out-of-line where the impl is complete. Then decide explicitly whether moved-from objects may be null or must remain usable.
- If async work must truly run asynchronously, spell `std::launch::async`. If you cannot tolerate an implicit join on destruction, do not rely on a `std::async` future's lifetime rules to "probably be fine."

## High-value heuristics

- Small-string optimization can make moving short `std::string`s roughly as expensive as copying them. Scott observed VC11 small-string capacities up to 15 chars; treat "move is cheap" as representation-dependent, not a type trait.
- For lookup-heavy tiny sets, flat storage can beat node-based containers by locality alone; Scott reported unsorted `std::vector` winning for very small sets and hash tables taking over around 20-50 elements. Measure before cargo-culting trees.
- Identical lambdas in two places are different closure types. If the call operator does not inline, you can pay code-size duplication. Hoist the lambda into an `auto` variable when reuse matters.
- Reference-qualified member overloads are all-or-nothing: once one overload of a given name/signature is ref-qualified, sibling overloads must be ref-qualified too.
- A deep-copy pImpl can preserve value semantics cleanly; a shallow move via `unique_ptr` is cheaper but creates null moved-from states that leak into every member function unless you guard them deliberately.

## NEVER do this

- NEVER add an unconstrained forwarding constructor or setter next to copy/move overloads because it is seductive API "unification," but overload resolution may prefer it for non-const lvalues of your own type and effectively hijack copying. Instead exclude self and derived types with `decay_t`-based constraints or use a named factory.
- NEVER use `{}` because "uniform initialization is safer" when overload sets or `auto` are involved, because braces trigger `initializer_list` preference, `auto` has special deduction rules, and narrowing may only produce a warning on some compilers. Instead use `()` by default and reserve `{}` for deliberate aggregate, `initializer_list`, or anti-narrowing intent.
- NEVER assume a move constructor makes code fast because the seductive part is seeing `&&` and `std::move`; the concrete failure is hidden copies or equally expensive moves for types like SSO strings and large fixed arrays. Instead inspect representation, `noexcept`, and the container or context that will consume the type.
- NEVER leave move operations potentially throwing when objects live in standard containers because the seductive part is preserving future flexibility; the concrete consequence is that reallocation may silently copy instead of move via `std::move_if_noexcept`. Instead make moves `noexcept` whenever the implementation can really uphold it.
- NEVER pass a move-only sink by value just to make the signature look simple because the seductive part is a single overload; the concrete consequence is an extra move, earlier resource release, and weaker exception-story options. Instead use `T&&` for true sinks and by-value only after proving the trade is worth it.
- NEVER stack multiple `enable_if` function overloads and expect the "most specific" predicate to win because that feels like class-template partial specialization, but function-template ordering does not rank satisfied predicates that way. The consequence is ambiguity or the wrong overload surviving refactors. Instead normalize the type and dispatch through a helper class template or concepts.
- NEVER use default capture modes in member functions that may outlive the current object because `[=]` feels like "copy everything," but in member functions it captures `this`, not a snapshot of members. The consequence is dangling-object bugs that appear only after scheduling or refactoring. Instead capture the exact values you need or use init capture.
- NEVER force `make_shared` or `make_unique` everywhere because the seductive part is fewer allocations and cleaner syntax; the concrete failure is that make-functions cannot call non-public constructors and may fail outright in clone-to-abstract-base patterns such as `make_shared<Base>(d.clone())`. Instead use direct smart-pointer construction or a factory with the right access.
- NEVER perfect-forward bitfields, `static const` data members, or braced initializers as if all expressions were first-class objects, because the seductive part is "universal" forwarding; the concrete consequence is deduction failure. Instead stage a local of the exact type, add an `initializer_list` overload, or use the unary-`+`/`+0` workaround only when you accept promotion side effects.
- NEVER assume dropping a `std::future` from `std::async` is non-blocking because the seductive part is RAII cleanup; the concrete consequence is an implicit join when the last reference to that shared state is released. Instead `get` or `wait` intentionally or choose a different primitive for fire-and-forget work.

## Fallback playbook

- If forwarding logic needs type-specific behavior, strip cv/ref first, then dispatch to a helper class template. This avoids partial-ordering fights among `enable_if` overloads and catches `const` and `volatile` variants.
- If SFINAE yields unreadable diagnostics, move "can this construct the target?" checks into a `static_assert` in the body or an immediately-invoked lambda in the initializer so call sites get a human message.
- If forwarding a `static const` member or bitfield is the only blocker, prefer a named local of the exact type. Use unary `+` or `+0` only as a last resort, because the promotion can change overload resolution.
- If pImpl value semantics become expensive, decide explicitly: keep deep copy and deep move for predictable post-move behavior, or accept shallow move and document or assert the null moved-from state.
- If async teardown can deadlock or stall, move the state the task needs into the closure rather than capturing references to owners whose destructor may wait on the task.
- If a generic API needs braces at the call site, stop pretending perfect forwarding will save you; add the overload that names the semantic shape you actually support.

## Output style when using this skill

- Give advice as short, testable items with the hidden failure mode attached.
- Favor "choose X because Y breaks under Z" over historical explanation.
- When in doubt, show which overload wins, which special member is generated or suppressed, and what runtime cost or lifetime bug follows.
