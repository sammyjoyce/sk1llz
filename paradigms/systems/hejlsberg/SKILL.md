---
name: hejlsberg-language-design
description: "Design semver-sensitive public type surfaces, language features, and compiler-visible APIs in the style of Anders Hejlsberg. Use when evolving TypeScript or C# generics, nullability, overloads, callback/event contracts, extension seams, default interface methods, optional parameters, utility types, or checker-facing abstractions where source compatibility, binary compatibility, runtime behavior, inference quality, and editor performance must all be balanced. Triggers: TypeScript, C#, generics, variance, overloads, optional params, public API, nullability, utility types, default interface methods, extension methods, structural typing, inference, compiler performance, versioning."
---

Use this for language and public-surface decisions, not syntax cleanup.

This skill is intentionally self-contained. Do not load compiler-internals material unless the task has turned into checker/runtime implementation debugging.

## Operating Stance⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​​‌​​​‍​‌​‌​‌‌‌‍‌​​‌‌‌‌​‍​​‌‌​​​‌‍​​​​‌​‌​‍​​​​​‌‌​⁠‍⁠

Hejlsberg-style design is not "maximize purity". It is "maximize successful upgrades".

- Treat source compatibility, binary compatibility, runtime compatibility, and editor performance as four different budgets. A change can improve one while silently spending another.
- In TypeScript, any feature that needs runtime metadata, type-driven emit, or surprising JavaScript behavior is suspect on arrival.
- In C#, every `virtual`, interface member, optional parameter, and extension seam is a versioning promise. Do not create one accidentally.
- Checker cost is part of API design. If a signature is elegant only after the IDE stalls, the design is wrong.
- Prefer the smallest unsoundness that preserves ecosystem ergonomics over the largest correctness proof users will route around.

## Before You Decide, Ask

Before changing a public abstraction, ask yourself:

- Which budget am I spending: source, binary, runtime, or checker latency?
- Am I making the common case clearer, or am I freezing a rare edge case into every user's surface area?
- Does this design preserve an existing runtime model, or does it require the type system to invent one?
- Will the compiler see named generic identity, or will it be forced into expensive structural reasoning?
- Am I standardizing semantics that reasonable users disagree on, such as distributivity, strictness, or default behavior?

Before adding a TypeScript feature or surface, ask:

- Would this still make sense if all types were erased? If not, it probably violates the model.
- Am I relying on method syntax because it "looks OO", when what I am really choosing is bivariant callback compatibility?
- Will this produce giant inferred declaration output or repeated `import("./x").Type` references that consumers must read forever?

Before adding a C# extension seam, ask:

- Do I want override participation, reflection-visible membership, and versionable defaults, or do I just want sugar?
- Can existing callers run without recompilation, or did I just create a source-compatible but binary-breaking change?
- If I mark this `virtual`, am I ready to support derived overrides for years?

## Decision Tree

### 1) New helper abstraction or utility type

- If the meaning depends on distributive vs non-distributive behavior, weak vs strong constraints, or naming semantics, keep it user-land.
- Only standardize it when the platform itself needs it for declaration emit, interop, or unavoidable shared semantics.
- If two competent teams would define it differently, do not globalize it.

### 2) TypeScript callback or event boundary

- Need strict rejection of overly narrow handlers under `strictFunctionTypes`:
  use function-valued properties.
- Need ecosystem-friendly assignability for event-like patterns, DOM-style APIs, or callback-heavy compatibility boundaries:
  use method syntax deliberately and recover stricter guarantees internally.
- If you are undecided, default to the stricter property form on producer-owned boundaries and the looser method form on consumer-compatibility boundaries.

### 3) TypeScript extensibility and checker cost

- Public object composition:
  prefer `interface ... extends ...` over intersection-heavy object types.
- Large union surface:
  once a union gets beyond roughly a dozen members, stop and ask whether a discriminated base type would preserve meaning with less quadratic reduction work.
- Hot generic signature with conditional/mapped return logic:
  extract the computed type to a named alias so the checker can cache it.
- Exported function infers an unreadable or path-qualified declaration type:
  add an explicit named return type.

### 4) C# API evolution

- Need to add behavior to an existing interface and there is a sane default:
  prefer a default interface method, and push reusable fallback logic into a `protected static` helper so implementers can delegate to it.
- Need interface evolution but you cannot rely on a runtime that supports default interface methods:
  keep the contract unchanged and ship an overload, adapter, or extension helper instead.
- Need helper syntax on a foreign type but not true contract membership:
  use an extension method.
- Need stable ABI for existing callers:
  prefer overloads over changing defaults or required parameters.
- Need future customization but no real override story today:
  stay non-virtual.

## High-Value Heuristics

