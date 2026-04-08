---
name: crockford-good-parts
description: 'Write or refactor JavaScript in Douglas Crockford''s "Good Parts" style — a defensive subset that bans `this`, `new`, `class`, `==`, `for...in`, `++`, and ASI-reliant code, replacing them with factory functions returning `Object.freeze({...})`, closure-based privacy, and JSLint-grade discipline. Use when authoring or reviewing JavaScript that must survive ten-plus years (financial code, browser-shipped libraries, security-sensitive widgets, JSON/data interchange, ad sandboxes), when refactoring class/`this`-heavy code toward immutable factories, when debugging surprising behavior from automatic semicolon insertion, `typeof null`, `for...in` prototype leaks, `hasOwnProperty` shadowing, `NaN`, or boxed primitives, when porting code to be JSLint-clean, or when the user mentions Crockford, "Good Parts", JSLint, "Ice Factory", frozen objects, class-free OOP, POLA / capability-based security, or ADsafe.'
tags: closures, prototypes, objects, functions, scope, json, patterns, clean-code, browser, web
---

# Douglas Crockford Style Guide⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍​‌​‌​​‌​‍​​‌‌‌‌‌​‍​​​‌‌‌‌‌‍​​‌​‌​​​‍​​​​‌​​‌‍​‌​​​​​‌⁠‍⁠

## What this skill encodes

Crockford's style is not "old JavaScript." It is a **defensive subset** designed for code that ships into adversarial environments — browsers running other people's scripts, financial systems, JSON parsers eating untrusted bytes, ad sandboxes embedded in third-party pages. Every rule is a scar from a real production bug. Treat each NEVER as "this caused an outage I can describe."

His thinking has **evolved** — modern Crockford (2014+) is *not* the 2008 book:

- He no longer recommends `Object.create` (which the book advocates). He abandoned it because prototype chains still leak via `for...in` and `instanceof` still crosses the trust boundary.
- He no longer recommends prototypal inheritance at all.
- His current pattern is: **factory function → close over private state → return `Object.freeze({...})`**. Lowercase factory name. No `new`, no `this`, no `class`, no `Object.create`, no prototype chain.
- He has accepted ES6 *destructuring*, `let`/`const`, template literals, and shorthand methods.
- He still rejects `class`, arrow functions for methods, hoisted function statements, `++`/`--`, and `==`.

If your reflex is "this looks old-fashioned," you have not yet hit the bug it prevents.

## Three questions before any line

1. **"Could this line behave differently if `this` is rebound?"** If yes, you wrote it wrong. Crockford-style code never reads `this`. Pass dependencies explicitly or close over them.
2. **"If a stranger augments `Object.prototype`, does my code still work?"** If no, you used bare `for...in`, or you trusted `instanceof` across realms, or you used `obj.hasOwnProperty(...)` directly.
3. **"Does the failure mode produce a wrong answer or a thrown error?"** Crockford prefers throws-loudly over silently-coerces. Every `==`, `++`, ASI-relying line is a vote for silent wrongness.

## The pattern (modern Crockford, post-2014)

```javascript
'use strict';

function make_account(spec) {
    const {initial_balance = 0} = spec;
    let balance = initial_balance;          // private via closure

    function deposit(amount) {
        if (typeof amount !== 'number' || !Number.isFinite(amount) || amount <= 0) {
            throw new Error('deposit: positive finite number required');
        }
        balance += amount;
        return balance;
    }

    function withdraw(amount) { /* ... */ }
    function get_balance() { return balance; }

    return Object.freeze({deposit, withdraw, get_balance});
}
```

Five non-obvious things in those twelve lines:

1. **Lowercase `make_account`.** Capitalised names are reserved by convention for `new`-able functions. Lowercase tells the reader "do not put `new` in front of me." Mixing the two is how `new make_account()` ends up returning the wrong `this`.
2. **`Object.freeze` is shallow.** If you return `{items: []}`, the array is still mutable. Freeze nested mutable state explicitly, or do not return it.
3. **`Object.freeze` fails *silently* in sloppy mode.** Without `'use strict';` (or an ES module context) reassigning a frozen property does nothing and throws nothing. Always emit modules or `'use strict';` at the file top.
4. **`Number.isFinite`, not global `isFinite`.** Global `isFinite('5')` is `true` because it coerces. `Number.isFinite('5')` is `false`. Same trap with `isNaN` vs `Number.isNaN`.
5. **`throw new Error(...)`, never `throw 'string'`.** A thrown string has no stack, no `name`, and breaks `err instanceof Error` checks downstream — silently swallowing the bug at the catch site.

## Specific landmines Claude does not naturally avoid

### ASI: the `return\n{` trap and the `(`-leading-line trap

Automatic semicolon insertion does **not** insert a semicolon when the next token continues a valid expression. The two famous victims:

```javascript
// Trap 1: return + newline → silently returns undefined
return                          // ASI inserts ';' here (return is a "restricted production")
{                               // because '{' on its own line at statement
    status: true                //   position is parsed as a *block*, not an object
};

// Trap 2: line starting with '(' is parsed as a continuation
const a = b + c
(d + e).print()                 // becomes: const a = b + c(d + e).print()
```

