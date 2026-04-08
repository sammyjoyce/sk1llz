---
name: wadler-monadic-elegance
description: Decide how far to escalate from parametric polymorphism to Applicative, Selective, Monad, transformers, or effect handlers in Haskell and similar typed FP systems, with emphasis on rollback, concurrency, and performance traps. Use when designing or refactoring effectful code, choosing between ReaderT/StateT/WriterT/ExceptT, diagnosing transformer-order bugs, recovering lost Applicative structure, or evaluating free/freer/effect-system trade-offs. Trigger keywords: monad, applicative, selective, applicativedo, ReaderT, StateT, WriterT, ExceptT, MonadFail, free monad, Codensity, effect handler, polysemy, fused-effects, effectful.
---

# Wadler: Spend Algebraic Power Carefully

The central habit is not "use monads elegantly." It is "spend as little semantic power as possible."

Every extra constraint destroys something:
- `Functor` to `Applicative`: you lose nothing about branch independence.
- `Applicative` to `Selective`: you admit value-dependent choice, but the branch set stays statically visible.
- `Selective` to `Monad`: you give up static shape, many free theorems, and optimizer freedom.
- Direct style to reified effects: you buy inspection and rewriting, but you now pay an interpreter cost model.

If a signature can honestly stay weaker, keep it weaker. That preserves laws, testing surface, refactoring freedom, and often performance.

## Before adding power, ask yourself

Before changing a type from `Applicative` to `Monad`, ask:
- Is there a real value dependency, or did syntax create a fake one?
- Are all possible branches known up front? If yes, try `Selective` before `Monad`.
- Did a failable or strict pattern in `do` force `>>=` even though the computation is structurally applicative?
- Am I spending proof power for convenience alone?

Before choosing a transformer stack, ask:
- On failure, should state disappear, survive for auditing, or survive only at selected boundaries?
- Under async exceptions or `concurrently`, does the state need to remain visible?
- Do I need to inspect or rewrite the program before running it, or only execute it?
- Is the hot path dispatch-heavy, or will I/O dominate enough that interpreter overhead is irrelevant?

Before introducing a new effect abstraction, ask:
- Is this exposing semantics, or just hiding plumbing?
- Will the next person be able to state the failure contract without mentally expanding the whole stack?
- Can I write the fully unwrapped type? If not, I do not understand the design yet.

## Mandatory decision procedure

Before touching any `StateT`/`ExceptT`/`WriterT` combination, write the unwrapped type on paper or in comments:
- `StateT s (ExceptT e m) a` means `s -> m (Either e (a, s))`
- `ExceptT e (StateT s m) a` means `s -> m (Either e a, s)`

Choose the one whose failure contract matches reality. Do not choose by aesthetics or library habit.

Before "upgrading" to `Monad`, test whether `ApplicativeDo` is being blocked by syntax:
- A strict or failable pattern in `do` forces `>>=` to preserve strictness.
- GHC only recognizes the final line as applicative if it is literally `return E`, `return $ E`, `pure E`, or `pure $ E`.
- `-foptimal-applicative-do` finds better splits, but the algorithm is `O(n^3)` and is a bad default for generated `do` blocks once you get into roughly 100+ statements.

If those are the blockers, rewrite the syntax, not the abstraction.

## Decision tree

- Need only independent effectful arguments and static structure matters:
  Use `Applicative`. This keeps parallelization, static analysis, and stronger free theorems available.

- Need runtime choice, but the set of branches is known ahead of time:
  Use `Selective`. This is the sweet spot people skip because `Monad` is familiar.

- Need to synthesize the next effect from a runtime value:
  Use `Monad`, but only for that boundary. Keep outer APIs weaker when you can.

- Need application wiring, resource acquisition, logging sinks, shared mutable cells, async exceptions, or concurrency:
  Default to `ReaderT Env IO` or a Reader-like effect system whose operational story is just as explicit.

- Need rollback semantics inside pure or bounded logic:
  Use local `StateT`/`ExceptT`/`WriterT`, but make the failure contract explicit by expansion first.

- Need to inspect, optimize, reorder, or reinterpret programs:
  Reify with free/freer/algebraic effects only at that seam. Do not drag a reified IR through the whole hot path.

## NEVER do these

- NEVER add a `Monad` constraint because `do` notation reads better. That is seductive because the code stops fighting you immediately. The hidden cost is lost static structure, weaker theorems, and fewer optimization opportunities. Instead first check for `ApplicativeDo` blockers such as strict patterns or a final line GHC cannot recognize, then ask whether `Selective` is enough.

- NEVER recommend `-Wmissing-monadfail-instances` as protection against bad `do` patterns. Old posts mention it, so it feels like institutional wisdom. Since GHC 8.8 it has no effect, which means you think you installed a guardrail when you did not. Instead ban failable patterns in polymorphic `do`, use explicit `case`, and rely on normal exhaustiveness warnings.