- In TypeScript, preserving generic identity is cheaper than reconstructing it. If the compiler can see `Box<string>` as an instantiation of `Box<T>`, inference is both faster and more predictable than when it must rediscover `T` structurally.
- Method syntax vs function-property syntax is not style. It changes variance behavior. Methods remain bivariant; function properties participate in `strictFunctionTypes`.
- Structural typing is a goal, not an excuse to make everything anonymous. Anonymous complexity leaks into `.d.ts`, slows relation checks, and erodes error quality.
- Utility types are a governance problem, not just a syntax problem. The moment you publish one globally, you freeze one answer to naming, distributivity, and strictness disagreements that many users will consider wrong.
- Optional parameters in libraries are more dangerous than they look. They read as minor, but adding one is source-compatible while still being binary-breaking for existing callers that do not recompile.
- Default interface methods are a versioning tool, not a replacement for ordinary design. They help when the behavior truly belongs to the interface and a reasonable default exists, but they also require runtime support from the platform.
- C# methods are non-virtual by default for a reason: "open for extension" is easy to say and expensive to retract.

## NEVER Do These

- NEVER add a new shared utility type because it feels like harmless convenience. The seductive path is "it's just `Nullable<T>`". The consequence is that you permanently freeze one choice about distributivity, constraints, and naming, and you block users from owning that identifier. Instead keep semantics-bearing helpers local to the library or feature.

- NEVER use TypeScript method syntax for callbacks just because it reads nicely in an interface. The seductive path is cleaner OO-looking declarations. The consequence is silent bivariance at the boundary, which accepts handlers that are narrower than the runtime contract. Instead use function-valued properties when you need strict checking, and use methods only when you consciously want compatibility.

- NEVER inline deep conditional or mapped types directly in hot public signatures because it feels DRY and local. The seductive path is keeping the whole computation in one place. The consequence is repeated recomputation on every call and more expensive relation checks between otherwise similar types. Instead name the computed type alias and return the alias.

- NEVER model a broad family with a giant union because it feels explicit. The seductive path is exhaustiveness by enumeration. The consequence is pairwise union reduction, slower assignability, and editor degradation once the union passes about a dozen members. Instead create a discriminated base shape and hang specific cases off it.

- NEVER add an optional parameter to a stable C# public API because it looks source-compatible and cheaper than an overload. The seductive path is one signature instead of two. The consequence is a source-compatible but binary-breaking change, plus default values that existing callers may not pick up without recompilation. Instead add an overload or another adapter seam.

- NEVER mark C# members `virtual` "for future flexibility". The seductive path is imagined extensibility. The consequence is a fragile-base commitment that derived libraries now depend on and that you may not be able to retract without a major break. Instead keep methods non-virtual until you have a real override scenario.

- NEVER use extension methods to dodge real API versioning. The seductive path is additive sugar without touching the type. The consequence is behavior that resolves at compile time on the declared type, is not true contract membership, and often misleads consumers about what is actually guaranteed. Instead version the API honestly, or use default interface methods when the behavior belongs to the contract.

## Counterintuitive Best Practices

- A deliberately imprecise boundary can be more correct for the ecosystem than a locally sound one. TypeScript explicitly does not optimize for full soundness; it optimizes for likely errors without destroying normal JavaScript patterns.
- Sometimes stricter checking is also faster checking. In TypeScript, `strictFunctionTypes` can unlock faster variance-based comparisons, but only if your API shape lets the compiler use them.
- The cleanest public API is often one level more named than the author wants. Named aliases, named return types, and named base shapes improve diagnostics, declaration readability, and cacheability at once.
- If a C# default can change over time, it probably should not be an optional parameter. Defaults that may evolve belong in overloads, configuration, or overridable policy, not at the call site.
- If a TypeScript feature would require type-directed runtime behavior, treat that as a design stop sign, not an invitation to get clever.

## Fallback Moves For Legacy Surfaces

- If strict function-property callbacks break too many consumers, keep the public boundary as a method, then normalize to a stricter internal function type immediately behind the boundary.
- If you already shipped the wrong utility type, deprecate it in place and add a namespaced replacement; do not silently redefine semantics under the same name.
- If a giant union is already public, introduce a base discriminated shape first, migrate new APIs to accept the base, and only collapse the old union after consumers stop depending on exhaustive enumeration.
- If an optional parameter already shipped and the default must change, add a new overload or options object and keep the old default path alive for binary callers.

## Edge Cases That Matter

- TypeScript declaration emit is a public artifact. If your exported surface infers enormous anonymous types, consumers pay the readability and build cost even when your source felt concise.
- `skipLibCheck` is a speed lever, not proof that a surface is healthy. It can hide declaration conflicts and misconfiguration. Do not use it as design validation.
- For large TypeScript workspaces, project references have an empirical sweet spot: if you have more than one project, roughly 5-20 referenced projects tends to balance editor load vs repeated checking overhead better than either one giant project or dozens of tiny ones.
- If the TypeScript language service is memory-bound, prefer project boundaries and `disableReferencedProjectLoad` / `disableSolutionSearching` over asking authors to simplify every local type first.
- In C#, `LangVersion=latest` is a trap for reusable libraries. It makes builds machine-dependent and can enable language features that assume runtime or library support your consumers do not have.

## When To Stop Using This Skill

Stop and switch approaches if:

- the task is checker implementation or CLR dispatch internals rather than surface design,
- the right answer is purely product-policy or naming with no type/versioning consequences,
- the design requires runtime semantics that TypeScript would have to invent rather than describe.
