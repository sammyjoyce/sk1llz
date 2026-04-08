# Coercion & Equality Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​‌​​​​‍​‌‌​​​​‌‍‌​​‌​​​‌‍​​​​​​​​‍​​​​‌​​‌‍‌​​‌‌​‌​⁠‍⁠

Load only when writing `==`, relying on a coercion, or implementing `Symbol.toPrimitive` / `valueOf` / `toString`.

## ToPrimitive — the algorithm that runs for every coercion

When the language needs a primitive from an object, it calls the abstract operation `ToPrimitive(input, hint)`. Steps:

1. If `input[Symbol.toPrimitive]` exists, call it with `hint` and use the result (must be a primitive or `TypeError`).
2. Else if `hint === "string"`: try `toString()` then `valueOf()`, take the first primitive.
3. Else (`hint === "number"` or `"default"`): try `valueOf()` then `toString()`, take the first primitive.

**Hint is chosen by context:**

| Operation | Hint |
|---|---|
| `String(obj)`, `` `${obj}` ``, template literal, `"" + obj` via `toString` path | `"string"` |
| `+obj`, `obj - 1`, `obj * 2`, `Number(obj)`, `obj < 5` | `"number"` |
| `obj + x` (binary `+`), `obj == primitive` | `"default"` |

**`Date` is the lone built-in that treats `"default"` as `"string"`** — every other built-in treats `"default"` as `"number"`. This is why `date1 - date2` is a number (milliseconds) but `date1 + ''` is a human-readable string.

## `Symbol.toPrimitive` — the full-control escape hatch

```js
const money = {
  amount: 4200,
  currency: "USD",
  [Symbol.toPrimitive](hint) {
    if (hint === "number") return this.amount;
    if (hint === "string") return `$${this.amount / 100}`;
    return `${this.currency} ${this.amount / 100}`;   // default
  }
};

+money;            // 4200
`${money}`;        // "$42"
money + "";        // "USD 42"
```

**Gotcha:** `Symbol.toPrimitive` must return a primitive. Returning an object throws `TypeError`. This is unlike `valueOf`/`toString`, which silently fall through to the next candidate if they return an object.

## `==` — the asymmetry matrix (selected highlights)

`==` is specified by the Abstract Equality algorithm which has eleven numbered steps and several recursive coercions. The results developers misremember:

| Expression | Result | Why |
|---|---|---|
| `null == undefined` | `true` | Step 2, hardcoded |
| `null == 0` | `false` | `null` coerces only to `undefined` in `==`, not to 0 |
| `null >= 0` | `true` | Relational operators use `ToNumber`, so `null → 0` |
| `null > 0` | `false` | Same `ToNumber`, but `0 > 0` is false |
| `[] == false` | `true` | `[] → "" → 0`, `false → 0` |
| `[] == ![]` | `true` | `![] → false → 0`, `[] → 0` |
| `'0' == false` | `true` | Both coerce to `0` |
| `'0' == 0` | `true` | `'0' → 0` |
| `'0' == ''` | `false` | Both strings, string-compared, different |
| `NaN == NaN` | `false` | `NaN` is never equal to anything, even itself |
| `{} == {}` | `false` | Two distinct object references |
| `new String('a') == 'a'` | `true` | Object coerces to primitive |

**Non-transitivity:** `'0' == 0` and `0 == ''` are both `true`, but `'0' == ''` is `false`. This alone should terrify anyone.

## The one sanctioned `==` — nullish check

```js
if (value == null) { ... }   // true iff value is null or undefined
```

Exactly equivalent to `value === null || value === undefined`. This is the only `==` that ESLint's `eqeqeq` rule accepts when configured as `["error", { null: "ignore" }]`. Use it freely; use `===` everywhere else.

## `Object.is` — `===` with two corrections

```js
Object.is(NaN, NaN);   // true    (=== gives false)
Object.is(+0, -0);     // false   (=== gives true)
Object.is(1, 1);       // true
```

Use `Object.is` when you need NaN-equals-NaN (memoization keys, `React.useMemo` dep arrays — which in fact uses `Object.is`) or when you specifically need to distinguish `+0` from `-0` (some math code). For all other cases, `===` is cheaper and clearer.

## `isNaN` vs `Number.isNaN`

```js
isNaN("abc");          // true    — coerces to NaN first. USELESS.
isNaN(undefined);      // true    — coerces to NaN first. USELESS.
isNaN(NaN);            // true
Number.isNaN("abc");   // false   — is the string literally NaN? No.
Number.isNaN(NaN);     // true    — is this literally NaN? Yes.
```

**Always use `Number.isNaN`.** The global `isNaN` is a historical mistake kept for web compatibility.

Likewise: `Number.isFinite`, `Number.isInteger`, `Number.isSafeInteger` — all prefer the `Number.` namespace for the non-coercing versions.

## Numeric precision gotchas

