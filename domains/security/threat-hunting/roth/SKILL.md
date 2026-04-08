---
name: roth-detection-engineering
description: "Expert playbook for Florian Roth style detection engineering across Sigma, YARA, YARA-X, LOKI, THOR, and signature-base. Use when writing or tuning portable SIEM detections, scoring YARA hits, reducing false positives, debugging unsupported backend conversions, or fixing slow rules. Triggers: sigma, pysigma, sigma-cli, yara, yara-x, THOR, LOKI, signature-base, rule tuning, false positives, backend mapping, threat hunting."
---

# Roth Detection Engineering

Roth's style is not "write more rules." It is "ship rules that survive backend translation, analyst review, false-positive pressure, and community reuse." A rule that looks elegant in YAML or YARA but dies in conversion, gets suppressed by analysts, or times out under load is not finished.

## Load Only What You Need

- Before editing exact Sigma syntax, modifiers, or condition grammar, READ `references/sigma_reference.md`.
- Do NOT load `references/sigma_reference.md` for a YARA-only task; it is context waste.
- Do NOT load `scripts/sigma_converter.py` unless you are debugging a local conversion artifact in this package. It hard-codes mappings and is not Roth's source of truth.

## Core Heuristics

- Portability is a hypothesis, not a property. Prove it per backend.
- False-positive cost matters more than cleverness. A noisy high-severity rule gets disabled faster than a modest low-severity hunt rule gets improved.
- Prefer stable public references or `Internal Research`. Rules that depend on private context do not scale to community use.
- Separate "broad hunt surface" from "actionable alert." Roth's ecosystems keep both, but they are not the same artifact.

## Before Writing Sigma, Ask Yourself

- Is this an alert, a hunt, an emerging-threat rule, or only a placeholder for later correlation?
- Which backends must pass today, and which fields or modifiers do those backends actually support after normalization?
- What benign software, admin workflow, or installer looks closest to this behavior?
- Which field is carrying the signal: raw log field, mapped field, or pipeline-enriched field? If you cannot answer that, you are not ready to call the rule portable.

## Sigma Procedure

1. Start from telemetry reality, not ATT&CK labels. Wrong `logsource` is a bigger portability killer than imperfect tagging.
2. Use the cheapest operator that preserves meaning. `contains`, `startswith`, `endswith`, and `windash` usually survive backend translation better than regex.
3. Convert against every target backend before you tune severity. Use `sigma-cli --fail-unsupported` in release gates. Use `--skip-unsupported` only during exploration, never as proof the rule is deployable.
4. If one backend cannot support the rule because of fields, modifiers, or aggregate logic, split the rule into a portable core plus explicit backend-specific variants. Do not pretend a broken universal rule is "good enough."
5. Set level by review cost, not by emotion. Sigma guidance is conservative for a reason: `low` and `medium` are for hunting, compliance, and correlation; `high` is for alerts that still need human review; `critical` should be close to intolerable-false-positive territory.

## Sigma Failure Modes That Practitioners Learn the Hard Way

- Backend support is uneven. Some pySigma backends reject specific fields or rule types; for example, the InsightIDR backend documents unsupported fields such as `CurrentDirectory`, `IntegrityLevel`, `imphash`, and `LogonId` for process starts, and it does not support deprecated aggregate `count()` style conditions. If you do not gate on that, you create silent coverage gaps.
- Modifier combinations can be logically correct on paper and broken in practice. Sigma release notes include real fixes where `windash` combined incorrectly with `all` changed rule logic. Treat modifier composition as something to test, not trust.
- Regex is seductive because it compresses thought, but it raises conversion drift and runtime cost. If a simpler modifier can express the intent, use the simpler modifier first.

## Before Writing YARA, Ask Yourself

- Am I detecting a family, a capability, an anomaly, or a triage lead?
- Which strings are truly unique (`$x*`), grouped behavior markers (`$s*`), scope limiters (`$a*`), and false-positive suppressors (`$fp*`)?
- Which engine will run this: classic YARA, YARA-X, THOR, LOKI, or VirusTotal-style infrastructure?
- Can any module value be undefined on irrelevant file types, and if so, what happens to my condition?
- What scan-time budget is acceptable on the real corpus, not on one sample?

## YARA Procedure

