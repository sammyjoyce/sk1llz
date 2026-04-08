---
name: mitre-attack-framework
description: "Turn ATT&CK into defender-grade mappings, hunt hypotheses, and coverage decisions that survive peer review. Use when mapping CTI, incident evidence, raw telemetry, Sigma/EDR detections, Navigator layers, or coverage claims to ATT&CK techniques and sub-techniques. Trigger keywords: map to attack, mitre attack, t10xx, ta00xx, sub-technique, navigator layer, detection coverage, dettect, attack flow, ttp mapping, hunt hypothesis, technique id, attack version."
tags: mitre, attack, ttp, detection, threat-hunting, cti, coverage, navigator, dettect, sub-technique
---

# MITRE ATT&CK for Threat Hunting and Detection Engineering

ATT&CK is useful only when it changes defender decisions. The failure mode is not "wrong ID"; it is building detections, metrics, and priorities on a neat but false mapping.

## Before you map anything, ask yourself

- **What exact observable am I mapping?** If you cannot quote the command line, API call, registry key, cloud API action, network pattern, or file path, you are not at technique level yet.
- **Am I mapping adversary behavior or my sensor artifact?** "Sysmon Event ID 10 fired" is telemetry, not a technique. "Process opened lsass.exe with suspicious access mask" is behavior.
- **How much procedure freedom does this technique have?** Low-freedom techniques can be covered. High-freedom techniques only make sense when decomposed into narrow procedure families.
- **Is this prevalence or visibility bias?** Vendor top-technique lists are useful seeds, but rankings move when telemetry improves. Red Canary's identity detections rose about 5x in 1H 2025 largely from better visibility and baselining, not a 5x change in attacker behavior.

## Use the right ATT&CK model for the task

- If the task is **CTI or incident mapping**, think in `behavior -> tactic -> technique -> sub-technique -> procedure example`.
- If the task is **detection or coverage**, think in `technique/sub-technique -> Detection Strategy (DET) -> Analytics (AN) -> Data Components (DC) -> concrete log sources -> mutable tuning elements`.
- ATT&CK's defensive model changed materially in v18. If your tooling still relies on old `x_mitre_detection` prose or treats "data source present" as coverage, mark it as legacy before comparing it with v18 defensive content.
- ATT&CK versions are not interchangeable. Pin version permalinks in every artifact. A technique that looked stable in v14 may have different detection guidance, sub-technique shape, or tactic placement in v18+.

## Procedure-space heuristic

Use this before promising "coverage":

| Procedure freedom | What it means | How to use ATT&CK |
|---|---|---|
| **Low** | Few realistic ways to perform it | Good coverage candidates. Example families: LSASS access, scheduled-task creation, Registry Run key writes. |
| **Medium** | Bounded but variant-heavy | Cover with multiple analytics plus context. Example families: WMI execution, service execution, cloud firewall changes. |
| **High / unbounded** | Too many valid procedures to "cover the technique" | Never claim full coverage. Split into narrow procedure families and test each one. Example families: Command and Scripting Interpreter, Masquerading, User Execution. |

If you cannot name the procedure family, you are still doing taxonomy, not detection engineering.

## Decision tree - what can I map?

```
Do I have a direct observable?
|- No -> map only to a tactic and label "tactic-only, insufficient detail"
\- Yes -> does the observable uniquely distinguish one technique or sub-technique?
    |- Yes -> use the most specific sub-technique the evidence supports
    \- No -> check for concurrent mappings, not a forced single winner
              then mark any remaining ambiguity explicitly
```

Most non-trivial behaviors map concurrently. HTTP C2 on port 8088 is not a tie between T1071.001 and T1571; it is both.

## Decision tree - should I build detection now?

```
Is this a priority threat scenario for my environment?
|- No -> do not build it just because the matrix has a blank cell
\- Yes -> do I have the required data components with enough retention and parsing?
    |- No -> this is a telemetry engineering problem, not a detection problem
    \- Yes -> is the technique low/medium freedom?
        |- Yes -> build analytics and record mutable tuning elements
        \- No -> narrow to one concrete procedure family first, then validate
```

## NEVER

- **NEVER map from tool names alone because ATT&CK procedure examples make that shortcut feel legitimate.** "Mimikatz" is seductive shorthand, but its modules span T1003.001, T1003.002, T1003.006, T1558, and more. The consequence is false coverage and the wrong downstream analytic. **Instead do** argument- and API-level mapping from the behavior you actually observed.
- **NEVER infer a technique from an outcome because tactics read like explanations.** "They gained persistence" sounds usable, but it hides whether the behavior was T1546.003, T1547.001, or something else. The consequence is that responders and detection engineers work the wrong branch. **Instead do** tactic-only mapping until the mechanism is known.
- **NEVER roll up to a parent technique when the sub-technique is knowable because the parent looks cleaner on a heatmap.** Process Injection is the classic trap: one green `T1055` cell can hide a dozen unaddressed sub-techniques. The consequence is coverage theater. **Instead do** sub-technique mapping whenever the evidence distinguishes it, and say "parent-only" only when the evidence truly cannot go further.
- **NEVER score ATT&CK coverage as a flat percent because the matrix punishes honesty.** Teams love a single number, but low-freedom and high-freedom techniques are not equally coverable, and visibility is not detection. The consequence is a dashboard that looks mature while attackers still have untouched procedure families. **Instead do** threat-scenario-based scoring with separate visibility and detection axes.
- **NEVER trust "top techniques" rankings without checking sensor bias because improved telemetry reorders the leaderboard.** Identity and cloud techniques rose sharply in 2025 partly because defenders finally instrumented them better. The consequence is overreacting to a measurement artifact or underfunding a blind area. **Instead do** prioritize with three inputs: adversary prevalence, business relevance, and your current visibility maturity.
- **NEVER build process-injection detection around raw `CreateRemoteThread`-style events because the API is common in benign software.** It is seductive because it looks technique-pure. The consequence is analyst fatigue and rule disablement. **Instead do** correlate cross-process access with unusual source processes, blank or implausible command lines, target-process baselines, and post-injection effects such as unexpected network or module activity.
- **NEVER force cloud-native activity into a single ATT&CK matrix because control-plane and workload-plane behavior live in different places.** The seductive path is one clean Navigator layer. The consequence is hiding the pivot from IaaS to Kubernetes to container to host. **Instead do** model the behavior across the relevant matrices and keep the seams visible.
- **NEVER ship a mapping that no second analyst has challenged because familiarity bias is strongest where confidence feels highest.** The consequence is missing concurrent techniques, overusing favorite IDs, and encoding your own blind spots into detections. **Instead do** require peer review for any mapping that leaves your team or drives coverage claims.