```js
0.1 + 0.2 === 0.3             // false  (IEEE 754 — it's 0.30000000000000004)
0.1 + 0.2 === 0.30000000000000004   // true

Number.MAX_SAFE_INTEGER + 1 === Number.MAX_SAFE_INTEGER + 2   // true (!)
Number.MAX_SAFE_INTEGER       // 9007199254740991 = 2^53 - 1

1e-7 + 1 === 1                // false
1e-16 + 1 === 1               // true

Math.round(2.5)               // 3  (half away from zero)
Math.round(-2.5)              // -2 (not -3! this is asymmetric)
(-2.5).toFixed(0)             // "-3"
```

**For money, use integer cents (or `BigInt`), never `Number`.** For hashing-style equality checks on floats, use `Math.abs(a - b) < Number.EPSILON * Math.max(Math.abs(a), Math.abs(b))`, not `a === b`.

## String-to-number coercion quirks

```js
Number("")          // 0       (empty string → 0, not NaN!)
Number(" ")         // 0       (whitespace only → 0!)
Number("  42  ")    // 42      (trimmed)
Number("42px")      // NaN
Number("0x10")      // 16
Number("08")        // 8       (not octal since ES5)
Number(null)        // 0
Number(undefined)   // NaN
Number([])          // 0
Number([42])        // 42      (single-element array)
Number([1,2])       // NaN     (multi-element)
Number({})          // NaN

parseInt("42px")    // 42      (parseInt stops at first non-digit)
parseInt("0x10")    // 16
parseInt("08")      // 8       (in strict-spec engines; always pass radix!)
parseInt("08", 10)  // 8
parseInt("  42  ")  // 42
parseFloat("3.14px") // 3.14
```

**Rule: always pass the radix to `parseInt`.** `parseInt(str, 10)` is the only form to use.

## `+` ambiguity

`+` is string concat if *either* operand coerces to a string, otherwise numeric addition. This creates:

```js
1 + 2 + "3"      // "33"   (left-to-right: 3 + "3")
"1" + 2 + 3      // "123"  (left-to-right: "12" + 3)
[] + []          // ""     (both ToPrimitive → "")
[] + {}          // "[object Object]"
{} + []          // 0      (!) — leading { parsed as block, then +[] → 0
({} + [])        // "[object Object]" (parens force expression context)
```

The `{} + []` case is the most infamous: a line starting with `{` at statement position is parsed as a block, not an object literal. The "expression" is then `+[]` which is `0`.

## Truthy / falsy — the complete list of falsy values

`false`, `0`, `-0`, `0n`, `""`, `null`, `undefined`, `NaN`.

**Everything else is truthy**, including:
- `"0"` (non-empty string)
- `"false"` (non-empty string)
- `[]` (object)
- `{}` (object)
- `new Boolean(false)` (object — object wrappers are always truthy)
- `new Number(0)` (object)

The `[]` case is the most common source of bugs: `if (arr)` does *not* check that `arr` is non-empty, because `[]` is truthy. Use `if (arr.length)` or `if (arr.length > 0)`.

## `??` and `||` — not equivalent

```js
const port = userInput || 3000;    // uses 3000 if userInput is 0, "", false — probably a bug
const port = userInput ?? 3000;    // uses 3000 only if userInput is null/undefined — correct
```

`||` falls back on any falsy value; `??` (nullish coalescing) falls back only on `null`/`undefined`. For "default if not provided", always prefer `??`. `||` is correct only when you intentionally want "default if falsy" (rare — usually only for strings where `""` means "use default").

**Precedence trap:** `a ?? b || c` is a syntax error in ES2020+. You must parenthesize: `(a ?? b) || c` or `a ?? (b || c)`. This is a deliberate guard against misreading.

## Optional chaining `?.` — less obvious details

```js
a?.b              // undefined if a is null/undefined, else a.b
a?.()             // undefined if a is null/undefined, else a()
a?.[key]          // undefined if a is null/undefined, else a[key]
a?.b.c            // short-circuits AT a — if a is null, returns undefined; does NOT check b
a?.b?.c           // checks a AND b
a?.b = 5          // SYNTAX ERROR — cannot assign to optional chain
delete a?.b       // Valid; no-op if a is null
```

**Short-circuit depth:** `a?.b.c.d.e` short-circuits only at `a`. If `a` exists but `a.b` is `null`, you get `TypeError: Cannot read property 'c' of null`. Chain `?.` at every uncertain step.

## Template literal tag function edge cases

```js
function tag(strings, ...values) {
  strings.raw;   // array of raw (unescaped) strings — "\\n" not "\n"
  strings;       // array of cooked strings — "\n" is a newline char
}
tag`hello \n ${name}`;
```

`strings` is a "frozen array" — same identity across calls for the same tagged-template call site (useful for caching). `strings.raw` is the only built-in way to see the source characters without escape processing. This is how `String.raw` is implemented.
