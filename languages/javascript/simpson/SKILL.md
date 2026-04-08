---
name: simpson-you-dont-know-js
description: Deep JavaScript semantics in the style of Kyle Simpson. Use when debugging or refactoring bugs involving `this`, scope, closures, prototypes, `class`/`super`, coercion, promises, microtasks, sparse arrays, or host-vs-language confusion. Triggers: "why does JS do this", "lost this", "closure bug", "prototype chain", "safe ==", "async ordering", "microtask", "class field", "super", "holey array".
---

# Kyle Simpson: Mechanics First

JavaScript is rarely "weird" at random. Most failures reduce to one of six systems: lexical scope, call-site binding, prototype lookup, coercion, job-queue scheduling, or host-environment behavior. Do not patch symptoms until you can name the system.

## Use this style for

- Semantic debugging, library/runtime code, API boundaries, and code review where the mechanism must be explainable, not merely tolerated.
- Refactors that should remove cargo-cult `bind`, accidental `class` misuse, unsafe equality, sparse arrays, or async ordering bugs.

## Loading Discipline

- This skill is intentionally self-contained. Do not load framework docs until you reduce the issue to a language mechanism.
- Only leave this skill for MDN/spec/V8 material when the code touches `super`, class fields, microtasks, or engine-sensitive array/object shapes.
- Do not load performance material for cold code. Readability wins unless the path is measurably hot.

## Before You Change Code, Ask Yourself

- Is this a lexical-scope problem, a call-site problem, a prototype problem, a coercion problem, a scheduling problem, or a host-API problem?
- Which values are own properties versus inherited ones right now?
- Did this change create a new function identity, a new promise boundary, or a new object shape?
- Am I relying on transpiler-era behavior that differs from current JavaScript semantics?

## Semantic Triage

1. Strip the bug down to the smallest reproduction that still fails.
2. Log the receiver, ownership, and queue boundary: `this`, `Object.hasOwn(...)`, `Object.getPrototypeOf(...)`, and every `await`/`.then(...)`/`queueMicrotask(...)`.
3. Remove the framework and host calls. If the bug disappears, it was never "just JavaScript."
4. Decide whether the fix must preserve function identity, prototype sharing, field initialization order, or microtask ordering.
5. Only after the mechanism is explicit should you refactor style or API shape.

## Decision Heuristics

- Wrong `this`: inspect the call-site, not the definition site. If the function should stay late-bound, keep a normal method. If the callback must inherit outer `this`, use an arrow only at that boundary. If teardown/removal matters, store a stable function reference once.
- `class` or `super` weirdness: check whether a class field replaced constructor assignment, whether a method using `super` was extracted, and whether initializer order now matters more than inheritance structure.
- Equality weirdness: reserve `x == null` for deliberate nullish collapse. For everything else, normalize the types before comparing.
- Map-like object weirdness: if keys are uncontrolled strings, use `Object.create(null)` or `Map`, then pair it with `Object.hasOwn(...)`.
- Async ordering weirdness: mark every `await` as a reentrancy point. If two branches must behave consistently, put both on the same queue or await once at a higher boundary.
- Array weirdness: look for holes, sparse writes, and indexed descriptors before looking for algorithm bugs.

## Mechanics That Matter

