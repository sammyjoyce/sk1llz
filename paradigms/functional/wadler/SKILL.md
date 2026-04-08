---
name: wadler-monadic-elegance
description: Write Haskell (or any typed functional code) the way Philip Wadler would — type-driven, parametricity-aware, choosing the weakest abstraction that fits, and avoiding the well-known landmines (WriterT space leaks, StateT/ExceptT order semantics, seq breaking free theorems, MonadFail silent failures). Use when designing a new effect API, picking between Functor/Applicative/Selective/Monad/effect-system, debugging a space leak in an mtl stack, refactoring `IO`-soaked code into something testable, or deciding between mtl, effectful, polysemy, free monads, and tagless final. Trigger keywords: monad, monad transformer, mtl, effectful, polysemy, free monad, tagless final, WriterT, StateT, ExceptT, MaybeT, MonadFail, applicative, selective functor, parametricity, free theorem, propositions as types, type-driven, Haskell space leak, ReaderT IO pattern.
---

# Wadler-Style Functional Design

A senior Haskeller's checklist. Skip the textbook material — Claude already knows what a monad is, the laws, and `do`-notation. This file is the things that took ten years and several production outages to learn.

## The Wadler Question (ask before every type signature)

Before writing any function, work the type top-down:

1. **What is the weakest abstraction that admits this implementation?** Functor < Applicative < Selective < Monad < `IO`. Picking a stronger class than needed throws away static analysis, parallelism, and free theorems for nothing.
2. **How many total implementations does this type permit?** If the answer is "many," your type is too weak — add parametricity or refine. `forall a. a -> a -> a` has exactly two total implementations; `Int -> Int -> Int` has infinity.
3. **Can I make the illegal state unrepresentable instead of validating it?** A phantom-typed `Connection 'Open` that only `read` accepts is a Curry-Howard proof, not a runtime check.
4. **If this type is polymorphic, what free theorem does it give me?** `reverse :: [a] -> [a]` for free gives `map f . reverse = reverse . map f`. If you can't name the free theorem, the type variable is doing nothing.

## Decision tree: which abstraction?

| Symptom | Right tool | Why not the next one up |
|---|---|---|
| Pure value transformation, no effects | `Functor` (`fmap`) | `Applicative` adds nothing |
| Multiple independent effects, want `traverse` to parallelize | `Applicative` | `Monad` forbids parallelization (later effects may depend on earlier results) |
| Conditional effect, but you can list **all possible branches up front** (build systems, parsers, form validation) | `Selective` (Mokhov 2019) | `Monad` makes the dependency graph un-analyzable; Selective lets you statically enumerate all reachable effects |
| Effect choice depends on a runtime value's contents | `Monad` | You actually need `>>=`; nothing weaker works |
| Need to combine ≥3 distinct effects in production code | **`ReaderT Env IO` with `IORef`/`MVar` fields**, NOT `StateT s (ExceptT e (ReaderT r IO))` | mtl stacks pay a continuation-passing tax per layer; the ReaderT-IO pattern is what large Haskell shops actually ship |
| Need to combine effects AND want pure tests | `effectful` or `cleff` (evidence-passing effect systems) | Free-monad libraries (`polysemy`, `freer-simple`) build a runtime program tree and walk it — measurably slower, often by 5–50× |
| Need nondeterminism + `bracket`/`fork` in the same code | You can't have both. Pick: `eff`/`speff` for nondeterminism, `effectful`/`cleff` for `MonadUnliftIO`. This is a fundamental design split, not a missing feature. |

## NEVER list (each costs a production outage)

- **NEVER use `Control.Monad.Writer` (any version) for accumulating logs or counters.** It is seductive because the type screams "append-only". The reality: `WriterT`'s `>>=` must hold both sub-results before calling `mappend`, so a million `tell`s build a million-deep thunk before any combination happens. Even `Strict.WriterT` is strict in the *monad*, not the accumulator — the accumulator is unreachable from outside, so you can't even bang-pattern your way out. **Instead:** use `Control.Monad.Trans.Accum` (lets you `look` at the running accumulator, which forces strictness), or `StateT s` with a strict record + `modify'`, or just an `IORef` in `ReaderT IO`.

- **NEVER assume `Control.Monad.State.Strict` is actually strict.** It is strict in the *monadic bind*, not the state value. `runState (do modify (+1); modify (+1)) 0` builds a thunk `(0+1)+1`. Strict StateT requires **all three** of: `{-# LANGUAGE StrictData #-}` on the state record, `modify'` (not `modify`), and `put $! x` (not `put x`) when assigning. Miss any one and you leak.

- **NEVER pick `StateT s (ExceptT e m)` and `ExceptT e (StateT s m)` by coin flip.** They have different semantics that bite in catch handlers:
  - `StateT s (ExceptT e m)` desugars to `s -> m (Either e (a, s))`. State **vanishes** on `throwError` — you cannot read the state in your handler. This is the right choice for transactional rollback (parser backtracking, search).
  - `ExceptT e (StateT s m)` desugars to `s -> m (Either e a, s)`. State **survives** the throw — handlers see whatever was last `put`. This matches imperative-language exception semantics.
  - Pick the wrong order and `catchError` will silently see the wrong state.

- **NEVER use polymorphic `seq` without realizing it weakens free theorems.** Wadler's parametricity assumes no `seq`. In a Haskell with `seq`, `forall a. a -> a` is no longer guaranteed to be `id` — `\x -> x \`seq\` x` and `\x -> x \`seq\` undefined` have the same type. Free theorems still hold *up to bottom*, which is why GHC's rewrite rules sometimes invalidate optimizations that look obviously sound. If you depend on a free theorem for correctness (not just intuition), the type must be in a `seq`-free fragment or you must prove it manually.

