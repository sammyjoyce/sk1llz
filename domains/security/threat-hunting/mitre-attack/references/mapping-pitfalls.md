# ATT&CK Mapping Pitfalls — Expert Catalogue⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​‌‌‌‌​‌‍‌‌​​​​​‌‍‌​‌‌‌‌​​‍​‌‌‌​‌‌​‍​​​​‌​‌​‍‌‌​​‌​​​⁠‍⁠

Load this file only when mapping a CTI report, an incident narrative, or raw telemetry that will be **published, shared with another team, or used to drive detection engineering**. Do not load it for ad-hoc hunt-query drafting.

This file compresses CISA's *Best Practices for MITRE ATT&CK Mapping* (Jan 2023, TLP:CLEAR), MITRE's *Getting Started with ATT&CK*, public critiques by Josh Zelonis (Forrester), Mick Douglas, and Omer Singer ("Stop Playing ATT&CK Bingo"), and years of peer-review lessons from the ATT&CK community into the traps that actually catch practitioners.

---

## Pitfall 1 — Leaping to conclusions

**Definition:** Prematurely committing to a mapping before examining the evidence.

| Wrong | Right |
|---|---|
| "C2 beacons on port 443 → T1071.001 Web Protocols." | "We observed TLS to port 443 but did not decrypt. Map to T1573.002 (Asymmetric Cryptography) plus T1071 (Application Layer Protocol); **do not** map to T1071.001 without confirming HTTP." |
| "Malware wrote `update.exe` to disk → T1105 Ingress Tool Transfer." | "`update.exe` was built locally from memory-resident payload; no network transfer observed. Map to T1027.009 (Embedded Payloads), NOT T1105." |
| "Adversary used `schtasks.exe` → T1053.005." | Confirm the `/create` flag and the payload before committing. `schtasks /query` is pure Discovery (T1083-adjacent), not Persistence. |
| "Registry write → T1547.001 Run Keys." | Check the specific key. `Winlogon\Userinit` is T1547.004, not T1547.001. |

**Rule:** Before committing any technique ID, answer: *"What protocol / flag / key / API call specifically confirms this sub-technique over its siblings?"*

---

## Pitfall 2 — Vague language in the report

CISA guidance: ambiguous narrative language causes downstream mapping errors even when the analyst understood the behavior correctly.

| Draft (ambiguous) | Improved (mapping-safe) |
|---|---|
| "The adversaries moved laterally within the network." | "The adversaries used `pivot.py` to copy payloads through remote file shares (T1570 Lateral Tool Transfer) via SMB admin shares (T1021.002)." |
| "They maintained persistence on infected hosts." | "They created a WMI `__EventFilter` + `CommandLineEventConsumer` binding (T1546.003), triggered on user logon." |
| "The attacker used PowerShell to execute commands." | "The attacker ran `powershell -enc <base64>` (T1059.001) containing an Invoke-Expression download cradle (T1105 + T1027)." |

**Rule:** In published reports, never leave an action described at the tactic level when the evidence supports a technique.

---

## Pitfall 3 — Incomplete search (missing concurrent mappings)

Most non-trivial behaviors map to **two or more** techniques simultaneously. Experienced analysts still miss 20–40% on the first pass. Always ask *"what else is happening here?"*

| Behavior | All applicable mappings |
|---|---|
| HTTP C2 over port 8088 | T1071.001 (Web Protocols) **AND** T1571 (Non-Standard Port) |
| `powershell -enc <base64> IEX (New-Object Net.WebClient).DownloadString(...)` | T1059.001 (PowerShell) **+** T1027 (Obfuscated Files) **+** T1105 (Ingress Tool Transfer) **+** T1140 (Deobfuscate/Decode) |
| Mimikatz run as SYSTEM against LSASS | T1003.001 (LSASS Memory) **+** T1134 (Access Token Manipulation) **+** the original execution technique (T1569.002 Service Execution, T1021, etc.) |
| WMI remote execution of `rundll32` | T1047 (WMI) **+** T1218.011 (Rundll32) **+** T1021 (Remote Services) |
| ISO file mounted and `.lnk` executed | T1204.002 (User Execution: Malicious File) **+** T1553.005 (Mark-of-the-Web Bypass) **+** T1218 (if `.lnk` invokes a LOLBin) |

**Rule:** When you finish a mapping, ask *"what technique have I not yet named?"* — then check the ATT&CK website's technique search for the specific strings you observed.

---

## Pitfall 4 — Tool-name → technique shortcut

Binary names on the ATT&CK procedure examples are the most dangerous shortcut in the framework.

- **Mimikatz** → NOT automatically T1003.001. Check modules: `sekurlsa::logonpasswords` = T1003.001, `lsadump::sam` = T1003.002, `lsadump::dcsync` = T1003.006, `kerberos::golden` = T1558.001, `sekurlsa::tickets` = T1558.003.
- **PsExec** → NOT automatically T1021.002. `psexec.exe \\host -s cmd.exe` hits T1021.002 (SMB/Admin Shares) **+** T1569.002 (Service Execution). The service creation is a separate technique from the SMB transport.
- **Cobalt Strike Beacon** → NOT a single technique. A typical beacon payload maps to 30+ techniques depending on which commands the operator issued. Always map **operator actions**, never "Beacon".
- **rundll32.exe / regsvr32.exe** → T1218.011 / T1218.010 only when used as a proxy. Legitimate registration calls don't belong in ATT&CK at all.

**Rule:** Map the *observed command line, flag, or API call* — never the binary name alone.

---

## Pitfall 5 — Biases in technique selection