Rules that defuse both:

- **K&R braces are not aesthetic.** `{` MUST be on the same line as the keyword. This is the only way `return {...}` and `if (x) {...}` survive ASI.
- **Defensive leading semicolon.** Any file or IIFE that starts with `(` or `[` MUST start with `;`, so concatenation cannot turn the previous statement into a call.
- **Never break before `++` / `--`** — ASI inserts a semicolon, then `++c` becomes its own statement, silently changing meaning.
- **`return`, `throw`, `break`, `continue`, postfix `++`/`--`** are all "restricted productions": a newline immediately after them triggers ASI even if the next line would have parsed.

### `for...in` is broken without `hasOwnProperty`

`for...in` walks the entire prototype chain. The moment any code on the page (a polyfill, a third-party library, an `Object.prototype` augmentation) adds an enumerable property to `Object.prototype`, your loop visits it.

```javascript
// WRONG: leaks inherited keys
for (const key in obj) { use(obj[key]); }

// RIGHT: filter, and call hasOwnProperty via Object.prototype to dodge shadowing
for (const key in obj) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
        use(obj[key]);
    }
}

// BETTER: do not use for...in at all
Object.keys(obj).forEach(function (key) { use(obj[key]); });
```

Why `Object.prototype.hasOwnProperty.call(obj, key)` and not `obj.hasOwnProperty(key)`? Because untrusted JSON like `{"hasOwnProperty": 1}` shadows the method on that one object — `obj.hasOwnProperty` is now a number, and your guard throws "is not a function." This is the canonical "untrusted JSON destroys your guard" bug, and it is the reason `Object.create(null)` exists.

### `typeof` lies in four places

| Expression          | `typeof` returns | Use instead                              |
|---------------------|------------------|------------------------------------------|
| `typeof null`       | `'object'`       | `value === null`                         |
| `typeof []`         | `'object'`       | `Array.isArray(value)`                   |
| `typeof NaN`        | `'number'`       | `Number.isNaN(value)` (not global `isNaN`, which coerces) |
| `typeof /regex/`    | impl-defined     | `value instanceof RegExp` *and accept it fails across realms / iframes* |

Heuristic: in Crockford-style code, `typeof` should only be checking against `'undefined'`, `'string'`, `'number'`, `'boolean'`, or `'function'`. Any other use is suspicious.

### Numbers are IEEE-754 doubles — and Crockford hates it

- `0.1 + 0.2 === 0.30000000000000004`. Crockford proposed DEC64 (decimal floats) to fix this; nobody adopted it. So in Crockford-style code, money MUST be integer minor units (cents) or `BigInt` — never `Number`. The moment you see `*100` followed by `Math.round`, you have a compounding bug.
- `Number.MAX_SAFE_INTEGER` is `2^53 - 1 ≈ 9.007e15`. Above that, integer arithmetic is *wrong*, not "imprecise": `9007199254740993 === 9007199254740992` is `true`. Database IDs, Twitter snowflakes, nanosecond timestamps routinely exceed this. **Use strings for IDs.**

### `Object.freeze` does not stop class-style mutation

Frozen factory output is immutable. But objects produced by `class` constructors are **not** — and worse, modifying `SomeClass.prototype.method` after instances exist mutates *every existing instance retroactively*. This is the strongest single reason Crockford banned `new` and `class`. If your codebase mixes paradigms, `Object.freeze(cart)` is no defense if `cart` was made with `new ShoppingCart`.

### Boxed primitives are a falsy-check bomb

`new Boolean(false)` is an *object*, and all objects are truthy:

```javascript
if (new Boolean(false)) { /* THIS RUNS */ }
typeof new Number(0) === 'object'   // not 'number'
```

Never `new String`, `new Number`, `new Boolean`, `new Object`, `new Array`. Use literals.

## Decision tree

| Situation                                          | Crockford answer                                                  |
|----------------------------------------------------|-------------------------------------------------------------------|
| "Should this be a class?"                          | No. Factory function returning `Object.freeze({...})`.            |
| "I need inheritance."                              | Compose: `const {method} = make_base(spec);` then re-export.      |
| "I need private state."                            | Closure variables. Do not return them.                            |
| "I need this method to remember the instance."     | Reference the closure variable. The function literally never sees `this`. |
| "I want to detect an array."                       | `Array.isArray(x)`. Never `instanceof`, never `typeof`.           |
| "I want to test equality."                         | `===`. Even for null: `x === null`, never `x == null`.            |
| "I want to iterate object keys."                   | `Object.keys(obj).forEach(...)`. Never bare `for...in`.           |
| "I want to define a method on a built-in."         | Don't. If you must, gate with `if (!Array.prototype.x) {...}`.    |
| "I need to parse JSON."                            | `JSON.parse(text)` inside `try`/`catch`. Never `eval`.            |
| "I need to handle money."                          | Integer minor units or `BigInt`. Never `Number`.                  |
| "Should this `return` span multiple lines?"        | The expression must start on the *same* line as `return`.         |
| "Should I use a function statement or expression?" | Expression assigned to `const`, so hoisting cannot bite.          |
| "Method on a returned object: arrow or `function`?"| Named `function`. Arrows steal `this` and produce anonymous frames in stack traces. |

