---
name: eich-language-fundamentals
description: "Write JavaScript with expert-level control over prototypes, dynamic objects, function binding, coercion, and V8 shape/array behavior. Use when working on hot Node/Chrome paths, serialization-heavy DTO pipelines, prototype-pollution risk, tricky `this` or optional-chaining bugs, or equality/defaulting migrations. Triggers: \"hidden class\", \"shape\", \"deopt\", \"elements kind\", \"prototype pollution\", \"__proto__\", \"Object.assign vs spread\", \"this binding\", \"arrow vs method\", \"optional chaining\", \"nullish coalescing\", \"Object.hasOwn\", \"Map vs Object\", \"coercion\", \"Eich\", \"ECMAScript semantics\"."
tags: javascript, v8, prototypes, coercion, security, performance
---

# Eich - JavaScript Language Fundamentals⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​‌‌‌​​‍‌‌‌‌​‌​​‍‌‌​‌‌​​‌‍​​​​​​‌‌‍​​​​‌​‌​‍‌​​​​‌​​⁠‍⁠

Use this skill when the bug is about JavaScript *semantics* or when V8/Node performance and object layout actually matter. Do **not** cargo-cult engine rules into ordinary feature work; first classify whether you are solving a semantics bug, a security boundary, or a measured hot path.

## Classify the problem before editing

Before changing code, ask yourself:

- **Is this a measured V8 hot path?** If not, prefer the clearest semantics and schema validation over engine folklore. Hidden-class rules are worth paying for only on profiled code or record-heavy serialization paths.
- **Can keys or option bags be influenced by JSON, query params, CLI flags, or third-party merges?** If yes, this is a prototype-pollution problem first, not a style problem.
- **Am I migrating `&&`/`||`, `?.`, `??`, `bind`, or method extraction?** Treat it as a parse/runtime correctness task; these operators have failure modes that look locally harmless but break at compile time or only after refactors.
- **Will this object be serialized thousands of times?** If yes, object shape and key spelling become throughput decisions, not just readability choices.

## High-value mental models

- **Shape path beats final key set.** In V8, two objects share a hidden class only if properties were added in the same order. `const x = {}; if (flag) x.a = 1; x.b = 2;` and `const y = { a: 1, b: 2 }` may look equivalent to you and still fragment ICs for the engine. Normalize once at construction.
- **`delete` and prototype mutation are "slow path forever" signals.** `delete obj.k` pushes named properties toward dictionary mode; `Object.setPrototypeOf` / `obj.__proto__ = ...` invalidates optimized property access far beyond the single line that changed it.
- **Array performance is a one-way lattice.** Sparse writes, `delete arr[i]`, and `new Array(n)` move arrays into holey or dictionary territory; out-of-bounds reads permanently taint that load site for slower prototype-aware checks.
- **DTO shape stability now affects JSON throughput directly.** In V8 13.8+ (Chrome 138+), `JSON.stringify` gets an "express lane" for repeated objects with the same hidden class when keys are enumerable, non-`Symbol`, and don't need escaping. If you emit arrays of records, keep key order identical and prefer `null` for absent optional fields instead of omitting keys on some rows.
- **Null-prototype objects change both security and ergonomics.** `{ __proto__: null }` is a spec-level prototype setter at creation time and is safe; `obj.__proto__ = null` is a deprecated accessor mutation and slow. On a null-prototype object, later `obj.__proto__ = x` creates an own property named `"__proto__"` instead of mutating the prototype.
- **Bound and arrow functions solve different `this` problems.** Arrow functions capture lexical `this`; class-field arrows therefore auto-bind but allocate one closure per instance. Prototype methods share code across instances, but extracted callbacks lose their receiver unless you bind or wrap them. Bound classes still construct, but lose own static properties and cannot be used on the right side of `extends`.
- **Prototype pollution is often a read bug, not only a write bug.** The attack is complete when polluted values are *observed*. `if (opts.isAdmin)` or `fetch(url, opts)` becomes unsafe if missing keys can resolve through a polluted prototype. Define defaults or require `Object.hasOwn`.

## Expert heuristics that save time

- When you need a dynamic dictionary with untrusted keys, choose **`Map` first**. Use a null-prototype object only when an API requires plain-object shape.
- When you must stay with objects, **declare all expected keys eagerly**. This simultaneously hardens reads against pollution and keeps shapes stable for optimized code and repeated serialization.
- For hot numeric storage, choose **`TypedArray`** over "clever" sparse arrays. It prevents mixed-type drift and removes `push`/hole footguns by construction.
- Prefer **`user?.id ?? fallback`** over legacy `user && user.id || fallback`. It fixes both falsy-value loss (`0`, `''`) and the `??`/`&&` parse trap during partial migrations.
- Treat **cross-realm values** as hostile to `instanceof`. If values can come from iframes, workers, or `vm`, use realm-stable checks such as `Array.isArray`, `Buffer.isBuffer`, or `Object.prototype.toString.call(x)`.
- For options objects that survive refactors, **use explicit defaults, not truthy reads**. `const opts = { __proto__: null, method: 'GET', mode: 'cors', ...userOpts }` is safer than branching on missing properties later.

## Decision trees

### Choosing the container

