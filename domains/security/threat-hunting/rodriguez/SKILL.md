---
name: rodriguez-threat-hunter-playbook
description: Build and review threat hunts in Roberto Rodriguez's Threat Hunter Playbook style: pre-hunt data management, relationship-driven analytics, notebook-first evidence chains, and replay validation with OTRF Security Datasets. Use when creating a new hunt playbook, auditing why a hunt failed, porting THP ideas into an existing SIEM or notebook stack, validating ATT&CK coverage claims, or working on HELK, OTRF, OSSEM, Security Datasets, Mordor, DCSync, remote PowerShell, WMI, service-creation, or other Windows behavior hunts.
---

# Rodriguez Threat Hunter Playbook

## Loading Rules

- Before redesigning the overall methodology or explaining HELK/OTRF lineage, READ `references/methodology.md`.
- Do NOT load `references/methodology.md` for live triage, a quick notebook pivot, or Sigma conversion. It is architecture-heavy and will slow down hunt work that depends on field-level decisions.

## What This Skill Adds

- Rodriguez is not "use ATT&CK plus notebooks." His edge is that most hunt failures are diagnosed as data-management failures before query rewrites begin.
- Treat ATT&CK as a planning scaffold for required data, not proof that the technique is huntable in your environment.
- Treat hunts as evidence chains. Single events are usually weak; joins on stable relationship keys are where the analytic becomes durable.

## Before You Hunt, Ask Yourself

- Is this really an analytic problem, or is the telemetry incomplete, inconsistent, late, or unmodeled?
- Which exact event provider, event ID, and relationship prove the behavior? If you cannot name the join key, you are not ready to write the query.
- Does the source log success, failure, or both? Several useful Windows events only expose one side of the action.
- Are you looking at event time or ingest time? Delayed forwarding can make a 24-hour-old action look current and destroy sequence-based reasoning.
- Is rarity global, or only rare inside the right slice? Rodriguez repeatedly baselines by business unit, department, role, or recurring source address, not by whole enterprise averages.

## Rodriguez Workflow

1. Start with data-source coverage, not detections.
   Use ATT&CK data sources as an initial 1/0 map of what your tools claim to provide. This is only the first pass.
2. Score data quality before trusting the map.
   Rodriguez's useful dimensions are completeness, consistency, and timeliness. Completeness is per source; consistency and timeliness are often pipeline-wide when everything lands in one SIEM. Full deployment is not full completeness: if `CommandLine` is blank on a meaningful slice of endpoints, the analytic is still weak.
3. Document only the events needed for the hunt, but document them deeply.
   Record raw fields, collector quirks, and the relationship keys that survive normalization such as `ProcessGuid`, `ParentProcessGuid`, `LogonId`, `ObjectHandle`, `TargetFilename`, and `SourceAddress`.
4. Build the hunt as a notebooked experiment.
   The notebook should preserve the reasoning chain: hypothesis, collection prerequisites, analytics, pivots, expected benigns, known bypasses, and validation notes.
5. Replay before you celebrate.
   Validate with OTRF Security Datasets or a controlled lab replay. A hunt that only works on your prettiest production shard is not validated.
6. Publish the blind spots.
   Rodriguez-style work always names missing audit rules, ETW-only visibility, noisy sources, and bypasses. Silence on prerequisites is a bug.

## High-Value Heuristics

- Use ATT&CK coverage bins such as `0-20`, `20-40`, `40-60`, `60-80`, `80-100` only for planning boards. They are useful for prioritization and misleading for capability claims.
- A tool saying it provides a data source does not mean the analytic has value. Rodriguez's Sysmon example is brutal: one exclude rule on a relevant image or module can reduce effective coverage for that analytic to zero.
- Packed fields are a quality problem, not just a parsing annoyance. Sysmon's combined `Hashes` field forces regex-heavy workarounds, increases query cost, and makes cross-source stacking harder than vendors imply.
- Data dictionaries are not paperwork. They expose useful fields analysts usually miss and frequently reveal stronger pivots than the original idea.
- Relationship modeling is the accelerator. Once you know which fields express "process created process" or "user authenticated host," you stop writing isolated queries and start writing behavior chains.

## Decision Tree

- If the primary event only logs failures:
  Treat it as a lead, not a detector. Find the success-side event and correlate it with logon, network, or object-handle telemetry.
- If the best source exists only in ETW or another hard-to-scale feed:
  Prove the idea in lab or targeted collection first. Then design a production fallback using more common logs, even if fidelity is lower.