- `this` is call-site state, not function identity. `self = this`, `bind(this)`, and arrow functions solve different problems; treating them as equivalent is how APIs become impossible to reason about.
- Inline `bind(this)` or `() => ...` in `addEventListener`, subscription APIs, or caches creates identity drift. Removal fails because teardown requires the exact same function object, not equivalent source text.
- Prototype methods are shared and patchable; arrow class fields are own properties created per instance. That means one closure per instance, different equality identity, weaker subclass override behavior, and harder spying/monkey-patching.
- Public class fields use `[[DefineOwnProperty]]`. In derived classes they are applied after `super()`, and they do not invoke setters on the base prototype. Old Babel/TypeScript output often behaved like constructor assignment, so migrations can silently change behavior.
- Computed public field names are evaluated once at class definition time, not per instance. A field like `[Math.random()] = 1` chooses one key for the class, not a new key for each object.
- `super` is tied to a method's home object, not rebound the way `this` is. Reusing or reassigning a method that uses `super` can keep `this` working while `super` still points at the original prototype chain.
- `==` is a disciplined tool only when the normalization contract is explicit. `x == null` is honest. The moment booleans, arrays, or `""`/`0` can enter the comparison, review quality collapses because the algorithm is now doing hidden `ToPrimitive` and `ToNumber` work.
- `parseInt(...)` is a parser, not a coercion primitive. Use it for `"42px"` or explicit radix rules. Use `Number(x)` or unary `+x` when the contract is "this should already be numeric-ish."
- `Object.create(null)` is the honest object-as-map when inherited keys or prototype pollution matter, but it also removes `hasOwnProperty`, `toString`, and other `Object.prototype` conveniences. Plan for that upfront with `Object.hasOwn(...)`.
- `delete arr[i]` does not remove an element; it creates a hole. In V8, one hole is enough to move an array off packed fast paths, and prototype lookup can satisfy that missing index later. Sparse writes like `arr[9999] = "x"` push arrays toward dictionary elements.
- Strings are immutable and not arrays. `split("").reverse().join("")` is a lossy Unicode hack that breaks astral symbols and grapheme clusters; use Unicode-aware tooling when text fidelity matters.
- `await` is not only syntax sugar; it is a scheduling boundary. Other microtasks can run between lines that look adjacent. Historically an `await` could cost extra promises and multiple microtask hops; modern engines optimized the common case, but the reentrancy point remains.
- `queueMicrotask(...)` and `Promise.resolve().then(...)` use the same queue, but `queueMicrotask(...)` avoids promise allocation and reports thrown errors as ordinary exceptions instead of rejected promises. Recursive microtasks can starve rendering and event processing.
- Promise rejection handling has a timing window. A `.catch(...)` attached in a later `setTimeout(...)` is often too late for unhandled-rejection reporting. Attach rejection handling in the same chain or the same turn.
- DOM events, timers, fetch, and loaders are host behavior layered on top of JS. Reduce a bug to pure language semantics before blaming the language.

## NEVER

- NEVER convert prototype methods to arrow class fields just to avoid `bind`, because the convenience hides per-instance closures, breaks stable listener identity, and weakens override/patch behavior. Instead keep a prototype method and bind once at the boundary that truly needs it.
- NEVER replace constructor assignment with a class field when setters, proxies, or initialization order matter, because standard field semantics use `[[DefineOwnProperty]]` and skip inherited setters. Instead preserve constructor assignment unless bypassing the setter is intentional.
- NEVER use `obj.hasOwnProperty(key)` on untrusted or map-like objects because shadowed methods lie and null-prototype objects throw. Instead use `Object.hasOwn(obj, key)`.
- NEVER use `delete arr[i]` because it creates holes rather than removing elements, which changes lookup behavior and engine representation. Instead use `splice`, a sentinel value, or a real map keyed by index.
- NEVER use `==` as a blanket "be flexible" operator because booleans, arrays, and empty strings trigger coercion paths humans review badly. Instead normalize types first, or reserve `x == null` for explicit nullish collapse.
- NEVER sprinkle `await` through hot loops or state machines because each `await` creates an interleaving point where invariants can be observed half-updated. Instead batch independent work with `Promise.all(...)` or move the await to a higher boundary.
- NEVER benchmark `+x` versus `Number(x)` or `++i` versus `i++` on cold code because engines aggressively rewrite trivial syntax and bad test setup dominates the result. Instead measure real hot paths and optimize data shape, holes, and scheduling boundaries first.
- NEVER blame JavaScript for DOM, timer, or fetch oddities before isolating a language-only reproduction, because host objects often look like normal objects while obeying very different lifecycle rules. Instead reduce first, then reintroduce the host API.

## Constraint Level

- High freedom: API design and refactors. Favor explicit data flow, named functions, behavior delegation, and code that teaches the mechanism.
- Low freedom: semantic bug fixes involving `super`, class fields, sparse arrays, or microtask ordering. Do not "clean up" until you can state the exact mechanism being preserved.
- Medium freedom: performance work. Improve shapes, hole avoidance, and queue boundaries only with measurements from the actual hot path.

## If the First Fix Fails

- Re-check whether a transpiler changed class fields into constructor assignments or vice versa.
- Replace host APIs with tiny stand-ins. If the bug disappears, you were debugging the host, not the language.
- Turn hidden work into explicit work: normalize values before compare, pass the receiver explicitly, materialize the queue boundary, or swap object-as-map code for `Map`.
- If you still cannot explain the behavior in one sentence, the code is not understood yet and should not be generalized.
