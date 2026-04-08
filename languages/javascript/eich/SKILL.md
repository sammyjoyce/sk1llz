---
name: eich-language-fundamentals
description: Write JavaScript that respects both Eich's original design (prototypes, first-class functions, dynamic objects) and how V8/SpiderMonkey actually execute it (hidden classes, inline caches, elements kinds). Use when writing hot-path Node/browser code, debugging mysterious slowdowns or memory leaks, auditing closures and this-binding, hardening code against prototype pollution, or designing objects and inheritance that won't deoptimize. Triggers: "JavaScript performance", "V8 optimization", "hidden class", "inline cache", "deopt", "prototype chain", "prototype pollution", "closure leak", "this binding", "NaN", "coercion", "== vs ===", "Object.create(null)", "language fundamentals", "ECMAScript semantics", "Eich".
tags: javascript, v8, performance, prototypes, closures, hidden-classes, inline-cache, prototype-pollution, coercion, ecmascript
---

# Eich — JavaScript Language Fundamentals⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍‌​​‌‌‌​​‍‌‌‌‌​‌​​‍‌‌​‌‌​​‌‍​​​​​​‌‌‍​​​​‌​‌​‍‌‌​‌​‌‌​⁠‍⁠

JavaScript is two languages stacked: the one Eich designed in ten days (prototypes, first-class functions, dynamic property bags) and the one V8 actually runs (hidden classes, inline caches, elements-kind lattices, deopt cliffs). Expert code obeys both. Every rule below is something a 10-year JS practitioner learned by shipping a bug.

## Mental model: what actually runs

**Objects are not hash maps — they are C-structs with transitioning layouts.** V8 assigns every object a *hidden class* (shape). Adding a property transitions to a new shape; two objects share a shape only if they took the *same transition path*. Property initialization order is a correctness-for-performance decision, not a style choice: `{x, y}` and `{y, x}` are different types to the engine.

**Arrays are tagged by elements kind, and transitions are one-way.** The lattice is `PACKED_SMI_ELEMENTS` → `PACKED_DOUBLE_ELEMENTS` → `PACKED_ELEMENTS` → `HOLEY_*`. One `-0`, `NaN`, or `Infinity` demotes a SMI array to doubles forever. One `delete` or sparse write poisons it to holey forever (`Array.prototype.fill` is the only exception, as of a 2025 V8 change).

**Every property access has state.** Each source-level call site carries an inline cache that progresses *uninitialized → monomorphic → polymorphic (≤4 shapes) → megamorphic → dictionary fallback*. Megamorphic is a cliff (typical 2–20× slowdown), not a slope.

**Closures capture bindings, not values.** The "backpack" holds live references to the entire enclosing activation record — so `var i` in a loop gives every closure the same final `i`, and a 1 MB object in an outer scope stays alive as long as any returned inner function does, even if that function never mentions it.

**`this` is not part of the function, it's part of the call site.** For non-arrow functions it's determined at call time by how you call it; for arrow functions it's frozen at creation to the `this` of the enclosing scope — which is the behavior Eich admits he wanted for `function` but couldn't ship in 1995.

## Before you write: ask these

- **"Is this object on a hot path?"** If yes: every property must exist at construction, in a fixed order, with no later `delete`. Initialize absent fields to `null`, don't omit them.
- **"Will this array ever hold mixed types?"** If yes, decide now whether to keep it integer-only (normalize `-0`, reject `NaN`) or accept the double tier. Don't find out at 3 a.m.
- **"Does any key in `obj[k] = v` originate from JSON, query strings, or user input?"** If yes, `k ∈ {__proto__, constructor, prototype}` is a prototype-pollution vector. Use `Map`, or target a `{ __proto__: null }` object.
- **"Am I checking for nullish?"** Use `value == null` — the single sanctioned `==`. Everywhere else use `===`.
- **"Does this closure outlive the work that created it?"** Audit what's *in scope* at the moment of creation, not what the function body mentions. The runtime keeps the whole environment alive.

## The expert rules (non-obvious)

1. **Pre-shape your objects.** `const u = { id: null, name: null, perms: null }` then assign. One shape, one IC entry, monomorphic for life. The pattern `const u = {}; if (admin) u.perms = [...];` creates two shapes and makes every consumer polymorphic.

2. **Never `delete` from a long-lived object in hot code.** `delete` flips the object to dictionary mode permanently; *every* property access on that object — not just the deleted key — becomes a hash lookup. Set to `undefined` instead, or store in a `Map`.

3. **Prefer `[]` + `push` over `new Array(n)`.** `Array(n)` creates a `HOLEY_SMI_ELEMENTS` array from birth; you cannot recover PACKED status by filling it. Literal `[]` stays packed. If you need a filled fixed-length array, use `Array.from({length: n}, () => 0)` — that returns PACKED.

