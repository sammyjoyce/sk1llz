# V8 Performance Reference

Load only when optimizing a hot path or diagnosing deopt/IC trace output.

## Hidden classes (V8 "Maps" / "Shapes")

Every object has an internal pointer to a hidden class that records:
- Property names and their memory offsets
- A transition table: "add property `x` → go to shape Cn"

**Transition rules that bite:**

1. Two objects share a shape **only if they took the same transition path**, meaning same properties added in the same order.
   ```js
   const a = {}; a.x = 1; a.y = 2;   // Shape: C0 → Cx → Cxy
   const b = {}; b.y = 2; b.x = 1;   // Shape: C0 → Cy → Cyx
   // a and b have different hidden classes despite same logical content.
   ```

2. A literal with the properties already present takes one transition path at construction:
   ```js
   const a = { x: 1, y: 2 };   // Always Cxy
   const b = { x: 3, y: 4 };   // Always Cxy — shares with a
   ```

3. **Conditional properties fork the shape tree:**
   ```js
   // BAD — two shapes
   const user = { name };
   if (isAdmin) user.permissions = [];

   // GOOD — one shape
   const user = { name, permissions: isAdmin ? [] : null };
   ```

4. **`delete` forces dictionary mode permanently.** The object's properties are copied into a hash table; every subsequent access on that object — not just the deleted key — is a dictionary lookup, typically 10–50× slower than a hidden-class offset read. The only fix is to abandon the object and copy its surviving properties into a fresh one.

5. **`Object.setPrototypeOf` / `obj.__proto__ = p`** globally invalidates optimized code that depended on the old prototype. Per MDN: "the effects are subtle and far-flung, and are not limited to the time spent in the statement."

## Inline cache (IC) states

Each property access or method call in your source is a "call site." V8 tracks per-site state:

| State | Shapes seen | Cost | When |
|---|---|---|---|
| Uninitialized | 0 | Full generic lookup | First execution of the line |
| Monomorphic | 1 | ~1 ns, direct offset | All calls saw the same shape |
| Polymorphic | 2–4 | A few ns, shape comparison chain | A handful of shapes |
| Megamorphic | >4 | ~20–100 ns, global hash lookup | Too many shapes — cliff |

**Going megamorphic is typically 2–20× slower than monomorphic** at that access, and once megamorphic the site never recovers in that execution. Built-in methods (`Array.prototype.forEach`, `Map.prototype.get`) handle polymorphism much better than user-written iterators and should be preferred in polymorphic code.

## Elements kinds lattice

```
PACKED_SMI_ELEMENTS   ──▶  PACKED_DOUBLE_ELEMENTS   ──▶  PACKED_ELEMENTS
        │                          │                            │
        ▼                          ▼                            ▼
HOLEY_SMI_ELEMENTS   ──▶  HOLEY_DOUBLE_ELEMENTS   ──▶  HOLEY_ELEMENTS
```

**Transitions are one-way.** Once your array is PACKED_ELEMENTS, it cannot go back to PACKED_SMI even if you refill it with integers. Once HOLEY, it stays HOLEY (only exception: `Array.prototype.fill` since V8 2025-02-28).

**Accidental demotions to watch for:**
- `arr.push(-0)` — `-0` is a double, demotes SMI→DOUBLE
- `arr.push(NaN)` or `arr.push(Infinity)` — demote SMI→DOUBLE
- `new Array(n)` — born HOLEY_SMI even before you write to it
- `arr[100] = x` when `arr.length < 100` — creates holes
- `arr.length = bigger` — creates holes
- `const a = new Array(3); a[0]='x';` — HOLEY_ELEMENTS forever

**Preserving PACKED_SMI for numeric arrays:**
- Normalize `-0` to `0` on ingest: `if (x === 0) x = 0;` (defeats `-0`)
- Reject or sanitize `NaN` / `Infinity`
- Never preallocate with `new Array(n)`; build with `[]` + push, or `Array.from({length: n}, () => 0)`
- For math-heavy workloads, use `Int32Array` / `Float64Array` — permanently typed, contiguous, no boxing, and `push` is not available (which is the point).

## The out-of-bounds read trap

```js
// ~6× slower than necessary on a 10k-element array
for (let i = 0; i <= arr.length; i++) {   // <= instead of <
  if (arr[i] > max) max = arr[i];
}
```

The last iteration reads `arr[arr.length]`, which is not an own property, so V8 walks `Array.prototype`, `Object.prototype`, `null`. Worse: the IC at that load is marked "has seen out-of-bounds" and every subsequent in-bounds read at that site is also slower. Same hazard: `for (let i=0, x; (x = arr[i]) != null; i++)`.

## `arguments` and leaking

The `arguments` object in non-strict functions is "materialized" lazily, but any function that references `arguments` cannot be inlined by TurboFan. In performance-sensitive code, use rest parameters (`function f(...args)`) — they're a real array, can be inlined, and don't have the "live binding to parameter slots" weirdness of `arguments`.

## Diagnostic flags

Run Node with:

```
node --trace-opt app.js           # Every function V8 decides to optimize
node --trace-deopt app.js         # Every deopt with function + reason
node --trace-ic app.js            # IC state transitions (verbose)
node --allow-natives-syntax       # Enables %OptimizationStatus(), %DebugPrint()
node --print-bytecode             # Ignition bytecode output
```

**Reading `--trace-deopt` output** — the "reason" field is what you act on:
- `wrong map` — hidden class instability. Find the construction sites for the arg object and align them.
- `not a smi` / `not a heap number` — elements kind changed. Find the push that introduced the other type.
- `insufficient type feedback` — the function ran too few times before TurboFan tried to compile; usually benign, but check that hot code is actually hot.
- `soft deopt` — informational, not a real cliff; V8 learned something and re-optimized.

With `--allow-natives-syntax`, in code:
```js
%DebugPrint(obj);              // Prints hidden class address + layout
%HaveSameMap(a, b);            // true if a and b share a hidden class
%OptimizationStatus(fn);       // Bitfield of TurboFan state
```

## String internalization

String literals and short strings are *interned* (de-duplicated) in V8. Comparing two interned strings is a pointer comparison. Comparing two non-interned strings of length N is O(N). `obj[dynamicKey]` where `dynamicKey` was built with `+` concatenation is slower than `obj[literalKey]` at the IC level because the key is not interned. For hot-path property access by computed key, either (a) intern explicitly with `String.prototype[Symbol.for]`-style registries or (b) use a `Map` and key by the concatenated string only once.

## The "avoid polymorphism" heuristic

Write functions that accept *one* logical object shape. If you need polymorphism, split the function. Example:

```js
// BAD — called with three different shapes, goes megamorphic
function render(node) { return node.children.map(render); }
// ...called with Element, TextNode, CommentNode in the same loop.

// GOOD — one call site per shape, each monomorphic
function renderElement(n) { ... }
function renderText(n)    { ... }
function renderComment(n) { ... }
function render(n) {
  switch (n.kind) {
    case 'element': return renderElement(n);
    case 'text':    return renderText(n);
    case 'comment': return renderComment(n);
  }
}
```

The `switch` is cheap; the IC cliff inside a generic `render` is not.