- If the behavior is administratively noisy:
  Baseline by department, role, source system, or daily recurring creator. Enterprise-wide rarity usually hides the signal you need.
- If replay data is unavailable:
  Reproduce the action in a controlled lab, or state explicitly that validation debt remains. Never imply replay validation that did not happen.

## Field-Tested Hunt Patterns

- DCSync:
  Correlate `4662` with `4624` logon type `3` on the domain controller by `LogonId`. If you can collect the Microsoft-Windows-RPC ETW provider, filter the replication interface GUID `E3514235-4B06-11D1-AB04-00C04FC2DCD2`, but do not make that a default scale dependency because it does not naturally land in `.evtx` and is operationally awkward without tooling such as SilkETW.
- Remote PowerShell:
  Do not stop at `wsmprovhost.exe`. Include inbound layer `44` traffic on ports `5985` and `5986`, then baseline by department if WinRM is common.
- Alternate PowerShell hosts:
  Hunt for `System.Management.Automation` loaded by non-`powershell.exe` processes and for named pipes beginning with `\\PSHost`. Those pivots surface hosts that never advertise themselves as PowerShell.
- Remote SCM handle discovery:
  `4656` on the SCM database is mostly a failed-handle story because the SCM object lacks the SACL coverage people assume. Use `4674` for successful privileged access, join to `4624` type `3`, inspect `5156` inbound `services.exe` traffic on layer `44`, and pivot on `ObjectHandle` to see what happened after the handle opened.
- Remote service creation:
  Join `4697` to `4624` type `3` by `LogonId` to separate remote installs from local administration. Stack service binary paths and daily creators; Microsoft software can legitimately create many unique services and will fool naive rarity tests.
- Registry-based hunts:
  If the target key lacks the required SACLs, the hunt is blind no matter how elegant the query is. Treat audit-rule verification as part of analytic validation.

## Anti-Patterns

- NEVER claim hunt readiness from ATT&CK coverage alone because vendor heat maps and binary matrices are seductive. They make partial telemetry look equivalent to usable telemetry and lead teams to trust blind analytics. Instead score coverage first, then grade completeness, consistency, and timeliness before making capability claims.
- NEVER write the query before documenting the raw event relationships because field names look familiar enough to improvise. You will miss the join key that carries the behavior across events and end up with a brittle single-event detector. Instead capture the provider, event IDs, join fields, and collection prerequisites first.
- NEVER rely on failure-side Windows events as if they represent the whole behavior because they are easy to observe and demo. The concrete result is systematic under-detection of successful tradecraft. Instead identify which event logs the successful action, then correlate both sides if useful.
- NEVER baseline noisy admin behaviors at whole-enterprise level because it is the fastest chart to build. The consequence is that remote PowerShell, service creation, or alternate hosts disappear into global admin noise. Instead baseline inside the operational slice that owns the behavior.
- NEVER trust normalized timestamps blindly because ingestion time is often easier to access than creation time. The consequence is broken timelines, bad pivot windows, and false "simultaneous" stories. Instead verify event-time semantics before using sequence-based logic.
- NEVER rebuild HELK just because Rodriguez used it and it feels canonical. The seductive path is platform cosplay. The consequence is weeks of plumbing work with no new hunt fidelity. Instead port the relationship model and notebook logic into the platform you already operate.
- NEVER accept "the tool collects it" as proof that the hunt has value. Partial field capture, Sysmon include/exclude drift, and vendor parsing shortcuts can drop the one attribute your analytic needs. Instead test the exact fields against replay data or a lab trace.

## Freedom Calibration

- High freedom:
  Choosing the behavioral chain, selecting pivots, picking the business slice for baselining, and deciding whether the right answer is a hunt, a data-quality escalation, or both.
- Low freedom:
  Event IDs, join keys, audit-rule prerequisites, ETW caveats, validation claims, and any statement about coverage. Do not invent telemetry that is not collected.

## Output Checklist

- State the hypothesis in one sentence.
- Name the exact providers, event IDs, fields, and join keys required.
- Call out collection prerequisites such as SACLs, command-line auditing, or ETW setup.
- Provide the primary analytic plus at least one relationship-based pivot.
- Name the expected benign slice and how you would baseline it.
- State validation method: OTRF dataset, lab replay, or explicit validation debt.
- End with blind spots and known bypasses, not just matches.