## NEVER list (with consequences, not just bans)

- **NEVER `==` / `!=`** because coercion has 100+ silent rules: `'' == 0` is `true`; `[1] == true` is `true`; `[null] == false` is `true`. The bugs are silent and rare, so reviewers stop noticing them. Instead use `===`; if you genuinely need both null and undefined, write `value === null || value === undefined` explicitly.
- **NEVER `eval` or `new Function(...)`** because it bypasses every static analysis tool, runs in caller scope, **deopts the whole enclosing function in V8** (named the "eval poison" in V8 internals), and enables RCE if any input is user-controlled. Instead build the data structure directly, or `JSON.parse` for data.
- **NEVER `with`** because it makes every name resolution a runtime lookup; one new property on the scope object silently shadows your locals. Strict mode forbids it. Instead destructure: `const {x, y} = obj;`.
- **NEVER `++` / `--`** because they encourage `arr[i++] = arr[j++]` which fuses sequencing, side effect, and value-returning into one operator — bugs land in the seam. Instead use `i += 1`. The discipline cost is one character; the bug rate drops measurably.
- **NEVER bare `function name() {}` at statement position** because function statements hoist to the top of the function, so reading order differs from running order, and `function f() {}` inside an `if` is implementation-defined. Instead `const f = function f() {...};` (the inner name aids stack traces).
- **NEVER `new String('x')` / `new Number(1)` / `new Boolean(true)`** because they create *boxed* objects, which are truthy even when wrapping `false`. The single most surprising falsy-check failure in JavaScript. Instead use literals.
- **NEVER `arguments`** because it is not a real array (no `.map`, no `.filter`); in non-strict mode it aliases parameters bidirectionally; arrow functions don't have one. Instead use rest parameters: `function f(...args) {}`.
- **NEVER bare `for...in`** — see the landmines section. Default to `Object.keys` / `Object.entries`.
- **NEVER throw a non-`Error`** because strings have no stack, no `name`, no `instanceof Error`, and tooling silently drops them. Instead `throw new Error('message')` or a subclass.
- **NEVER mutate inputs** because Crockford-style functions return new frozen values; input mutation creates spooky-action-at-a-distance and breaks `Object.freeze` discipline at the call site. Instead spread: `return Object.freeze({...input, changed: value});`.
- **NEVER use arrow functions for methods on a returned object** because they capture `this` from the definition site, and Crockford-style code has no `this` to capture. Instead use named function expressions so stack traces are readable.
- **NEVER omit `'use strict';` outside ES modules** because without it, `Object.freeze` violations, accidental globals, and `delete` of non-configurable properties all fail silently.
- **NEVER use a `Number` for an ID** because once it crosses `2^53`, equality lies. Instead use strings or `BigInt`.

## Edge cases and fallbacks

- **Codebase already uses `class` heavily.** Don't rewrite — *isolate*. Wrap class instances in a Crockford-style facade: `function make_x(class_instance) { return Object.freeze({...}); }`. New code uses the facade; legacy stays untouched.
- **Framework requires `this` (older React class components, Node streams, Express middleware bound to `req`).** Frameworks override Crockford. Quarantine the `this` to one thin adapter file; keep your business logic in pure factories the adapter calls.
- **Performance-critical hot loop creating millions of objects.** Frozen factories are 2–10× slower than `class` instantiation in V8 because they cannot share hidden classes. Profile first; if and only if object creation is the actual bottleneck, drop to `class` for that one hot path with a comment explaining the deviation. Crockford's own answer: "if you have a million-object hot loop, use a typed array instead."
- **You consume an API that returns prototype-bearing objects.** Convert at the boundary: `const safe = Object.freeze({...untrusted})`. The spread breaks the prototype chain *for own enumerable properties* and the freeze prevents downstream mutation. Note: the spread does not copy non-enumerable properties or symbols — if those matter, use `Object.create(null)` plus explicit copying.
- **You must accept untrusted JSON keys.** Build the destination with `Object.create(null)` so prototype keys (`__proto__`, `constructor`, `toString`) cannot be smuggled in via prototype pollution. Or use `Map`.

## Loading triggers

**Before** starting a multi-module design that needs capability-based security or sandboxed execution, **READ** [`references/philosophy.md`](references/philosophy.md). It contains the POLA (Principle of Least Authority) patterns, the historical evolution from `object()` → `Object.create` → frozen factories, the `Function.prototype.method` augmentation pattern, and ADsafe context that informs why each rule above exists.

**Do NOT load** `references/philosophy.md` for routine "make this code Crockford-style" tasks — the rules in this file are sufficient and the philosophy file is historical context, not a checklist.

**Before** declaring any code "Crockford-clean," **RUN** `node scripts/jslint_check.js path/to/file.js`. JSLint catches the cases your eyes will miss: ASI traps near closing parens, `==` hidden inside long boolean expressions, `for...in` without `hasOwnProperty`, `++`/`--` in expression position, missing `'use strict'`. Crockford's own rule: **JSLint warnings are errors, not suggestions.**
