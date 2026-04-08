---
name: graham-hackers-painters
description: Write exploratory, macro-aware code in Paul Graham's Hackers & Painters style. Use when building Lisp, Arc, Common Lisp, Scheme, REPL-first prototypes, embedded DSLs, terse internal tools, or when the user asks for bottom-up programming, brevity-as-power, macros, or Paul Graham style. Covers when to invent operators, when a utility or macro has earned its cost, how to keep code loadable in one brain, and when to prefer closures and data over OO scaffolding.
---

# Graham Hackers Painters⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​‌​‌​‌‍​​‌​​‌‌​‍​‌‌‌​​‌‌‍​‌‌‌‌‌​​‍​​​​‌​​‌‍‌​​​​​‌​⁠‍⁠

This skill is for code that should feel like sketching with a powerful language, not filling out a bureaucratic form. Optimize for leverage, mutation speed, and the ability to hold the live top layer in one head.

Do not use this skill for proof-oriented functional purity, ceremony-heavy enterprise APIs, or large-team standardization work. This is a leverage skill, not a safety-rail skill.

## Core Stance

- Treat the language as adjustable. The goal is not to express a finished idea faithfully; the goal is to discover a better idea while writing.
- Optimize for token reduction, not character golf. Brevity matters when it shortens the real path through the program.
- Prefer a larger language and a smaller program. Bottom-up work should produce a different program, not the same program in a different order.
- Judge readability at program scale. Per-line friendliness is often a trap if total program mass grows.
- Keep the active layer small enough to reload quickly. If a module no longer fits in one brain, delete layers, compress repeated forms, or split ownership.
- Treat inherited syntax as suspect. If a token survives only because people are used to it, it may be an onion. Remove it only after trying the simpler version in real code.
- Prefer abstractions whose best spec is source code. If a feature only makes sense as hidden compiler magic, be suspicious.

## Before Writing Code, Ask Yourself

- What is the smallest working slice I can sketch at the REPL before I commit to structure?
- What part of this code is real domain logic, and what part is bookkeeping the language should absorb?
- Is the repeated pattern actually stable, or have I only seen it once in disguise?
- Do I need a function, a macro, or a tiny language? The order is usually function -> utility -> macro -> DSL, not macro first.
- If I invent a new operator, will it make the top layer easier to hold in my head tomorrow morning?
- Is this still single-brain code? If multiple authors need to modify it independently, has the local cleverness turned into shared tax?

## Bottom-Up Procedure

1. Start with the smallest end-to-end slice that exposes the real data shape. Graham style begins with a sketch, not a spec.
2. Leave the first version ugly but runnable. Rewriting is part of the search process, not a cleanup pass.
3. After 2-3 real reuses, extract a utility. That is usually where a helper starts paying for itself.
4. Only promote a utility into a macro when you need one of Lisp's hard powers:
   - non-standard evaluation
   - new binding forms
   - source-to-source transformation that shifts work to read or compile time
5. Keep macro bodies thin. Put real work in ordinary functions and let the macro only arrange syntax, evaluation, or bindings.
6. If the abstraction starts reading like a mini-language, decide explicitly:
   - If host forms can express it by transformation, compile the DSL into host code.
   - If the shape is still unstable or must be inspected as data, keep it as data first and add syntax later.
7. Macroexpand the surface forms. If the expansion is noisy, the abstraction is not done yet.
8. When one person owns a fast-moving module, rewrite aggressively. When many people must touch it, freeze the clever surface earlier and make the boring interface explicit.

## Decision Rules That Experts Actually Use

