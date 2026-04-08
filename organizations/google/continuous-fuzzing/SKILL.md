---
name: google-continuous-fuzzing
description: Improve OSS-Fuzz, ClusterFuzz, and CIFuzz integrations for real bug yield: choose harness shape, diagnose plateaued fuzzers, triage unreliable crashes, and avoid corpus-destroying or coverage-killing mistakes. Use when working on OSS-Fuzz project.yaml/build.sh/Dockerfile/CIFuzz, writing or splitting fuzz targets, tuning seed corpora or dictionaries, or debugging timeouts, OOMs, and low-coverage fuzzers. Triggers: oss-fuzz, clusterfuzz, cifuzz, fuzz target, seed corpus, dictionary, timeout, oom, sanitizer, fuzz introspector, unreliable crash.
---

# Google Continuous Fuzzing

Use this as a yield-maximization skill, not a fuzzing primer. The job is to keep ClusterFuzz exploring deeper code every day, not to merely "have fuzzers".

## Load Boundaries

- Before generating a Python starter harness, READ `scripts/fuzz_harness_template.py`.
- Do NOT load `scripts/fuzz_harness_template.py` for triage, CIFuzz wiring, or plateau analysis.
- Do NOT load `references/fuzz_techniques.md` unless you explicitly need a syntax refresher for mutation families or dictionaries; it is reference material, not decision support.

## Core Model

- A healthy target keeps finding new interesting inputs after the seed corpus stops carrying it.
- A target with great seed-corpus coverage can still be bad if tiny mutations immediately break the format. Measure discoverability, not just seeded coverage.
- Treat timeouts, OOMs, and shallow bugs as throughput killers first and correctness bugs second. Even "low severity" blockers can stop the engine before it reaches deeper security-relevant paths.
- Optimize for stable target identity. Renames, format churn, and bespoke encodings destroy accumulated corpus value.

## Before You Touch A Harness, Ask Yourself

### Scope

- Is this API actually one target, or is it a 20k-30k+ reachable-edge umbrella that should be split?
- Can this input be represented in its real on-disk or on-wire format so corpora can be shared across targets?
- What state must be reset between executions to keep behavior deterministic?

### Searchability

- If I start from an empty corpus, can mutations plausibly reach deeper parsing logic, or will crypto, CRCs, compression, or checksums reject everything?
- Are the interesting branch selectors tiny flags, keywords, or magic values that deserve a dictionary or explicit enumeration?
- Is this slow because the code is expensive, or because the harness is doing logging, I/O, process launch, or rebuilding state every iteration?

### Operations

- Will this target behave the same under ASan, UBSan, and MSan, or am I about to hide a sanitizer-specific failure mode?
- If this crash is unreliable, am I looking at true nondeterminism or state leaking across iterations?

## Harness Shape Decision Tree

### If the input format is common and shareable

- Prefer consuming the real format directly.
- If there are only a few parsing modes, split into multiple fuzz targets, one mode per target. OSS-Fuzz cross-pollinates corpora automatically, so narrower targets usually beat a single mode-switching monster.
- If you only need a tiny side channel such as flags, prefer embedding it in comments or custom chunks of the real format when the format allows it.

### If the target needs multiple sub-inputs

- If corpus sharing matters, avoid custom serialization. First try separate targets, embedded metadata, or a magic separator found with `memmem`.
- Use `FuzzedDataProvider` only when you accept that the corpus stops being valid instances of the original format.
- If you use `FuzzedDataProvider`, avoid methods returning `std::string` unless the API truly needs strings; they can hide off-by-one bugs that ASan would otherwise catch.
- `ConsumeEnum` and `PickValueInArray` are useful when a few discrete values matter; they draw from the end of the input, which can preserve validity for seed files in some cases.

### If the target is stateful

- For quick exploration, a byte-coded action trace is acceptable.
- For long-lived targets, prefer protobuf-modeled traces even though they are slower. Human-readable reproducers and maintainable schemas usually repay the speed cost.
- Model only values that actually change control flow. For state machines, a tiny domain of meaningful host IDs, status codes, or flags outperforms "full fidelity" schemas stuffed with irrelevant ranges.

### If mutations die at the parser front door

- First try dictionaries, smaller targets, and fuzzing-mode patches that bypass crypto, CRC, checksums, or authentication gates.
- Move to structure-aware fuzzing only when generic mutation cannot survive the format.
- For non-native structured inputs, protobuf plus a converter is usually the maintainable choice when the mapping is stable and the target has long-term value.

## Plateau Triage

When a target has gone quiet for days, use this order:

1. Check for blockers first.
   If ClusterFuzz is repeatedly surfacing timeout or OOM, fix that before chasing coverage.
2. Compare static reachability to dynamic coverage.
   Use Fuzz Introspector. If static reachability is much higher than observed coverage, you likely need a new target or a narrower one, not "more time".
3. Separate "bad corpus" from "bad target".
   Generate coverage from the aggregated OSS-Fuzz corpus, not just your local seeds. If local coverage looks fine but public corpus coverage is weak, the target has not been fed enough useful material.
4. Ask whether the format is mutation-hostile.
   Compression, encryption, CRC, and similar gates often require fuzzing-mode bypasses or custom mutators.
5. Ask whether the target is simply too large.
   Search cost grows superlinearly. One monolithic target often looks productive early because it finds shallow bugs, then stalls.

## Numbers That Matter

