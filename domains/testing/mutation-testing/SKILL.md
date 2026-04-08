---
name: lipton-mutation-testing
description: Separate productive mutants from noise so mutation testing improves tests instead of gaming a score. Use when tuning PIT or Stryker runs, triaging survivors, timeouts, static mutants, or NoCoverage results, setting PR or CI mutation gates, or deciding whether a mutant is equivalent, arid, or worth a new test. Trigger keywords: mutation testing, mutation score, PIT, Stryker, survivor triage, equivalent mutant, arid code, static mutant, timeoutFactor, incremental, NoCoverage.
tags: mutation-testing, test-quality, fault-injection, coverage-analysis, CI-gates
---

# Richard Lipton Mutation Testing

## Start Here
- Before changing any knob, read the repo's active mutation config (`pom.xml`, `build.gradle`, `stryker.config.*`, CI job) and compare it to the failing run. Mutation failures are often selection/config failures, not assertion failures.
- If the immediate problem is a surviving mutant, jump to `Survivor Triage`. If it is runtime blow-up or hangs, jump to `Timeout Triage`. If the argument is about thresholds or CI, jump to `Gate-Setting Rules`.
- Do NOT load generic mutation-testing primers for live triage. They explain the idea, but they do not tell you why incremental reuse lies, why static mutants explode runtime, or why logging mutants create brittle change-detector tests.

## Mindset
Lipton's idea only pays off when the surfaced mutant is more useful than the test you would write to kill it. Google's large-scale mutation service had to suppress "unproductive mutants" aggressively; otherwise the "not useful" rate stayed around 80%. After arid-code suppression and line-level limiting, it dropped to roughly 15%, which is the right order of magnitude to reach before tightening gates.

Before doing anything, ask yourself:
- Is this mutant coupled to behavior we actually promise, or only to logging, tuning constants, cache warmth, or mock-insulated plumbing?
- If I kill this mutant, will I strengthen an invariant, or will I create a brittle change-detector test?
- Is the run telling the truth, or is selection logic hiding tests because of static initialization, stale incremental data, mixed mutants, or compiler-generated duplicates?
- Is the expensive problem "bad assertions" or "bad mutant choice"? Fix the latter first.

## Choose The Run Shape First
| Situation | Primary move | Why | Fallback |
|------|-------------|-----|----------|
| Small risky PR | Mutate changed lines/files only, preferably one mutant per changed line | Google found line-level sampling was enough in more than 90% of lines once noisy mutants were suppressed | Expand to the surrounding package only if survivors cluster |
| Reviewer-facing rollout | Cap surfaced mutants to about 5-7 per file | Beyond that, humans stop reviewing findings and start skimming | Split the run by package or risk area |
| New module or shaky suite | Run one focused full-module pass before adding gates | Global score on unstable infra teaches nothing | Keep reports informational until two stable runs agree |
| CI runtime explodes | Constrain `targetClasses`/`targetTests` or `--mutate`, then inspect static mutants | Broad test discovery is the fastest way to turn mutation into a queueing problem | Use incremental only after a clean dry-run baseline |
| Suspect equivalent/noise mutants | Improve suppression/exclusions before adding tests | Thresholds cannot compensate for arid code | Mark as equivalent/unproductive with rationale and revisit later |

## Heuristics Experts Use
- Keep `targetTests` explicit in PIT. If you let PIT infer from a broad classpath, runtime expands unpredictably and the signal-to-noise ratio collapses before score quality does.
- Treat `NoCoverage` as a routing failure, not a near miss. It means the mutant never reached a relevant test, so improving assertions first is usually the wrong move.
- For PIT timeouts, separate fixed overhead from proportional slowdown. Defaults are `timeoutFactor=1.25` and `timeoutConstant=4000ms`. Increase the constant first when only first-touch mutants fail after classloading or JIT warmup; increase the factor when the whole suite slows proportionally under mutation.
- Use PIT's default logging exclusions unless logging or audit output is itself contractual. Disabling `FLOGCALL` makes PIT mutate log lines again, which often produces survivors that reward brittle tests instead of stronger behavior checks.
- If PIT reports confusing duplicates around `finally` blocks or single-line concatenations, suspect bytecode inlining, not missing assertions. `detectInlinedCode` helps, but PIT deliberately stays conservative when it cannot distinguish genuine duplicates from compiler copies.
- If one oversized class dominates runtime, use `+CLASSLIMIT(limit[n])` or smaller analysis units before turning down mutators globally. Runtime hotspots are often class-shape problems, not whole-project problems.
- In Stryker, `ignoreStatic` is a runtime lever, not a correctness lever. Static mutants run against all tests because per-test coverage cannot be measured once module initialization has already happened.
- Stryker incremental mode always needs the dry run. It discovers test mapping and proves the unmutated suite is runnable. Skipping that step is how teams trust stale reuse.
- Incremental reports only notice some changes. Stryker documents that env vars, snapshots, support files outside mutated/test files, and dependency updates can invalidate reused results without being detected. After those changes, force a rerun instead of trusting cache reuse.
- If your runner only reports test files or test names, not locations, incremental reuse is approximate. Vitest/Mocha/Jasmine-style partial reporting deserves more skepticism than runners that emit exact test locations.
- Google measured fault coupling on bug-fix lines and found a useful mutant on the bug-introducing change in roughly 70% of cases after arid suppression. That is the practical reason to prefer diff-scoped mutation over global score theater.