- A utility usually pays for itself after 2-3 uses. A hairy macro may need closer to 10-20 uses before the readability savings beat the cognitive tax.
- If a macro exists mainly to save a function call, try an inline function first. That is usually the cheaper bet unless the compiler cannot help you.
- If an embedded language can be implemented by transformation instead of interpretation, transform it. You get less code, better speed, and reuse the host compiler instead of rebuilding its work.
- Ask what can be paid once at read time or compile time instead of at every run. Graham style treats read time, compile time, and runtime as one continuum.
- Intentional variable capture is a power tool, not a default. Use it only when the captured name is the whole point of the abstraction and the calling context is short and local.
- Shorten frequent operators, not rare domain nouns. Arc-style terseness works when the compressed token appears everywhere; compressing business terms just destroys searchability.
- Keep local macros with the code they shape. Extract only the genuinely reusable ones; a central macro graveyard makes programs harder to read.
- If morning reload time keeps exceeding about 30 minutes, the working set is too large. Shorten the top layer or isolate the experiment.
- Protect deep work blocks. The wrong interruption can wipe context in 30 seconds; the fixed cost to reload it is much larger.

## NEVER

- NEVER write a macro just because repetition offends you, because the first repeated shape is often a false pattern. Instead wait for 2-3 real uses or one edge case that reveals the true abstraction.
- NEVER replace functions with macros just to look Lispy, because macro definitions are harder to read and debug than equivalent functions. Instead default to functions and spend macro budget only on evaluation control, bindings, or compile-time translation.
- NEVER trust a weird temporary variable name to avoid capture, because nested expansions will eventually collide in ways that are hard to reproduce. Instead `gensym` internal bindings and reserve deliberate capture for local anaphoric-style forms.
- NEVER centralize all macros in one file because they are macros, because separating the local language from the code it shapes makes the program harder to read. Instead keep domain-local macros beside their callers and extract only general-purpose utilities.
- NEVER reach for class and protocol scaffolding when a closure, table, or list will do, because OO ceremony is seductive precisely when you want code to look substantial. Instead use data and closures until you truly need multi-author extension boundaries.
- NEVER ship an interpreter-shaped DSL when a transform-shaped DSL would suffice, because you will reimplement the host language badly and pay the cost on every execution. Instead lower DSL forms into host code and let the existing compiler and runtime do the heavy lifting.
- NEVER confuse per-line readability with total readability, because verbose code can feel friendly while making the whole program impossible to hold in one head. Instead minimize total program mass and accept some conceptual density where it deletes bookkeeping.
- NEVER freeze the full architecture before the first working slice, because early on the thing you most need to change is the problem itself. Instead prototype a narrow subset, learn the real shape, then rebuild around what survived contact with reality.
- NEVER preserve syntax or ceremony only because it is traditional, because legacy onions survive long after their original purpose disappears. Instead test whether the simpler surface loses real expressive power before you keep it.

## When The Host Language Is Not Lisp

- Use higher-order functions, closures, code generation, parser combinators, or build-time transforms as substitutes for macros.
- Keep the "invent the operator" instinct, but express it through the host language's real metaprogramming hooks instead of imitating Lisp syntax badly.
- If the language fights runtime mutation or macro-like transforms, bias toward data-first DSLs and aggressive helper extraction rather than fake cleverness.
- If the host language makes compile-time tricks brittle, stop one layer earlier: reusable functions plus data often beats a fragile pseudo-macro system.

## Edge Cases And Fallbacks

- If a macro is hard to debug, split it into a pure expansion function plus a thin macro wrapper. That keeps the language layer testable.
- If other engineers cannot predict what a macro expands into, the abstraction has crossed the line from power to private language. Rename it, shrink it, or demote it to a function.
- If performance is the only justification for cleverness, measure first. Graham style values programmer-time first, but it does not excuse hand-wavy speed claims.
- If the code must survive broad team ownership, preserve the bottom-up insight in the internals but present a plainer public API. Graham style is strongest at the frontier; it is not a license to make maintenance theatrical.

## Output Style

When applying this skill:

- delete scaffolding before adding framework
- name the domain primitives after the concepts, not after the implementation
- show the top layer first, then the machinery underneath only if needed
- prefer one sharp abstraction over five managerial ones
- sound like a builder who discovered the design by working, not a committee that approved it