- Compile-time keys, JSON output, no untrusted writes: use a plain object literal.
- Dynamic or user-controlled keys: use `Map`.
- Dynamic keys but downstream API requires an object: use `{ __proto__: null }`, then validate/normalize before passing it on.
- Dense fixed-length numeric data: use a `TypedArray`.
- Arrays of records for API/cache serialization: use a single factory/constructor so every record gets the same keys in the same order.

### Choosing the callable form

- Needs `new`, `prototype`, shared methods, or `extends`: use `function` / class prototype methods.
- Needs callback-safe lexical `this` and instance count is small enough that one closure per instance is acceptable: use an arrow class field.
- Needs a callback with stable `this` but shared prototype methods: keep a normal method and bind/wrap at the boundary where it is passed away.
- Needs constructor currying: use a wrapper function or subclass, not `Class.bind(...)`.

### Debugging "JS got weird"

- `wrong map`, shape mismatch, or repeated deopts in Node/V8 traces: audit constructor/factory order and remove conditional property adds.
- Array code got slower after "harmless" refactor: look for `new Array(n)`, sparse writes, `delete`, `-0`, `NaN`, `Infinity`, or one out-of-bounds read.
- Missing option suddenly behaves truthy: suspect prototype pollution or inherited reads before you blame business logic.
- `?.` still throws: check whether someone grouped part of the chain, e.g. `(obj?.a).b`.
- Mechanical `||` to `??` migration fails to parse: parenthesize or rewrite as optional chaining; mixed `??` with `&&`/`||` is intentionally a syntax error without parentheses.

## NEVER do these

- **NEVER** use `Object.assign({}, parsedUserJson)` on untrusted data because it performs `[[Set]]` on the target, which triggers `__proto__` setters. It is seductive because it looks like a harmless clone. The consequence is prototype mutation on the target object. **Instead do** spread into a fresh object or normalize into `{ __proto__: null }`.
- **NEVER** read option bags with inherited fallthrough (`if (opts.flag)`, `opts.method || 'GET'`) when the object crossed a trust boundary. It is seductive because plain objects make missing keys feel cheap. The consequence is that polluted prototypes silently change control flow or request config. **Instead do** explicit defaults up front and gate reads with `Object.hasOwn`.
- **NEVER** partially migrate `a && a.b || fallback` into `a && a.b ?? fallback`. It is seductive because it looks like a one-token upgrade. The consequence is a parse-time `SyntaxError` because `??` cannot mix unparenthesized with `&&` or `||`. **Instead do** `a?.b ?? fallback` or parenthesize deliberately.
- **NEVER** group halfway through an optional chain, e.g. `(obj?.a).b`, because short-circuiting stops only along one continuous chain. It is seductive during refactors and formatting. The consequence is a runtime `TypeError` on `undefined`. **Instead do** `obj?.a?.b`.
- **NEVER** use arrow functions as shared methods by default. It is seductive because the syntax is shorter and "auto-bound". The consequence is one closure per instance, no `prototype`, and unusable `call`/`apply` rebinding. **Instead do** prototype methods unless lexical `this` is the actual requirement.
- **NEVER** curry classes with `bind()` when subclassing or statics matter. It is seductive because bound constructors still work with `new`. The consequence is lost own static properties and a constructor that cannot be used with `extends`. **Instead do** a wrapper factory or subclass.
- **NEVER** preallocate hot arrays with `new Array(n)` or terminate scans by reading past `arr.length`. It is seductive if you come from C/C++ and think in reserve/capacity terms. The consequence is holey arrays, prototype-chain checks, and load sites that never become fast again. **Instead do** dense pushes, `fill`/factory initialization, or `TypedArray`.

## Fallbacks when the ideal path is unavailable

- If you cannot keep record shapes uniform because upstream data is sparse, normalize once at the boundary into a DTO with a fixed key set; keep raw data raw and make the DTO the thing hot code touches.
- If a null-prototype object breaks downstream helpers that expect `toString`/`hasOwnProperty`, keep the unsafe shape at the edge only: use `Map` internally, then convert with `Object.fromEntries()` or a validated serializer at the boundary.
- If V8-specific tuning hurts readability and the profiler says the code is not hot, delete the tuning and keep the semantics. Engine folklore without measurement is technical debt.

## Mandatory reference loading

- **Before** interpreting `--trace-opt`, `--trace-deopt`, `--trace-ic`, `%DebugPrint`, hidden-class churn, array hole regressions, or `JSON.stringify` throughput on Node/Chrome, **READ** [`references/v8-perf.md`](references/v8-perf.md).
- **Before** accepting user-controlled keys, reviewing deep merge / clone / `obj[k] = v` code, hardening options objects, or deciding between `Map` and null-prototype objects, **READ** [`references/prototype-pollution.md`](references/prototype-pollution.md).
- **Before** changing equality/defaulting semantics, writing `==`, implementing `Symbol.toPrimitive`, or migrating legacy guard expressions to `?.` / `??`, **READ** [`references/coercion-traps.md`](references/coercion-traps.md).

## Do NOT load references for these cases

- Do **NOT** load `references/v8-perf.md` for ordinary feature work, code review with no profile evidence, or cross-engine library code that has not been benchmarked on the target runtime.
- Do **NOT** load `references/prototype-pollution.md` when every key is compile-time authored and the object never accepts user input or third-party merge data.
- Do **NOT** load `references/coercion-traps.md` for straightforward typed code paths where there is no equality/defaulting/coercion behavior under review.