## Detection-engineering rules that are easy to miss

- In v18+, ATT&CK detection strategies expose **mutable elements**. Record them. A detection that says "T1110 covered" is weak; a detection that records `TimeWindow`, `FailureThreshold`, allowlists, credential type, and expected source population is reviewable.
- Good mutable elements are specific enough to tune and defend in review: `TimeWindow=10m` for clustered container discovery, `TimeWindow=5m` for create -> start -> first activity correlation in container deployment, `SustainedCPU >15m` for resource-hijacking behaviors, or explicit `ScheduleWindow` values such as `@hourly` for suspicious CronJob cadence. If your rule cannot name its tuning knobs, it is not ATT&CK-grounded yet.
- Prefer ATT&CK content that names **data components**, not vague "data source" claims. "We have network logs" is useless. `Network Connection Creation`, `Cloud Service Modification`, and `Process Access` are different detection surfaces with different blind spots.
- Treat **blank command line** as signal only for processes where a command line is normally expected. It is powerful for injected or proxy-executed processes and useless as a universal heuristic.
- For LSASS-oriented detections, baseline **who normally opens lsass.exe** in your environment before you write alerts. Protected processes, EDR agents, backup tools, and debuggers can all make naive rules collapse.
- When ATT&CK offers a DET object, use it to break one technique into multiple analytics rather than writing one "mega detection." That is the only sane way to handle medium-freedom techniques.

## Practical operating procedure

1. Quote the behavior literally from the evidence. Screenshots, figure captions, command blocks, and footnotes often contain the real mapping clues; analysts routinely miss them on the first pass.
2. Search ATT&CK with verbs, flags, paths, APIs, ports, and cloud actions, not just nouns. "schtasks /create", `MiniDumpWriteDump`, `Set-Mailbox`, `AuthorizeSecurityGroupIngress`, and `__EventFilter` beat "persistence" every time.
3. Check whether multiple techniques are simultaneously true. If so, map all of them and explain the different defender implications.
4. If the task is detection or coverage, identify the exact data components and write down the mutable tuning elements before you call anything "detected."
5. Pin the ATT&CK version and attach permalinks. Include the parent technique on first mention of every sub-technique.
6. Send any publishable or backlog-driving mapping for peer review.

## Edge cases and fallback paths

- If the evidence supports **only Reconnaissance or Resource Development** but your task is internal detection, stop trying to write SIEM coverage for it. That is an external visibility problem for CTI, ASM, takedown, or brand monitoring.
- If the behavior appears to fit ATT&CK poorly, do **not** force the nearest popular ID. Record the literal behavior, cite the closest candidate only as a hypothesis, and note that ATT&CK may need a new or revised technique.
- If the same behavior crosses **multiple matrices or planes** such as cloud control plane, Kubernetes, container, and host, preserve that split in the output. Collapsing it into one matrix is convenient but analytically false.
- If the consumer needs machine-readable output, store **ATT&CK ID, ATT&CK version, evidence excerpt, and confidence note** together. ID without version and evidence is not durable enough for later automation.

## Mandatory loading triggers

- **Before publishing or sharing a mapping outside the immediate task, READ `references/mapping-pitfalls.md`.** It contains the failure patterns that most often survive first-pass review.
- **Before writing or revising hunt queries, Sigma, EDR analytics, or validation tests, READ `references/techniques.md`.** It is detection-facing and intentionally query-heavy.
- **Do NOT load `references/techniques.md` for CTI-only mapping or coverage-scoring tasks.** It will bias you toward whichever behaviors already have ready-made hunts.
- **Do NOT load `scripts/coverage_analyzer.py` unless the task is explicitly to calculate or visualize coverage.** It is useful for scoring workflows, not for determining the mapping itself.

## Coverage model that survives scrutiny

Maintain three distinct layers:

1. **Threat scenario layer**: techniques and sub-techniques actually used by threat groups, intrusions, or abuse cases that matter to your environment.
2. **Visibility layer**: required data components exist, are retained long enough, and are parsed well enough to hunt.
3. **Detection layer**: tuned analytics exist and have been validated with atomic or purple-team testing.

`Gap = Threat scenario - (Visibility intersection Detection)`

Anything else is just ATT&CK-colored optimism.
