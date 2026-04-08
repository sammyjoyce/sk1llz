---
name: mitre-attack-framework
description: Map detections, CTI reports, incidents, and hunt hypotheses to MITRE ATT&CK techniques and sub-techniques without falling into coverage theater. Use when (1) mapping a threat report or incident narrative to techniques, (2) building or reviewing technique-aligned detections and Sigma/EDR rules, (3) assessing detection coverage or producing an ATT&CK Navigator layer, (4) prioritizing detection-engineering work by real adversary prevalence, (5) writing hunt hypotheses framed by ATT&CK, or (6) auditing vendor "100% ATT&CK coverage" claims. Trigger keywords - "map to ATT&CK", "technique ID", "T10xx", "T1059", "TA00xx", "sub-technique", "ATT&CK coverage", "coverage heatmap", "Navigator layer", "DeTT&CT", "hunt hypothesis", "TTP mapping", "threat-informed defense", "kill chain to ATT&CK", "gap analysis".
tags: mitre, attack, ttp, detection, threat-hunting, cti, coverage, navigator, dettect, sub-technique
---

# MITRE ATT&CK for Detection Engineering & Threat Hunting

ATT&CK is not a checklist, not a kill chain, and not a score. It is a **shared vocabulary for adversary behavior**. It creates value only when mappings are precise, prioritized by real prevalence, and tied to analytics that actually fire — not to the data sources that *enable* analytics.

## The one question that prevents most mistakes

Before writing a single technique ID, ask:

> **"What specific observable behavior am I mapping — and what is the shortest path I can justify from that evidence to this ID?"**

If you cannot quote the behavior (command line, API call, registry key, network pattern, file artifact) and trace it to the technique's description on attack.mitre.org, you are guessing. Guessing is worse than no mapping because it inflates coverage dashboards and produces CTI that creates *false confidence*.

## Think in these frames before you map

| Frame | Question to ask yourself |
|-------|--------------------------|
| **Behavior-first** | Does my evidence describe *how* the adversary acted, or only *what* they achieved? The latter maps only to a tactic. |
| **Granularity floor** | What is the *most specific* sub-technique the evidence actually supports? Never roll up to the parent to look cleaner. |
| **Concurrency** | Which *other* techniques apply to the same behavior simultaneously? Non-trivial behaviors usually map to 2+. |
| **Tactic cross-listing** | Does this sub-technique live under multiple tactics (e.g., T1055 is both Defense Evasion and Privilege Escalation)? State all applicable tactics. |
| **Version pin** | Which ATT&CK version am I targeting? Technique IDs are renamed, split, and deprecated across releases. Permalinks matter. |
| **Sector prevalence** | Is this technique in the Red Canary / CISA / sector top-20, or am I mapping a 0.1%-prevalence technique while ignoring T1059 (Command and Scripting Interpreter)? |

## NEVER

- **NEVER report "ATT&CK coverage" as a percentage of techniques.** Seductive because it fits a dashboard and closes budget conversations. But T1055 (Process Injection) has 15+ sub-techniques; detecting one leaves 14 viable attack paths open while your heatmap turns green. Leadership then believes you are protected and declines further investment. *Instead:* score per sub-technique on **two independent axes** — visibility (data source present?) and detection (tuned analytic fires?) — using a 0–5 scale (DeTT&CT model). Report progress against a **prioritized threat-scenario list**, not the full matrix.
- **NEVER leap from a tool name to a technique.** "Mimikatz observed → T1003.001" is seductive because ATT&CK lists Mimikatz as an example. But Mimikatz has 20+ modules spanning LSASS (T1003.001), SAM (T1003.002), DCSync (T1003.006), Kerberos ticket theft (T1558), LSA Secrets (T1003.004), and more. Mapping to T1003.001 alone hides the actual tradecraft, misdirects the responder, and leaves your Kerberoasting blind. *Instead:* map to the **command-line arguments and API calls you actually observed**, not the binary name.
- **NEVER infer a technique from an outcome.** "The adversary achieved persistence" sounds like a mapping but is only a tactic (TA0003) — you have no *how*. If your report writes "T1547" and the real persistence was WMI event subscription (T1546.003), a downstream engineer will build a Run-key detection against the wrong behavior. *Instead:* when you cannot name the *how*, map only to the tactic and **label it "tactic-only, insufficient detail"** so consumers know not to build detections from it.
- **NEVER trust "100% ATT&CK coverage" claims — yours or a vendor's.** Enterprise ATT&CK is ~200 techniques and ~400+ sub-techniques, each with dozens to hundreds of viable variants. No product detects them all. No SOC can tune them all. Mick Douglas and Josh Zelonis have been publicly calling this out for years. *Instead:* demand demonstrated coverage against **named threat scenarios** (e.g., "ALPHV Blackcat TTPs T1078, T1486, T1558, T1003, T1021"), each validated via Atomic Red Team or purple-team exercise.
- **NEVER treat Reconnaissance (TA0043) and Resource Development (TA0042) as huntable inside your environment.** These occur *outside* your perimeter before compromise; you have zero internal telemetry. Trying to "cover" them with SIEM rules wastes engineering cycles on controls you cannot build. *Instead:* address those tactics via external CTI feeds, attack-surface management, certificate-transparency monitoring, and brand/domain monitoring — never SIEM detection rules.
- **NEVER confuse T1595 (Active Scanning) with T1046 (Network Service Discovery).** The names read nearly identically but sit on opposite sides of the compromise line. T1595 = Reconnaissance, *external*, pre-breach. T1046 = Discovery, *internal*, post-breach. Confusing them breaks your entire kill-chain reconstruction and sends responders to the wrong phase of the intrusion.
- **NEVER equate "data source present" with "technique detected."** Having Sysmon Event ID 10 (ProcessAccess) enables *visibility* into LSASS reads. It is not a *detection*. Without a tuned analytic that distinguishes malicious from benign access patterns (and there are many legitimate LSASS readers: MsMpEng, csrss, wininit, EDR agents), nothing ever fires. *Instead:* require a named, tested, **false-positive-tuned analytic** before marking any cell as detected.
- **NEVER finalize a non-trivial mapping solo.** MITRE's own ATT&CK team performs at least two peer reviews on every public mapping. Solo mapping is biased toward the analyst's familiar techniques and routinely misses image/screenshot/command-line evidence that a second pair of eyes catches. *Instead:* peer review is mandatory for any mapping that will drive detection engineering or leave your team.