1. Add cheap scope limiters first: magic bytes, file size, file-type/module checks, then threat strings.
2. Score rules deliberately. In Neo23x0's style guide, `0-39` is capability dust or weak packer evidence, `40-59` is noteworthy anomaly, `60-79` is suspicious heuristic/generic detection, and `80-100` is reserved for high-confidence malware or hack-tool matches.
3. Use hashes that directly refer to the file you expect to match. Do not use archive/container hashes unless the archive itself is the artifact. Memory-only rules are the exception.
4. Guard module-dependent logic with `defined` when the file type may vary. In YARA/YARA-X, `undefined` propagates in unintuitive ways: `A or undefined` can still be true, while many direct comparisons on undefined values simply collapse to non-matches.
5. Profile before "optimizing." In YARA-X profiling, rules under `100ms` are omitted; chase the outliers. If pattern matching time dominates, your atoms/regex/hex patterns are the problem. If condition evaluation dominates, your loops or module-heavy predicates are the problem.
6. Pick the engine intentionally. Classic YARA is still about `2-3x` faster on already-optimized plain text or simple hex rules, while YARA-X is much better on regex-heavy, complex hex, and loop-heavy rules and materially reduces timeout/rejection pressure on shared scanning systems.

## YARA Failure Modes That Matter

- Slow rules do not only hurt themselves. At VirusTotal scale, a single inefficient rule can starve shared capacity; timeouts were severe enough that performance-warning rules were historically rejected, and YARA-X was adopted partly to push timeout impact from about `2%` of scanned files to under `0.2%`.
- "Compiles" does not mean "can match." YARA-X now warns on unsatisfiable expressions such as comparing lowercase `hash.md5(...)` output to uppercase hex, or comparing `uint8(...)` to values outside `0..255`. Treat those warnings as correctness failures, not lint noise.
- Neo23x0 signature-base includes rules that rely on external variables for THOR/LOKI. Reusing those rules in another engine without supplying equivalent variables yields `undefined identifier` failures.

## Decision Tree

- Need one rule to share across multiple SIEMs? Start in Sigma. If conversion fails on one backend, decide whether portability is required. If yes, rewrite to the portable core. If no, fork backend-specific variants and document why.
- Need file, memory, or artifact scanning? Start in YARA. If the rule is slow or rejected, profile it first; move to YARA-X for diagnosis when regex, complex hex, or loops are involved.
- Need LOKI/THOR compatibility? Prefer Neo23x0 conventions: scoped scores, explicit false-positive suppressors, and tool-aware variables. If you are outside that ecosystem, strip or replace external-variable logic before you trust the rule.
- Rule is still noisy after two serious filter passes? Downgrade it to hunt or lower its score/level. Do not promote noisy heuristics and hope operations will tolerate them.

## Anti-Patterns

- NEVER call a Sigma rule portable because it compiles once; that is seductive because YAML feels backend-neutral. The consequence is silent blind spots when a backend drops unsupported fields, modifiers, or aggregates. Instead, convert against every target backend and fail the build on unsupported constructs.
- NEVER lead with regex in Sigma because it feels expressive and future-proof. The consequence is slower queries, conversion drift, and higher odds that the rule gets rewritten or disabled downstream. Instead, exhaust field modifiers first and reserve regex for patterns that genuinely need grammar.
- NEVER guess `logsource` from ATT&CK technique names because it feels faster than validating telemetry. The consequence is a rule that looks correct but never lands on the event family you actually ingest. Instead, anchor the rule on real source data and only then tag it.
- NEVER mark a heuristic rule `high` or `critical` because urgent labels get attention. The consequence is analyst fatigue, blanket suppression, and permanent distrust of the ruleset. Instead, keep broad heuristics at hunt-friendly levels until prevalence and filters justify escalation.
- NEVER copy Neo23x0 signature-base rules blindly into another scanner because THOR/LOKI-specific external variables are convenient and easy to miss. The consequence is `undefined identifier` errors or silently altered logic. Instead, isolate those rules or provide equivalent variables explicitly.
- NEVER "fix" YARA performance by randomly deleting strings because the slowest rules are usually dominated by bad atoms or unbounded condition work, not by string count alone. The consequence is lower specificity with little speed gain. Instead, profile and attack the dominant cost bucket.
- NEVER compare module outputs or hashes without thinking about type, case, and range because YARA/YARA-X will happily let you write logic that can never be true. The consequence is silent false negatives that look like clean scans. Instead, use `defined`, normalize case, and treat unsatisfiable-expression warnings as blocking.

## Fallback Tactics

- If a portable Sigma rule loses too much fidelity, keep a small portable detector for shared coverage and pair it with backend-native enrichment rules for precision.
- If benign tools collide with malicious strings in YARA, move benign markers into `$fp*` logic or tool-specific false-positive filters instead of globally excluding whole directories.
- If LOKI filename IOCs are noisy, use its `Regex;Score;False-positive Regex` format before resorting to broad path exclusions.
- If references are unstable or private, prefer `Internal Research` over dead links and preserve the analytic rationale in metadata, not in tribal memory.
