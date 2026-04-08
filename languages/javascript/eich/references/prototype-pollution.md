# Prototype Pollution Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​‌‌‌‌​‍​​‌​​‌​‌‍‌‌​‌‌​​​‍‌​​‌‌​‌​‍​​​​‌​‌​‍‌​​​‌‌​‌⁠‍⁠

Load only when accepting user-controlled keys into any object-merge, clone, deep-set, or `obj[k] = v` pattern.

## The attack in one line

If an attacker can influence the property name `k` in an expression like `obj[k1][k2] = value`, they can set `k1 = "__proto__"` and write to `Object.prototype`. Every object in the program then inherits the polluted property.

## Two attack vectors

**`__proto__` setter:**
```js
const obj = {};
obj["__proto__"]["isAdmin"] = true;   // writes to Object.prototype
({}).isAdmin;                          // true — every object now admin
```

**`constructor.prototype` chain** (works even if `__proto__` is disabled):
```js
obj["constructor"]["prototype"]["isAdmin"] = true;
```

Any user-supplied key path of length ≥ 2 can be either attack. Length-1 writes (`obj[userKey] = value`) are *not* directly exploitable because you can only create an own property named `"__proto__"`, not mutate the prototype — but they become exploitable the moment something merges, clones, or deep-copies `obj`.

## The JSON.parse + Object.assign trap

```js
// Safe: parser creates an own property named "__proto__"
const parsed = JSON.parse('{"__proto__": {"isAdmin": true}}');
parsed.__proto__;           // { isAdmin: true } — own property, harmless
({}).isAdmin;               // undefined — Object.prototype is clean

// UNSAFE: Object.assign iterates source's own keys and does [[Set]] on target,
// which triggers the __proto__ SETTER and mutates target's prototype.
const target = Object.assign({}, parsed);
({}).isAdmin;               // true — ALL objects are now admin
```

**The rule:** `JSON.parse` result is safe until it touches any operation that does `[[Set]]` on a fresh object. `Object.assign`, deep-merge libraries (lodash `merge`, `defaultsDeep`), `_.set(obj, path, val)`, and `for...in` copy loops all do this.

**`{...parsed}` (spread) is safe** because spread uses `[[CreateDataPropertyOrThrow]]`, which does not invoke setters. Prefer spread over `Object.assign` for this reason when the source might be untrusted.

## Real CVEs to remember the shape of

| Library | Pattern |
|---|---|
| `lodash.merge` / `lodash.set` (<4.17.12) | `_.merge({}, JSON.parse(userJson))` |
| `minimist` (<1.2.3) | `--__proto__.isAdmin=true` on the command line |
| `jQuery.extend(true, …)` (<3.4.0) | Deep extend from user JSON |
| `set-value`, `dset`, `object-path` (various) | `set(obj, userPath, val)` where path is `"__proto__.x"` |

Pattern recognition: any library whose README says "deep merge," "deep set," "deep defaults," or "path assignment" is a prototype-pollution sink if the path or source is user-influenced.

## Defenses, in order of robustness

### 1. Use the right data structure

**`Map` has no prototype chain for its keys.** If your dictionary is keyed by user data, it should almost always be a `Map`, not a plain object:

```js
const users = new Map();
users.set(userInput, userData);   // Completely safe; keys are not property names.
```

### 2. Null-prototype objects for object-shaped configs

When you need object-literal syntax (e.g., passing to an API that requires an object), use the literal form:

```js
const opts = { __proto__: null, mode: "cors", body: payload };
fetch(url, opts);
```

**Critical distinction:** `{ __proto__: null }` in an *object literal* is a dedicated, fast, spec-guaranteed feature that sets the `[[Prototype]]` internal slot. It is NOT the same as `obj.__proto__ = null` (which is a deprecated accessor). And on a null-prototype object, `obj.__proto__ = x` silently creates an own property called `"__proto__"` — it does not change the prototype.

`Object.create(null)` is equivalent for construction but does not allow setting initial properties in one expression.

### 3. Validate keys explicitly

If you must use `obj[userKey] = value` with a plain object, blacklist the three names:

```js
const FORBIDDEN = new Set(["__proto__", "constructor", "prototype"]);
function safeSet(obj, key, val) {
  if (FORBIDDEN.has(key)) throw new Error(`unsafe key: ${key}`);
  obj[key] = val;
}
```

For nested paths, apply this recursively — `foo.__proto__.x` is just as dangerous as `__proto__`.

### 4. Schema-validate the input

ajv, Zod, io-ts, valibot, etc. with `additionalProperties: false` (or equivalent strict mode) rejects unknown keys including `__proto__`. This is the most robust long-term defense because it also catches typos and future attack variants.

### 5. Read defensively

Even if you cannot prevent pollution (third-party code in your process), you can mitigate on read:

```js
// Instead of:  if (options.someFlag)
// Use:
if (Object.hasOwn(options, "someFlag") && options.someFlag) { ... }
```

`Object.hasOwn` is the ES2022 replacement for `obj.hasOwnProperty(key)` and works on null-prototype objects. For boolean flags specifically, always require the property to be own and truthy — never trust inherited truthy values.

### 6. Freeze prototypes at startup

In Node, at the very top of your entry point:

```js
Object.freeze(Object.prototype);
Object.freeze(Array.prototype);
Object.freeze(Function.prototype);
// … repeat for built-ins you care about
```

Or use the SES shim (`@endo/ses`) which walks all intrinsics. In Node you can also pass `--disable-proto=delete` to make `Object.prototype.__proto__` unreachable. Both approaches can break polyfills that legitimately modify prototypes, so run them after any polyfill loading.

## Defense checklist

When reviewing code, flag every instance of:

- [ ] `obj[k1][k2] = ...` where `k1` comes from user input, JSON, query string, CLI args, env vars, or a database column populated by any of those
- [ ] `Object.assign(x, untrusted)` — switch to `{...untrusted}` or a schema-validated copy
- [ ] `_.merge`, `_.defaultsDeep`, `_.set` — or any "deep" variant in any library — fed user data
- [ ] `for (const k in source) target[k] = source[k]` — trivially pollutable
- [ ] `obj.hasOwnProperty(key)` — replace with `Object.hasOwn(obj, key)`
- [ ] `if (options.flag)` where `options` came from untrusted input — add `Object.hasOwn`
- [ ] Plain object literal `{}` used as a user-keyed map — replace with `Map`
- [ ] `fetch(url, options)` where `options` is a plain `{}` from user data — use `{ __proto__: null, ...validated }`
