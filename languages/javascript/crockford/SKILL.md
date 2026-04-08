---
name: crockford-good-parts
description: >-
  Write or refactor JavaScript in Douglas Crockford's later defensive subset:
  class-free, this-free, semicolon-full, JSLint-clean code for long-lived,
  browser-exposed, JSON-heavy, or security-sensitive modules. Use when auditing
  or rewriting code with coercion, ASI, prototype leakage, ambient authority,
  frozen-object, `for...in`, `eval`, or unsafe-number bugs, and when the user
  mentions Crockford, Good Parts, JSLint, ADsafe, capability security, frozen
  factories, or class-free JavaScript.
---

# Crockford After The Book

Treat Crockford as a risk-control doctrine, not a nostalgia act. The point is not to look old-fashioned; the point is to stay out of the parts of JavaScript where parser ambiguity, capability leakage, prototype tricks, and silent coercion hide bugs for years.

This skill is strongest for code that crosses trust boundaries, gets embedded in hostile pages, consumes attacker-controlled JSON, or must remain understandable after frameworks churn. It is too strict for framework-owned glue unless you deliberately confine the non-Crockford code to a thin adapter.

## Pick The Lane First

| Situation | Default | Why | Allowed deviation |
|---|---|---|---|
| Browser widget, third-party embed, sandbox, plugin host, JSON boundary | Full Crockford subset | Ambient authority and prototype surprises are the real enemy | None without a written reason |
| Legacy class/framework integration | Quarantine `this`/`class` in an adapter; keep domain logic Crockford-clean | Rewriting framework-owned surfaces is churn, not risk reduction | Thin outer shim may use framework conventions |
| Performance-critical hot loop | Keep the semantic bans, relax `freeze` only if profiling proves object creation is the bottleneck | `Object.freeze` and per-instance closures cost real throughput | Prefer typed arrays or data-oriented structures before reintroducing classes |
| Routine app code with no trust boundary | Keep the hard bans (`eval`, `==`, ASI reliance, boxed primitives, unsafe numbers) | Those bugs stay expensive even in internal code | Full class-free style is optional if it fights the framework |

## Before You Touch Code, Ask Yourself

- **Authority:** Does this function really need a global, clock, random source, logger, DOM handle, or service singleton, or should that capability be passed in explicitly?
- **Prototype behavior:** If `Object.prototype` is polluted, or the value comes from another realm/iframe, will this code still behave the same?
- **Parser ambiguity:** Would a newline, concatenation boundary, or formatter rewrite change the parse?
- **Value model:** Is this value truly arithmetic, or is it an opaque identifier that must never become a `Number`?
- **API shape:** Am I exposing a message-oriented object, or just leaking a mutable property bag with getters/setters?

## What Modern Crockford Actually Means

- Later Crockford is more anti-`this` and more anti-inheritance than the 2008 book. The security reason matters: detached methods and implicit receivers leak authority in ways closures do not.
- `Object.freeze` is only loud in strict code. In sloppy mode, writes to frozen properties can fail silently. If the file is not an ES module, strictness is part of the pattern, not a garnish.
- Prototypes and freezing do not mix well. Using `Object.create(proto)` as a "cheap copy" looks elegant until `proto` is frozen: writes can throw, and inserting a new property forces ancestor checks up the entire chain.
- `Object.create(null)` is for dictionaries, not ordinary records. It avoids prototype leakage and the frozen-prototype insertion scan, but it also removes conveniences like `toString`, `instanceof Object`, and `obj.hasOwnProperty`.
- Treat JSON text and arbitrary objects differently. Copying a general object with spread or `Object.assign` runs getters; copying `JSON.parse` output does not. The shortcut is safe for parsed JSON, not for arbitrary host objects.
- Crockford-style public APIs should be verbs, not field toggles. If callers mostly call `set_x`, `set_y`, `set_z`, you have exposed representation, not behavior.

## Non-Obvious Heuristics

- Prefer a lowercase factory that returns `Object.freeze({...})`. Capitalized names imply `new`; lowercase is a defense against accidental constructor calls.
- For attacker-controlled string keys, choose `Map` when key identity matters, choose `Object.create(null)` when you need JSON-like serialization, and choose a plain object only when the key set is trusted.
- Use `typeof` only for primitive-ish checks: `"undefined"`, `"string"`, `"number"`, `"boolean"`, `"function"`. For everything else, assume `typeof` is trying to trick you.
- Treat all values above `2^53 - 1` as already corrupt if they passed through `Number`. Database IDs, snowflakes, and nanosecond timestamps belong in strings or `BigInt`, not doubles.
- If a line begins with `(` or `[`, assume concatenation can misparse it unless the previous statement is terminated. Defensive leading semicolons are a parser guard, not a style tic.

## Never Trade Clarity For Cleverness