## Decision tree — "Where does this evidence map?"

```
Is the evidence a specific observable (command line, API call, registry key, network pattern, file artifact)?
├── NO  → Map only to a tactic. Label "tactic-only, insufficient detail." STOP.
└── YES → Is there exactly one technique whose description matches the observable?
         ├── YES → Does that technique have sub-techniques?
         │        ├── YES → Pick the most specific sub-technique the evidence supports.
         │        │        NEVER roll up. If evidence spans multiple, map ALL of them.
         │        └── NO  → Map to the technique.
         └── NO  → Multiple apply concurrently — the common case.
                   Map ALL of them. Example: HTTP C2 on port 8088 →
                   T1071.001 (Web Protocols) AND T1571 (Non-Standard Port).
```

## Decision tree — "Should I build detection for this technique now?"

```
Is this technique in a current top-20 prevalence list (Red Canary TDR, CISA advisories, your sector CTI)?
├── YES → Visibility score ≥ 3 (do you have the data source AND retention)?
│        ├── YES → Build analytic. Target ONE sub-technique variant at a time.
│        │        Validate with Atomic Red Team before shipping.
│        └── NO  → This is a DATA ENGINEERING problem, not a detection problem.
│                 Stop writing rules. Onboard the data source first.
└── NO  → Is it actively used by a threat group targeting your sector?
         ├── YES → Medium priority. Queue after the top-20 are complete.
         └── NO  → DEPRIORITIZE. Burning cycles here is how teams end up
                   with "100% coverage" dashboards and 0% real detection.
```

## Expert patterns that separate seniors from juniors

- **Map concurrently.** CISA's canonical example: HTTP-based C2 over port 8088 maps to **both** T1071.001 (Web Protocols) AND T1571 (Non-Standard Port). Picking one is incomplete.
- **Use versioned permalinks** (`https://attack.mitre.org/versions/v14/techniques/T1105/`), never the bare URL. ATT&CK evolves; your report must pin the version you analyzed against.
- **Always cite parent technique alongside sub-technique** (e.g., `T1003.001 (sub-technique of T1003 OS Credential Dumping)`). Sub-technique suffixes like `.001` are reused across dozens of parents and create table-reading errors.
- **Map the images, command blocks, and footnotes** in CTI reports — not just the narrative. Experienced analysts miss 20–40% of mappable TTPs on the first pass because they skim screenshots.
- **Hunt hypothesis template:** *"If adversary uses technique Y via sub-technique Z in our environment, we would see [observable] in [data source]; absence of the observable means either no activity OR a visibility gap."* The final clause forces you to separate "hunted and clean" from "we were blind."
- **Defense Evasion is a hunting gift, not a blocker.** Techniques designed to hide generate their own telemetry: T1070.001 (Clear Windows Event Logs) fires Event ID 1102; T1562.001 (Impair Defenses: Disable Tools) fires security-service stop events; process hollowing creates unusual memory patterns. Hunt the evasion, not just the payload.

## Workflow

1. **Quote the behavior literally** from your evidence (command line, API, registry, network).
2. **Before mapping for publication, detection engineering, or CTI release: READ `references/mapping-pitfalls.md`.** It contains the CISA-derived common-mistake catalog (leaping to conclusions, vague language, incomplete search, bias) plus the corrected rewrites. This is MANDATORY for any mapping that leaves your team. Do NOT load it for routine hunt-query drafting.
3. **When you need battle-tested hunt queries for the top-20 techniques** (PowerShell download cradles, LSASS access patterns, scheduled-task creation, UAC bypass, etc.), READ `references/techniques.md`. Do NOT load it when you are mapping CTI narratives — it is detection-facing, not analysis-facing.
4. **Cross-check** your candidate mapping against at least one procedure example on `attack.mitre.org/techniques/<ID>/`. Procedure examples reveal whether your interpretation matches how the community has historically used the ID.
5. **Peer review** — a second analyst signs off on every mapping that will drive detection engineering or be published.
6. **Record the ATT&CK version** (e.g., "mapped against ATT&CK v14") in the artifact itself.

## Graduated coverage scoring (use this instead of red/yellow/green)

| Axis | 0 | 1 | 2 | 3 | 4 | 5 |
|------|---|---|---|---|---|---|
| **Visibility** | None | Partial logs | Full logs, no retention | Logs + retention | Logs + retention + parsing | Indexed and hunt-ready |
| **Detection** | None | 1 variant alerts | Multiple variants alert | Variants alert + FP-tuned | Validated via purple team | Continuously validated (Atomic Red Team in CI) |

Visibility and Detection are **independent**. A cell with V=5 D=0 is a **hunt opportunity**. A cell with V=1 D=3 is a **lie to yourself** — you cannot detect what you cannot see. Never collapse these two axes into one number.

For a reference implementation of this scoring model as a Python analyzer, `scripts/coverage_analyzer.py` holds a working skeleton. Do NOT load it for CTI-report mapping tasks — it is orthogonal to analysis work.