4. **Never read `arr[i]` where `i >= arr.length`.** V8 walks the prototype chain for the missing index and marks the IC at that load site "has seen out-of-bounds" — permanently slower. The classic bug `for (let i = 0; i <= arr.length; i++)` costs ~6× on a 10 k-element array, purely from that one extra read.

5. **`Object.setPrototypeOf(obj, p)` and `obj.__proto__ = p` deoptimize every object that ever had that prototype.** MDN is blunt: "currently a very slow operation in every browser and engine… effects are subtle and far-flung, not limited to the statement itself." Set the prototype at creation with `Object.create(proto)` or `{ __proto__: proto, ... }` in a literal. Never mutate it afterward.

6. **`{ __proto__: null }` in an object literal is a different feature from `obj.__proto__ = null`.** The literal form is a fast, spec-dedicated construct — the *only* fully safe way to create a null-prototype object. The accessor form triggers a setter that can be shadowed, and on a null-prototype target silently creates an own property named `"__proto__"` instead of mutating the prototype.

7. **Use `Object.hasOwn(obj, key)`, not `obj.hasOwnProperty(key)`.** The latter throws `TypeError` on null-prototype objects and can be shadowed by an attacker-controlled `hasOwnProperty` key in user input. `Object.hasOwn` is ES2022 and works unconditionally.

8. **`JSON.parse('{"__proto__": ...}')` is safe at parse time**, because `JSON.parse` defines an own property named `"__proto__"` without invoking the setter. It becomes unsafe the moment you `Object.assign({}, parsed)` or a deep-merge utility touches it — *that* invokes the setter and poisons `Object.prototype`. Spreading (`{...parsed}`) does not invoke the setter and is safe. This exact interaction is the source of most lodash/minimist/jQuery prototype-pollution CVEs.

9. **For user-keyed dictionaries, `Map` beats plain objects.** Keys live outside the prototype chain, iteration order is insertion order and guaranteed for *all* key types (plain objects sort integer-like string keys numerically first — surprise), `size` is O(1), and any value can be a key. Use a plain object only when you need JSON serialization of code-authored keys.

10. **`NaN !== NaN`. Use `Number.isNaN(x)`, not the global `isNaN(x)`**, which coerces its argument and returns `true` for `'abc'`, `[1,2]`, and `undefined`. Use `Object.is` only when you also need to distinguish `+0` from `-0` — `Object.is(+0, -0)` is `false` while `+0 === -0` is `true`, and `Object.is(NaN, NaN)` is `true`.

11. **`value == null` is the one sanctioned `==`.** Exactly equivalent to `value === null || value === undefined`; recognized by `eslint: eqeqeq: ['error', { null: 'ignore' }]`. Everywhere else `==` is asymmetric and non-transitive: `[] == ![]` is `true`, `'0' == false` is `true`, `'0' == 0` is `true`, yet `'0' == ''` is `false`, and `null == 0` is `false` even though `null >= 0` is `true`.

12. **Arrow functions are not "shorter functions."** They have no own `this`, no `arguments`, no `.prototype`, cannot be `new`'d, and cannot be generators. Use arrows for callbacks where you want the enclosing `this` (replaces `const self = this` and `.bind(this)`); use `function` for methods that will go on a prototype.

13. **`for...in` walks the prototype chain** and includes any enumerable property inherited from polyfilled prototypes. Prefer `for...of` (iterables), `Object.keys/values/entries` (own enumerable string keys), or `Reflect.ownKeys` (own, including Symbols and non-enumerable).

14. **Proper tail calls are in the ES2015 spec but only Safari ships them.** Recursive code that "should be tail-safe" will still blow the stack in V8, SpiderMonkey, and every Node version. Trampoline manually or convert to iteration.