- **NEVER use direct `eval`, `new Function`, or string-based `setTimeout`/`setInterval`** because the seductive shortcut is "I can interpret this little DSL later", but the real consequence is caller-scope access, CSP breakage, disabled inlining, and runtime name lookups. Instead parse data, dispatch on an allowlist, or pass explicit capabilities into a predeclared function.
- **NEVER traverse records with bare `for...in`** because the seductive part is the one-line loop, but the consequence is inherited keys, prototype-pollution surprises, and guards that explode when data contains its own `hasOwnProperty`. Instead use `Object.keys`, `Object.entries`, `Map`, or `Object.create(null)` plus explicit copying.
- **NEVER build "immutable copies" by inheriting from frozen prototypes** because it feels cheaper than copying, but the consequence is write exceptions and slower property insertion due to ancestor scans. Instead copy own data into a fresh object and freeze that result.
- **NEVER store money or opaque IDs in `Number`** because one numeric type looks convenient, but the consequence is silent rounding for decimals and false equality above `9007199254740991`. Instead use minor units or `BigInt` for arithmetic, and strings for identifiers.
- **NEVER rely on automatic semicolon insertion** because formatters and line wraps make it look harmless, but the consequence is `return`-newline-object bugs, accidental call continuations, and hard-to-see parse changes at bundle boundaries. Instead terminate every statement and keep the returned/thrown expression on the same line as the keyword.
- **NEVER put function expressions in loops or block-scoped function statements in lint-clean code** because the inline callback feels local and tidy, but the consequence is JSLint rejection, closure capture mistakes, and anonymous stack traces. Instead declare helpers outside the loop or bind the current value explicitly before creating the function.
- **NEVER use boxed primitives or "generic object checks"** because `new Boolean(false)` and `typeof x === "object"` feel object-oriented, but the consequence is truthy false values, `typeof null === "object"`, arrays passing as objects, and `NaN` passing as a number. Instead use literals, `value === null`, `Array.isArray`, `Number.isNaN`, and `Number.isFinite`.
- **NEVER give business logic ambient access to `Date.now`, `Math.random`, globals, or mutable service singletons** because it is faster to code once than to inject capabilities, but the consequence is non-reproducible tests, hidden nondeterminism, and authority leaks in sandboxes. Instead pass clock, RNG, storage, and I/O in explicitly.

## JSLint Rules That Still Surprise Experienced Developers

- JSLint expects expression statements to be assignments or calls. A stray object literal, ternary, or comma expression in statement position is treated as a bug, not an aesthetic choice.
- JSLint accepts function statements at file/function-body scope, not inside blocks. This matters because block function semantics were historically divergent across engines.
- JSLint allows arrow functions only in the expression-body form when you are chasing strict lint cleanliness; block-bodied arrows are rejected to avoid ambiguity.
- JSLint treats `+ +x`, `a+++b`, and similar plus/minus adjacency as bug magnets. If numeric coercion is intended, write `Number(x)` or add parentheses.
- JSLint distrusts `for` itself, not just `for...in`. If the loop body is a collection transform, expect the more Crockford answer to be `forEach`, `map`, `reduce`, or a purpose-built helper.

## Operating Procedure

1. Classify the code path: boundary, adapter, hot path, or routine app code.
2. Remove ambient authority first. Pass dependencies in before touching syntax.
3. Choose the object model deliberately:
   - Message-oriented frozen factory for most modules.
   - `Map` or null-prototype dictionary for attacker-controlled keys.
   - Typed arrays or data tables before classes for hot paths.
4. Eliminate parser and value traps before style cleanup:
   - `==`, ASI, `for...in`, boxed primitives, `Number` IDs, string eval.
5. Only then normalize the surface shape: lowercase factories, explicit semicolons, no `this`, no inheritance-driven reuse.

## Loading Triggers

**MANDATORY:** Before designing or refactoring sandboxed code, capability-based APIs, or any module where the question is "can this code be given less power?", read [`references/philosophy.md`](references/philosophy.md) for the ADsafe and POLA context behind the subset.

**Do NOT load** [`references/philosophy.md`](references/philosophy.md) for routine one-file cleanups, equality fixes, ASI fixes, or frozen-factory refactors. The checklist in this file is enough.

**MANDATORY:** Before claiming a file is Crockford-clean, run:

```bash
node languages/javascript/crockford/scripts/jslint_check.js path/to/file.js
```

Treat the bundled checker as a preflight, not the final judge. It does not catch restricted-production ASI traps, function-in-loop issues, or the stricter arrow/function-position rules from upstream JSLint.

**If the change touches parser-sensitive code, loop-created closures, sandboxing, or security boundaries:** also run upstream JSLint if it is available in the environment. A green result from the local script alone is not enough for those cases.