## Survivor Triage
1. Check whether the mutant lives in arid code: logging, metrics, error-message polish, tuning constants hidden behind mocks, cache warmup, or initialization-only glue. These are common sources of equivalent or unproductive mutants.
2. If the state is `NoCoverage`, add the thinnest test that executes the line before changing assertions. You are debugging reachability first.
3. If the state is `Survived` with real coverage, look for missing relational assertions: ordering, boundary pairs, idempotence, cache invalidation, monotonicity, or "same input, different context" checks. Surviving mutants often mean the test only checked one side of a relation.
4. If the survivor sits in caching code, ask whether removing the cache changes externally visible behavior. Cache-bypass mutants are classic equivalents when tests only assert results, not performance or call count.
5. If the mutant lives in compiler-generated or duplicated bytecode, do not immediately write new tests. Confirm the source mapping first.

## Timeout Triage
1. Re-run the unmutated suite in the same runner/config. If baseline is not clean, mutation results are invalid.
2. If only a few first-touch mutants timeout, raise fixed slack before proportional slack. In PIT that means `timeoutConstant` before `timeoutFactor`.
3. If many mutants in top-level constants, module initializers, or static constructors explode runtime, classify them as static and decide whether to exclude or isolate them.
4. If Stryker.NET mixed mutants produce weird cross-test side effects, disable mixed mutants for diagnosis before rewriting tests.
5. Only after the above should you suspect a genuine infinite loop.

## Gate-Setting Rules
- Use a floor plus trend, not a single heroic number. PIT explicitly warns that equivalent mutants exist, so a hard 100% threshold is usually a governance smell rather than a quality bar.
- Start with module-local thresholds, then tighten only after the "not useful" rate is acceptable. If developers keep dismissing surfaced mutants, your gate is ranking noise.
- Prefer PR-scoped gates and periodic full sweeps over full-suite mutation on every commit. Developer attention, not raw compute, is the real bottleneck.
- When a tool or runner upgrade lands, compare a control run on unchanged code before ratcheting any threshold. Tooling changes can reshuffle survivors without changing product risk.

## NEVER Do These
- NEVER chase 100% mutation score because equivalent mutants, cache-bypass mutants, and arid-code mutants make that target look precise while teaching the team to game the metric. Instead, gate on stable module floors plus survivor review quality.
- NEVER add hyper-specific assertions just to kill the current survivor because that is how change-detector tests get born. Instead, encode the behavior the caller actually relies on: invariants, boundary pairs, error semantics, or cross-call relationships.
- NEVER trust incremental reuse after changing dependencies, env vars, snapshot files, custom test environments, or support code outside the mutated/test file set because Stryker does not detect all of those changes. Instead, rerun with a forced scoped pass or drop incremental for that run.
- NEVER mutate logging, metrics, or error-string code just because the tool can. It is seductive because survivors disappear quickly, but the usual consequence is brittle tests that assert narration instead of behavior. Instead, mutate those paths only when observability output is itself contractual.
- NEVER interpret every timeout as "the mutant created an infinite loop" because cold start, classloading, static initialization, and mixed-mutant side effects produce the same symptom. Instead, separate fixed overhead, proportional slowdown, and static execution before touching product code.
- NEVER treat `NoCoverage` as a mild success because it means your selection logic or test architecture never exercised the mutant at all. Instead, fix reachability or explicitly exclude dead/generated code.

## Edge Cases That Fool Smart People
- `finally` blocks and compiler inlining can produce duplicate-looking mutants with asymmetric kill behavior.
- Static or module initialization can make one mutant run every test regardless of coverage settings.
- Mocked time/network/storage layers make tuning-constant mutants look valuable even when the real behavior is unreachable in tests.
- In-source tests need explicit exclusion from mutation or the tool will happily mutate the test harness with the code under test.
- Research features such as PIT's full mutation matrix are useful for analysis, but PIT warns that other features may not behave correctly when enabled; do not turn them on in routine gates.

## When To Stop
Stop the mutation pass and fix the environment instead when:
- the base suite is red or order-dependent,
- surfaced mutants are mostly unproductive or arid,
- incremental reuse disagrees with a forced rerun,
- timeout behavior changes more than score behavior after a tool upgrade.

Mutation testing earns its keep when it ranks missing behavioral checks. If it is ranking noise, fix selection and suppression before you write a single new test.
