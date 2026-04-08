---
name: griesemer-precise-go
description: "Design and review Go code in the Robert Griesemer style: spec-first semantics, tight constraints, named-type preservation, and compatibility-safe API evolution. Use when writing or reviewing generics, interfaces, method sets, comparison logic, zero-value behavior, or exported APIs where exact language rules matter. Triggers: Go generics, constraints, comparable, type sets, underlying type, method sets, pointer receivers, zero value, public API compatibility."
tags: go, generics, constraints, comparable, interfaces, method-sets, api-design, compatibility
---

# Griesemer Precise Go

This skill is for Go work where the hard part is semantic precision, not style polish.

This skill is self-contained. Do NOT load other Go style skills for pure type-system or public-API work unless the user explicitly asks for concurrency tuning, CLI UX, or low-level performance work after semantics are settled.

## Core stance

Griesemer-style Go starts from the spec, not from folklore, compiler accidents, or what "usually works on gc".

Every abstraction must survive these questions:
- What exact operations does the type set permit?
- Which part of the API is constrained, and which part stays maximally reusable?
- Does this preserve named types, zero values, and future compatibility?
- If a reader only knows the spec, will they predict the behavior correctly?

On Go 1.25+, reason about generic operands directly from the type set rules. Do not keep using "core type" as a design crutch. The right question is "is this operation valid for every type in the set?"

## Before you write code, ask yourself

Before introducing a type parameter, ask yourself:
- Does the algorithm truly ignore the concrete type except for a few operations? If not, use an interface or concrete code.
- Am I modeling a reusable family of types, or am I just avoiding two small copies of code? If it is the latter, duplication is often clearer.
- Will the signature preserve user-defined named types, or silently erase them to a built-in type?

Before constraining with `comparable`, ask yourself:
- Do I need equality everywhere, or only in one map-backed implementation?
- Could another implementation work for non-comparable types if I left the interface unconstrained?
- Am I banning valid future use cases just to make today's implementation convenient?

Before using pointer receivers in a generic API, ask yourself:
- Does the function need to allocate a fresh zero value itself, or can the caller pass a ready instance?
- Am I forcing an extra type parameter only because the API shape is wrong?
- Would a plain interface value make ownership and initialization clearer?

Before exporting an API, ask yourself:
- If I need one more option later, can I add it without changing the function signature?
- Am I returning an interface that I may want to grow later, when a concrete type would age better?
- Do I want values of this struct to remain comparable forever, or should I block comparison now with a zero-size non-comparable sentinel like `[0]func()`?

## Decision rules that matter

### Interface vs generic vs comparator

Use an interface when the algorithm only calls behavior already expressed as methods. If all you do is call `Read`, `Write`, `Compare`, or `All`, a type parameter usually adds noise.

Use a type parameter when the implementation is structurally identical across element types and the value should be stored directly, not boxed behind an interface.

For ordered containers, pick the narrowest contract that matches the job:
- `cmp.Ordered` is right when numbers and strings are the whole domain.
- A comparator function is the most general surface and keeps the shared implementation unconstrained.
- A self-referential generic interface such as `Comparer[E]` is the zero-value-friendly variant when element types already own comparison logic.

If you need all three, keep the shared core unconstrained and pass the comparator as a parameter. Wrap it with ordered and method-based entry points. Passing a comparator argument is easier for the compiler to analyze than storing a function in a struct field.

### Fast chooser

| If the real need is... | Prefer | Because |
| --- | --- | --- |
| Many implementations with different internal trade-offs | `interface{ ... }` with `any` | The abstraction should not pre-commit all implementations to one constraint set. |
| One algorithm reused over many concrete element types | A generic type or function | The code shape is truly identical and values stay unboxed. |
| Ordering only for numbers and strings | `cmp.Ordered` | It is the smallest honest contract. |
| Ordering for arbitrary domain types | Comparator function first | It is the most general surface and keeps the shared core reusable. |
| Zero-value container plus type-owned ordering | `Comparer[E]` wrapper over the shared core | You recover zero-value ergonomics without constraining the common implementation. |
| Pointer-receiver methods plus internal allocation | Caller-supplied interface value first; `PtrTo...` only if allocation is essential | Extra type parameters are a cost, not a badge of sophistication. |

### Constraint placement

Push strong constraints to the last responsible layer.

Good:
- `type Set[E any] interface { ... }`
- `type OrderedSet[E interface { comparable; Comparer[E] }] struct { ... }`