| Bias | Symptom | Correction |
|---|---|---|
| **Familiarity bias** | Analyst always maps to techniques they wrote detections for. | Peer review by someone from a different sub-specialty (endpoint vs network vs identity). |
| **Novelty bias** | Analyst gravitates to freshly-added techniques to look current. | Prefer the oldest technique that still fits. New IDs often have ambiguous scope. |
| **Compliance bias** | Analyst stretches mappings to cover "required" cells. | Reject tactic-only mappings rather than inflate them. A blank cell is more honest than a wrong cell. |
| **Tool-vendor bias** | Analyst copies the vendor's mapping from a product console. | Re-derive the mapping from the raw evidence. Vendor mappings are often optimistic. |

---

## Pitfall 6 — Sub-technique confusion

Sub-techniques look similar in the UI but mean very different things.

- **T1027 Obfuscated Files or Information** ≠ **T1027.002 Software Packing**. Packing is *compression with UPX-style loaders*. Base64-encoded PowerShell is T1027, not T1027.002.
- **T1055 Process Injection** has 15 sub-techniques. Classic `CreateRemoteThread` = T1055.002; reflective DLL = T1055.001; process hollowing = T1055.012; APC queue = T1055.004; atom bombing = T1055.009. Each needs its own detection; parent-only mapping is meaningless.
- **T1036 Masquerading** has 7 sub-techniques. Renaming `svchost.exe` → T1036.003 (Rename System Utilities). Invalid code signing → T1036.001. Double-extension `.pdf.exe` → T1036.007 (Double File Extension).
- **T1078 Valid Accounts** has 4 sub-techniques split by *account type*, not *access method*. T1078.001 Default, T1078.002 Domain, T1078.003 Local, T1078.004 Cloud. Red Canary 2024 shows T1078.004 as one of the fastest-growing techniques year-over-year.

**Rule:** Read the sub-technique description on attack.mitre.org *before* committing. The parent page's description rarely matches any single sub-technique.

---

## Pitfall 7 — Cross-tactic sub-techniques

Some sub-techniques live under multiple tactics. Mapping to only one is incomplete.

| Sub-technique | Tactics |
|---|---|
| T1055.001 DLL Injection | Defense Evasion **AND** Privilege Escalation |
| T1548.002 UAC Bypass | Defense Evasion **AND** Privilege Escalation |
| T1543.003 Windows Service | Persistence **AND** Privilege Escalation |
| T1053.005 Scheduled Task | Execution **AND** Persistence **AND** Privilege Escalation |
| T1078 Valid Accounts | Defense Evasion **AND** Persistence **AND** Privilege Escalation **AND** Initial Access |

**Rule:** After mapping a sub-technique, look at its ATT&CK page header — every listed tactic applies. Note all of them.

---

## Pitfall 8 — Tactic ≠ kill-chain order

ATT&CK tactics are *not* a linear sequence. Adversaries skip, repeat, and reorder them. Do not:

- Assume Reconnaissance happens once at the start.
- Assume Execution precedes Persistence (persistence can execute on reboot).
- Treat Lateral Movement as a single step — it typically loops Discovery → Credential Access → Lateral Movement many times.
- Assume Impact is the final stage — destructive wipers sometimes fire early as a distraction.

**Rule:** Map each observed behavior to its tactic independently. Do not infer ordering from ATT&CK column positions.

---

## Pitfall 9 — Version drift

Between versions, MITRE renames, splits, merges, and deprecates techniques. Examples:

- v8 merged PRE-ATT&CK into Enterprise as Reconnaissance (TA0043) + Resource Development (TA0042). Pre-v8 reports referencing PRE-ATT&CK IDs need translation.
- v9 introduced sub-techniques; older reports using `T1086 PowerShell` must be mapped forward to `T1059.001`.
- Group IDs have been merged (APT28 ↔ Fancy Bear ↔ Sofacy ↔ G0007) — confirm you are using the canonical ID.
- Technique descriptions tighten each version. "HTTP" scope on T1071 has narrowed as sub-techniques were added.

**Rule:** Every mapping must record the ATT&CK version. Always cite permalinks of the form `https://attack.mitre.org/versions/v14/techniques/T1105/`, never the bare technique URL. The ATT&CK STIX bundle is versioned on GitHub; pin your analysis to a specific release.

---

## Pitfall 10 — Coverage theater (the meta-pitfall)

Everything above feeds into the biggest trap: producing a map that looks comprehensive without reflecting reality.

**Symptoms of coverage theater:**
- A Navigator layer with every cell scored.
- A "we cover 87% of ATT&CK" slide.
- Multiple detections tagged `T1055` with no sub-technique.
- A red-yellow-green heatmap with no visibility/detection split.
- Detection coverage reported independently of data-source readiness.

**Fix:** Produce **three** layers, overlaid, never one:
1. **Threat-scenario layer** — only the techniques attributed to groups you actually care about (sector + geography + TTP prevalence).
2. **Visibility layer** — what data sources you collect, retention-weighted.
3. **Detection layer** — tuned, validated analytics.

Gaps = Threat-scenario ∖ (Visibility ∩ Detection). Any other definition of "gap" is coverage theater.

---

## Reporting hygiene (CISA-aligned)

Every published mapping must include:

1. **In-line ATT&CK links in the narrative**: "The actor delivered TrickBot via phishing links [T1566.002]" — not dumped in a table at the end.
2. **Summary table** with columns: `Tactic | Technique ID | Technique Name | Use (procedure-level detail)`. The `Use` column must contain *enough* detail that a downstream detection engineer can act on it without re-reading the source.
3. **Navigator JSON layer** attached to the report, not just a screenshot.
4. **Versioned permalinks** on every technique reference.
5. **Parent technique cited alongside sub-technique** on first reference.
6. **At least two analysts' names** on anything published externally. MITRE's own team does two reviews minimum.