- **NEVER write `do Just x <- m` in a `Monad` you don't fully understand.** This desugars to a call to `fail`, whose behavior is monad-specific: in `Maybe` it returns `Nothing` (silently swallowing the pattern failure), in `IO` it throws a runtime exception, in `STM` it `retry`s, in `[]` it returns `[]`. Same syntax, four wildly different runtime behaviors. **Instead:** use explicit `case` or `MonadFail`-aware code, and add `{-# LANGUAGE NoMonadFailDesugaring #-}` if you can.

- **NEVER reach for `Monad` when `Applicative` works.** It looks innocuous but it costs you: `traverse` can no longer parallelize, `Const`-based static analysis stops working, your type has more inhabitants so refactors are harder to verify. Specifically, if you find yourself writing `do { x <- foo; bar }` where `bar` doesn't reference `x`, you wanted `foo *> bar`.

- **NEVER pick `polysemy` for greenfield production code in 2024+.** It is seductive because the API is the cleanest of any effect library. Reality: it builds a runtime tree of effect operations and traverses it on every bind, costing an order of magnitude vs. `effectful`/`cleff`. Use it for prototypes and DSLs only. **Instead:** start with the `ReaderT IO` pattern and graduate to `effectful` when you need typed effect tracking.

- **NEVER add a new monad transformer "to be principled."** Each layer adds CPS overhead and a `lift` that breaks `MonadUnliftIO` for things like `bracket`, `withFile`, async/forkIO. The cost compounds: a 4-deep stack is roughly 16× slower than `IO` for tight loops. **Instead:** flatten to `ReaderT Env IO` where `Env` carries `IORef`s for state and a `Logger` handle for output.

- **NEVER use `liftIO` deep in business logic.** It is the smell of a missing abstraction. If a function is in `IO`, it can do anything and tests can do nothing. **Instead:** parameterize over a small effect (`MonadReader Logger m`, `MonadDB m`) or use a tagless-final algebra so you can run it in `Identity` for tests.

## Hard-won knowledge

### Type-driven implementation, in practice

Start at the type, derive the body. The trick: at each step, ask "what *can't* this expression do?" The smaller the answer, the closer you are. For a polymorphic `f :: forall a. (a -> a) -> a -> a`, the only useful structure is "apply the function some natural number of times" — there are countably many implementations. For `f :: forall a b. (a -> b) -> [a] -> [b]`, by parametricity it must be `map`-shaped (or always return `[]`). When you finish writing a function and notice "I could have written this several genuinely different ways and they'd all type-check," your type is too weak — add a constraint or a phantom parameter.

### Free theorem as a debugging tool

When optimizing, the free theorem of the type tells you what rewrites are sound *without proof*. `map f . reverse ≡ reverse . map f` lets you fuse passes. But: this assumes `f` is total. If `f` can be `undefined` or use `seq`, the equation only holds up to ⊥. GHC's `RULES` pragmas exploit free theorems — when you write your own `RULES`, ask "does this rule survive the introduction of `seq`?" If not, gate it on a non-bottom type.

### Tagless final vs free monad — the actual rule

Both let you defer commitment to a concrete monad. They are not interchangeable:

- **Tagless final** (an `mtl`-style typeclass per algebra): less boilerplate, runs directly in the target monad, easy to combine algebras. Stack safety inherits from the target. Pattern matching on the program is hard. **Use for** the bulk of business logic where you mostly want to swap interpreters for tests.
- **Free monad** (an ADT per operation, interpreted by `foldFree`): the program is reified as data, so you can pattern-match it for batching, caching, transactions, optimization. Stack-safe by construction. Boilerplate to combine algebras (Coproducts + `Inject`). **Use for** cross-cutting concerns where you genuinely benefit from inspecting the program: database transaction coalescing (Slick's `DBIOAction`), build system dependency analysis (Shake), query optimization.

A common pattern: tagless-final at the application boundary, compiled down into a free-monad DSL when you cross into the optimization-friendly subsystem.

### `Identity` is the test oracle

The benefit of writing code over an abstract `m` is not philosophical purity — it is that `runIdentity :: Identity a -> a` (or `runWriter`, `runState`) lets you test pure semantics without any IO. If your "tagless final" code can only be run in `IO`, your effects are leaking abstractions. Concretely: any operation whose interpreter must call `getCurrentTime`, `randomIO`, or `forkIO` should be its own algebra, not buried inside a generic `MonadIO m =>` constraint.

## When to load reference files

Most tasks need only this file. Load deeper material **only** when the trigger applies:

- **Debugging a space leak, deciding transformer order, or seeing a `WriterT` in code** → MANDATORY: read [`references/transformer-traps.md`](references/transformer-traps.md) end-to-end before proposing a fix. It contains the exact desugaring of all four common stacks and the rewrite rules to flatten them.
- **Designing a new effect API or library, or choosing between Functor/Applicative/Selective/Monad** → MANDATORY: read [`references/abstraction-ladder.md`](references/abstraction-ladder.md). It has runnable code for each rung and the laws each level must satisfy.
- **Routine bug fix in existing well-typed code** → do NOT load either reference. The decision tree above is sufficient.

## Signature Wadler moves

- Always pick the weakest typeclass that compiles.
- Make illegal states unrepresentable with phantom types and GADTs before adding runtime checks.
- Treat every polymorphic type as a free theorem; name it before optimizing around it.
- Default to `ReaderT Env IO` for production; reach for an effect system only when you need typed effect tracking and have measured the cost.
- When in doubt about transformer order, write out the desugared `s -> m (Either e (a, s))` form on paper.