- Good target speed: about 1000 exec/s per core is a healthy baseline; 10000+ exec/s is common for lightweight targets; below 10 exec/s usually means the harness is wrong.
- Practical memory target: aim well below 1.5 GB per core even though OSS-Fuzz reports OOM around 2.5 GB RSS by default. Sanitizers add overhead.
- Default timeout/OOM reporting kicks in around 25 seconds or 2.5 GB RSS for a single input.
- CIFuzz default time is 600 seconds; keep 600 as the floor and scale upward with project size rather than downward for convenience.
- ClusterFuzz stats are delayed by up to 24 hours. Do not declare a change ineffective on the same day.
- CIFuzz runs against 30-day-old public corpora and regressions. That is a feature: it gives you regression testing and realistic mutation fuel on pull requests.
- Introspector reports become available after ClusterFuzz has run the project for less than a day.

## ClusterFuzz And CIFuzz Operating Rules

- Build the project in the Dockerfile by cloning with `git`; CIFuzz depends on Git metadata and will not behave correctly if you used `go get` or other fetch shortcuts.
- Keep the OSS-Fuzz project name exact and case-sensitive in CIFuzz.
- Match the sanitizer in both CIFuzz build and run steps. A mismatch creates fake confidence.
- Use `dry-run` only while bringing CIFuzz online. Leaving it on is equivalent to requiring humans to catch crashes manually in logs.
- For PR fuzzing, coverage support matters operationally: with coverage, CIFuzz runs only affected fuzzers; without it, the same time budget is divided across every target.
- For coverage analysis, prefer the aggregated OSS-Fuzz corpus. If you need a full download, daily zipped backups are faster than recursively copying the bucket.
- If `coverage` starts failing locally, pull fresh Docker images before blaming the target. Toolchain/image drift is a common false lead.

## Triage And Fix Heuristics

- Regression range is the highest-value triage signal, but only for reliably reproducible crashes. Narrow ranges require frequent archived builds.
- For unreliable crashes, first suspect leaked state or nondeterminism in the harness. If you cannot reproduce locally, a speculative fix is reasonable when the stack is convincing; let ClusterFuzz verify it over the next few days.
- Remember the unreliable-crash lifecycle: a tracking bug auto-verifies after 2 weeks without frequent sightings, while a lone unreproducible testcase auto-closes after about a week. Do not waste time fighting that automation unless the evidence changed.
- ClusterFuzz re-tests fixed issues daily. Use that instead of inventing ad hoc "looks fixed" rules.
- If a dependency is actually at fault, CC that dependency's maintainers instead of treating the crash as your project's bug forever.
- Timeout issues are not automatically unimportant. Sometimes they are the only reason the fuzzer cannot reach deeper code. Hot-patching the slow path only in fuzzing mode is acceptable when it improves search without polluting production logic.

## Anti-Patterns

- NEVER rename a fuzz target casually because ClusterFuzz closes the old bugs and the new target starts from seed-only corpus. Instead keep the binary name stable, or migrate by copying the old accumulated corpus into the new target location.
- NEVER raise `timeout` or `rss_limit_mb` first because it is the fastest way to preserve a slow, coverage-starved harness. Instead prove the target is intrinsically large-input and remove avoidable work before loosening limits.
- NEVER use a custom TLV or ad hoc serialization for a common file format just because it is easy to parse. Mutations break the framing constantly and you lose corpus reuse. Instead keep the native format, split targets, or use protobuf/custom mutators only when corpus sharing does not matter.
- NEVER use `fmemopen` when an in-memory API exists because it can inhibit important search algorithms in the fuzzing engine. Instead call the in-memory API directly and keep the bytes visible to the mutator.
- NEVER launch helper daemons or child processes just to reach a service-style API because child processes are not coverage-tracked and process launch overhead wrecks throughput. Instead mock the external boundary inside the same process.
- NEVER trust graceful `malloc` failure handling as your OOM strategy because OSS-Fuzz kills on RSS watchdog overshoot, not on allocator return values. Instead cap allocations with a custom allocator or explicit input budgeting.
- NEVER return non-zero from `LLVMFuzzerTestOneInput` as a casual discard mechanism because the meaning is engine-specific and other engines may not honor it. Instead reject uninteresting sizes or ranges cheaply and still return zero.
- NEVER keep one "full stack" target for a huge API because shallow bugs, timeouts, and state explosion will dominate all CPU time. Instead create multiple narrower targets and let corpus sharing do the rest.
- NEVER celebrate high seeded coverage if empty-corpus mutations cannot rediscover it. Instead test discoverability with no corpus, dictionaries, or structure-aware mutations.

## Fallbacks

- If OSS-Fuzz cannot host the harness upstream yet, keeping targets in the OSS-Fuzz repo is acceptable as a temporary bridge, but expect bit rot because they are no longer built with the project's normal tests.
- If Introspector cannot build, verify `-flto -fuse-ld=gold` compatibility before assuming the target itself is the problem.
- If CIFuzz is impossible because the project is not GitHub-hosted or not on OSS-Fuzz, pivot to ClusterFuzzLite or local helper-based regression fuzzing instead of mimicking OSS-Fuzz badly in bespoke CI.

## Done Looks Like

- The target is deterministic, single-process, and resets state between iterations.
- The binary name, corpus strategy, and sanitizer matrix preserve historical signal instead of resetting it.
- Public or aggregated corpora, coverage, and Introspector all agree on where the next bottleneck is.
- CIFuzz runs long enough to be meaningful and fails loudly on real regressions.