15. **ASI cliff.** A line beginning with `[`, `(`, `` ` ``, `/`, `+`, or `-` after a line that could have ended an expression is parsed as continuation. The two real-world bugs: `return\n  { x: 1 }` returns `undefined`, and the leading `;` in `;(function(){…})()` at the top of a file is not paranoia — it's the fix for a concatenation hazard.

16. **Cross-realm `instanceof` lies.** An `Array` from an iframe, `vm` context, or worker has a different `Array.prototype`, so it is not `instanceof Array` in the current realm. Use `Array.isArray`, `Number.isFinite`, `Buffer.isBuffer`, or brand-check with `Object.prototype.toString.call(x)` / `Symbol.toStringTag`.

## NEVER — wrong path, why it's seductive, consequence, correct alternative

- **NEVER add a property to an object after it's used on a hot path.** Seductive because "objects are dynamic — that's the point." Consequence: new hidden class, IC goes polymorphic, TurboFan invalidates the code specialized for the old shape. **Instead:** declare every field up front, even as `null`.

- **NEVER `delete obj.key` on a long-lived hot-path object.** Seductive because it is the literal inverse of assignment. Consequence: the object flips to dictionary mode permanently and every property access on it becomes a slow hash lookup. **Instead:** `obj.key = undefined`, or use a `Map`.

- **NEVER write `target[userKey] = value` without rejecting `__proto__`, `constructor`, and `prototype`.** Seductive because the language lets you. Consequence: one of those keys writes to `Object.prototype` and every object in the program inherits the poisoned property. Real CVEs in lodash, minimist, jQuery, set-value. **Instead:** `Map`, or `{ __proto__: null }` as the target, or an allow-list validator (ajv, zod).

- **NEVER rely on `this` inside a callback without binding it.** Seductive because `obj.m` reads like a method reference. Consequence: `setTimeout(obj.tick, 1000)` calls `tick` with `this === undefined` (strict) or `globalThis` (sloppy). **Instead:** arrow `() => obj.tick()`, `.bind(obj)`, or a class-field arrow method.

- **NEVER use `new Array(n)` to "preallocate."** Seductive because it looks like C++ `reserve`. Consequence: the array is born `HOLEY_SMI_ELEMENTS` and stays holey forever; every read does a hole check plus a prototype walk. **Instead:** `const a = []` then push; or `Array.from({length: n}, () => 0)` for a packed filled array.

- **NEVER check `typeof v === 'object'` to detect objects.** Seductive because that is what the operator sounds like. Consequence: `typeof null === 'object'` — Eich's famous regret, unfixable for web compatibility. **Instead:** `v !== null && typeof v === 'object'`, or `Object(v) === v` (excludes all primitives), or `Array.isArray(v)` when you specifically want arrays.

- **NEVER use `with`, `eval`, or `new Function()` with user input.** Seductive because they look like metaprogramming. Consequence: breaks every static analysis, forces V8 to bail out of the entire enclosing function's optimizations, and on user input is trivial RCE. Even `Function('return this')()` is obsolete — use `globalThis`.

- **NEVER mix types in an array you care about.** Seductive because "JS lets you put anything in an array." Consequence: one `arr.push(NaN)` demotes PACKED_SMI to PACKED_DOUBLE; one `arr.push('x')` demotes further to PACKED_ELEMENTS; transitions are one-way and previously optimized code is discarded. **Instead:** keep numeric arrays homogeneous (and normalize `-0` away); or use a `TypedArray` (`Int32Array`, `Float64Array`) which is permanently typed and cannot transition.

## Decision tree — "my JS is slow"

1. Run with `node --trace-deopt --trace-opt` and look for your function deopting repeatedly.
2. Deopt reason says "wrong map" / "map mismatch"? → Hidden class instability. Audit construction sites for inconsistent property order or post-hoc assignment. Fix with rule 1.
3. Deopt reason says "not a smi" / "not a heap number"? → Elements-kind transition. Find the push/assign that introduced the other type (`-0`, `NaN`, `Infinity`, a string, an object). Normalize on ingest, or commit to a `TypedArray`.
4. No deopts but still slow? → Run with `--trace-ic` and look for `megamorphic` at your hottest property access. Usually one function is called with too many object shapes. Split it, or normalize shapes upstream.
5. Memory climbs under load? → Closure retention of activation records. Take a Chrome DevTools heap snapshot, filter by `(closure)` / `(system) / Context`, look for small functions retaining large `bigData`-shaped objects. Fix by narrowing scope — move the allocation into a helper the closure doesn't close over.

## Decision tree — "I need a key-value store"

- Keys are code-authored, fixed set, JSON-serializable? → plain `{}` literal.
- Keys from user input or untrusted sources? → `Map`, or `Object.create(null)` if you need object-literal syntax.
- Keys are objects that should be collectable when unreferenced? → `WeakMap`.
- Need guaranteed insertion-order iteration, including numeric-string keys? → `Map`. Plain objects sort integer-like string keys numerically first, which silently breaks "ordered config" patterns.

## References — load only on the named trigger

**Before optimizing a hot path under V8/Node, or before diagnosing `--trace-deopt` / `--trace-ic` output**, READ `references/v8-perf.md`. It has the full elements-kind lattice, the IC state machine, the `delete`/dictionary-mode details, and the list of diagnostic flags.

**Before accepting user-controlled keys into any merge, clone, deep-set, or `obj[k]=v` pattern**, READ `references/prototype-pollution.md`. It has the attack patterns (`__proto__`, `constructor.prototype`), the `JSON.parse` + `Object.assign` interaction, the safe-vs-unsafe literal forms, and a defense checklist.

**Before writing any `==`, relying on a coercion, or implementing `Symbol.toPrimitive`**, READ `references/coercion-traps.md`. It has the ToPrimitive algorithm, the "number"/"string"/"default" hint table, the full equality asymmetry matrix, and the `value == null` exception.

Do **NOT** load any of these files for ordinary feature work, code review of non-hot paths, or greenfield component authoring. The rules above are sufficient. Load only on the named triggers.