Bad:
- `type Set[E comparable] interface { ... }`

The interface should preserve implementation freedom. The concrete type or function that truly needs map keys should pay the `comparable` cost.

### Preserve named types intentionally

If a helper accepts a slice, map, or channel-like shape and should return the same named type it received, deconstruct the type:

```go
func Clone[S ~[]E, E any](s S) S
func CopyMap[M ~map[K]V, K comparable, V any](m M) M
```

Using `[]E` or `map[K]V` directly is a semantic choice: it erases named types and their methods on return.

### Pointer-receiver generic escape hatch

If a generic function must allocate a value internally and then call pointer-receiver methods, the standard escape hatch is:

```go
type PtrToSet[S, E any] interface {
	*S
	Set[E]
}
```

Then use `new(S)` and convert to the pointer type parameter. This works, and the trailing pointer type can usually be inferred by callers.

If this makes the public signature look clever, stop and redesign. In practice, accepting a concrete `Set[E]` value from the caller is often simpler and more flexible.

### Fallbacks when the first design gets too clever

- If the constraint literal grows past a short screenful, name it or push it down to the concrete type that truly needs it.
- If you need two extra type parameters only to connect value and pointer method sets, try an interface value API before shipping the generic signature.
- If you cannot preserve named types without making the API unreadable, decide explicitly whether preserving the outer type matters. Erasing it is sometimes correct, but it must be a deliberate semantic choice.
- If the only argument for a generic abstraction is "the compiler will probably optimize it", keep the concrete version and benchmark first.

## Non-obvious edge cases

- `any` supports `==` syntactically as an interface type, but it is not a safe stand-in for `comparable` in type arguments. Interface comparison can still panic at run time when the dynamic value is uncomparable.
- Method sets are asymmetric for a reason: an interface holding `T` cannot safely fabricate `*T`. If a method mutates state, a value receiver in an interface would silently mutate a copy.
- Map iteration order is unspecified, not "random enough". Never treat it as a shuffle primitive or a stable bias source.
- Generic operations are the intersection of what every member of the type set supports. If one member breaks the operation, the operation is invalid no matter how natural it feels for the others.
- If you must support code before Go 1.22, slice-delete helpers need an explicit tail-clearing audit for pointer-bearing elements. On Go 1.22+, `slices.Delete`, `Compact`, `DeleteFunc`, `CompactFunc`, and `Replace` clear obsolete elements, but ignoring the returned slice is still a logic bug.

## NEVER rules

- NEVER put `comparable` on a generic interface just because the first implementation uses a map, because that convenience quietly forbids tree-, list-, or comparator-based implementations for non-comparable types. Instead constrain only the concrete type or function that actually needs map keys.
- NEVER write `func F[E any]([]E) []E` for helpers meant to round-trip named slice types, because the short signature is seductive and compiles, but it strips methods and aliases on return. Instead preserve the outer type with `S ~[]E`.
- NEVER design an exported API around hypothetical generic methods, because the symmetry looks attractive but Go does not plan to support methods with type parameters. Instead use top-level generic functions or put the type parameters on the receiver type.
- NEVER assume Go generics guarantee C++-style per-type specialization, because gc commonly shares one instantiation per shape and future compilers may make different trade-offs. Instead benchmark the real hot path and add concrete fast paths only when measurement proves they matter.
- NEVER keep a comparator as a long-lived struct field merely to simplify recursion, because it destroys zero-value usability and makes inlining harder. Instead pass the comparator through the unconstrained core and offer thin wrappers for ergonomic surfaces.
- NEVER change an exported function signature to add options, context, or variadics "compatibly", because call sites may still compile while function values, interface satisfaction, and assignments break. Instead add a sibling function or an options struct with meaningful zero values.
- NEVER return an exported interface unless you want third parties implementing it forever, because every future method becomes a compatibility event. Instead return a concrete type unless outside implementation is an explicit goal.

## Practical review procedure

When reviewing or writing code in this style, run this sequence:

1. List every operation performed on each type parameter.
2. For each operation, prove it is valid for every type in the constraint's type set.
3. Check whether the signature preserves named types or erases them.
4. Check whether the zero value is useful, or whether a function field / constructor requirement made it unusable.
5. Check whether `comparable` or pointer-receiver constraints were pushed higher than necessary.
6. For exported APIs, ask how you would add one new option, one new method, and one new field without breaking users.

If any answer depends on "the compiler probably optimizes this" or "users probably will not do that", the API is not precise enough yet.