- NEVER put `ExceptT e` on top of `IO` for application-core error semantics because `IO` can still throw anything at any time. It is seductive because the type looks "documented." The consequence is a false failure model and undefined team expectations around cancellation and `concurrently`. Instead keep `Either` in pure/domain layers and convert to exceptions or boundary errors at the application shell.

- NEVER use `StateT` for shared application state because it looks pure and linear in single-threaded examples. Under exceptions you lose the threaded state, and under concurrency the semantics are cloning-plus-arbitrary-survivor, not shared mutation. With `put 4 >> concurrently (modify (+1)) (modify (+2)) >> get`, plausible outcomes are `4`, `5`, or `6`, not `7`. Instead move shared state into `IORef`/`TVar` fields inside `Env` and make the mutation story explicit.

- NEVER default to `WriterT` for production logging because "strict" sounds like it fixed the classic issue. The seductive part is the pleasant API. The consequence is retained thunks, memory growth, and poor failure visibility exactly where logs are supposed to help you. Instead use explicit logger handles or mutable accumulators in `ReaderT`; reserve `WriterT` for small, bounded, morally pure builders that you have benchmarked.

- NEVER choose transformer order by habit. `StateT s (ExceptT e m)` and `ExceptT e (StateT s m)` expose different business truths about rollback and auditability even when their surface code looks similar. Instead expand both to the unwrapped type and choose the one that matches retry, compensation, and debugging requirements.

- NEVER assume `MonadUnliftIO`, `MonadBaseControl`, lifted `bracket`, or lifted concurrency are semantics-preserving on stateful stacks. This is seductive because the code compiles and simple tests pass. The consequence is cleanup running with stale state or state updates disappearing across exception boundaries. Instead unlift only through Reader/Identity-style stacks with no monadic state, or move mutable state into explicit refs that survive exceptions.

- NEVER reach for free or freer effects because handler syntax looks elegant. The hidden trap is cost: left-associated binds over free structures are the classic quadratic failure mode, and "the interpreter is pure" does not change that. Instead keep direct monads in hot paths, and if reification is truly required, use Codensity/fusion techniques or lower to a concrete carrier early.

## Cost-model heuristics experts use

- Treat every extra transformer in a polymorphic `mtl`-style stack as part of the bind cost model. In dispatch-heavy code, deep stacks can become materially slower because each bind walks layer by layer.
- A published `effectful` benchmark on GHC 9.2.4 / Ryzen 9 5950X found the deep `mtl` countdown case about 50x slower than the `ST` baseline, precisely because bind dispatch became the hot loop. The same benchmark showed that once real I/O entered the picture, the gap narrowed sharply.
- The heuristic is therefore: benchmark dispatch when dispatch is the work; benchmark end-to-end when I/O dominates. Do not generalize from one regime to the other.
- `WriterT` is not forbidden everywhere. It can be the right answer in a bounded builder with no concurrency, no need to survive exceptions, and measured evidence that alternatives are worse.
- Free/freer interpreters are not forbidden everywhere. They are justified when the program shape itself is an asset: optimization passes, alternate backends, effect elimination, or static inspection.

## Practical review patterns

When reviewing an API, ask:
- Could this signature be weakened from `Monad` to `Applicative` or `Selective` without lying?
- Did someone add `MonadIO` merely to reach logging, randomness, or time? If so, a narrower capability or environment handle is probably the better boundary.
- Did a refactor make a function "more convenient" by adding constraints? If yes, ask what free theorem just got spent.

When debugging a stack bug, do this in order:
1. Expand the stack to unwrapped types.
2. Mark every place where failure, cancellation, or `catch` can happen.
3. Decide whether state/logs/resources should be visible after each failure site.
4. Only then change transformer order or carrier choice.

When a free/freer system feels slow, do this in order:
1. Check whether the hot path is interpreter dispatch or real I/O.
2. Look for left-associated bind growth and repeated reinterpretation.
3. If the program shape is no longer being exploited, collapse back to direct style.

## Edge-case reminders

- If you care about preserving applicative structure, avoid failable patterns in `do`; pattern-match after the fact.
- If you need shared state across threads, `StateT` is the wrong story even when the type is shorter.
- If your logs matter during failure investigation, do not hide them in a transformer that disappears on exceptions.
- If you cannot explain the cancellation behavior of your error model, the model is not ready for concurrent code.

## Do not use this skill for

- Beginner explanations of what monads, functors, or monad laws are.
- Local syntax fixes where no abstraction or failure-semantics decision is being made.
- Category-theory exposition detached from code-shape, runtime behavior, or API design.

Use this skill when the question is not "how do I write the code?" but "what semantic power am I willing to spend, and what will that cost me later?"
