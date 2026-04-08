# VERIS Classification — Deep Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​‌​​​​‍‌​​​​‌​‌‍‌​​​​‌​​‍‌‌​‌‌‌‌​‍​​​​‌​‌​‍‌‌​​‌‌‌‌⁠‍⁠

Load this file ONLY when writing a DBIR-style incident analysis or classifying real incidents into the VERIS framework. The framework is the open standard maintained at <https://verisframework.org> and <https://github.com/vz-risk/veris>.

## The 4A model — every incident must classify all four

### A1 — Actor: who did this?

Three top-level varieties, in mutually exclusive order:

- **External**: not part of the org or its trusted partners. Sub-varieties include `Organized crime`, `State-affiliated`, `Activist`, `Former employee`, `Unaffiliated`, and the joke ones — `Acts of God`, `Mother Nature`, `Random chance`. (External does not necessarily mean malicious.)
- **Internal**: anyone with implicit trust granted by the organization — full-time employees, contractors, interns, executives. The DBIR specifically calls out that internal actors are not always malicious; the most common internal action is `Misdelivery` (sending an email to the wrong recipient).
- **Partner**: a third party with some level of trust granted by formal contract — vendors, suppliers, integrators, hosting providers. The 2025 DBIR shows this category has *doubled* year over year (~30% of breaches involve a partner) and is the fastest-growing actor variety.

Common pitfall: classifying a phishing victim as the actor. The phishing victim is the **asset** (`Person`); the phisher is the **actor**.

### A2 — Action: how did they do it?

Seven mutually-exclusive top-level categories. Pick exactly one **primary** action even if multiple were involved (you record the others as secondary).

| Category | Definition | Common varieties |
|---|---|---|
| **Malware** | Any malicious software | Ransomware, RAT, Backdoor, Spyware/Keylogger, Capture stored data, C2, Downloader |
| **Hacking** | Attempts to intentionally access or harm an asset without authorization | Use of stolen creds, Exploit vuln, Brute force, SQLi, Backdoor (use), MitM |
| **Social** | Deception, manipulation, intimidation of human users | Phishing, Pretexting, Bribery, Influence, Forgery, Spam |
| **Misuse** | Unapproved use of legitimate access | Privilege abuse, Data mishandling, Email misuse, Possession abuse |
| **Physical** | Threats originating from the physical world | Theft, Tampering, Surveillance, Snooping, Connection |
| **Error** | Anything done (or not done) incorrectly or accidentally | Misdelivery, Misconfiguration, Publishing error, Disposal error, Loss, Programming error |
| **Environmental** | Power, hazards, weather, earthquakes, EMP, etc. | Power failure, Hazard, Earthquake, Flood, Fire |

Decision rules that catch people out:
- **Ransomware is `Malware/Ransomware`, not `Hacking`.** The ransomware deployment is the action that defines the breach.
- **Phishing that delivers malware is two actions**: `Social/Phishing` (the entry vector) and `Malware/<variety>` (the impact). Pick the one that *defines the breach* as primary — usually the malware.
- **"Accidentally exposed S3 bucket" is `Error/Misconfiguration`**, not `Hacking`. There was no unauthorized access — the data was authorized to the world by the misconfiguration.
- **"Programming error in CrowdStrike update brought down 8 million machines" was `Error/Programming error`** with vector `Software update`, attributed to the **Partner** actor (CrowdStrike). This is the canonical 2024 example; use it as a sanity check.

### A3 — Asset: what was affected?

Top-level categories with their nuances:

- **Server**: web app server, database server, file server, mail server, DNS server, DHCP server, etc. Most common asset in `System Intrusion` patterns.
- **Network**: routers, switches, IDS, firewalls, load balancers, telephony equipment. Verizon includes its own carrier equipment here.
- **User device**: laptop, desktop, mobile phone, tablet, kiosk, POS terminal. Common in `Lost and Stolen Assets` and BYOD-driven `System Intrusion`.
- **Person**: yes, people are assets in VERIS — they are the "where" affected by social actions. Common varieties: `End-user`, `Finance`, `Executive`, `System admin`, `Developer`. The Finance person variety has been trending up alongside BEC pretexting.
- **Media**: USB sticks, CDs, paper documents, payment cards, hard drives. Niche but important for `Lost and Stolen Assets` and physical-pattern incidents.
- **Kiosk/Terminal**, **Embedded**: ATMs, IoT, OT/ICS.

If your finding is "infrastructure was compromised," you are at the wrong granularity — drop down to a specific server / device / person.

### A4 — Attribute: how was the asset affected?

The CIA triad, plus a fourth (sometimes) for accountability:

- **Confidentiality**: data was viewed or copied that should not have been. This is what makes an incident a *breach*. Sub-varieties: `Personal`, `Credentials`, `Internal`, `Medical`, `Bank`, `Payment`, `Secrets`, `Classified`, `Source code`, `System`, `Other`.
- **Integrity**: data was modified, software was altered, fraudulent transactions occurred. Sub-varieties: `Software installation`, `Modify configuration`, `Alter behavior`, `Fraudulent transaction`, `Repurpose`, `Created account`.
- **Availability**: asset cannot be accessed when needed. Sub-varieties: `Destruction`, `Loss`, `Interruption`, `Degradation`, `Acceleration`, `Obscuration`.

Same Action can affect multiple attributes — ransomware almost always hits both Availability (encryption) and Confidentiality (exfiltration in double-extortion). Record both.

## The incident-vs-breach distinction

This is the single most-misused pair of words in the security industry. Get it wrong and you will be cited.

- **Incident**: any security event that compromises the integrity, confidentiality, or availability of an information asset. A blocked phishing attempt that you detected is an incident. A ransomware attempt that your EDR caught is an incident.
- **Breach**: an incident with **confirmed disclosure** of data to an unauthorized party — i.e., a confirmed Confidentiality attribute violation. Ransomware that encrypts but does not exfiltrate is an *incident, not a breach*. Ransomware with double-extortion is a breach.

The 2025 DBIR analyzed roughly 22,000 incidents but only ~12,000 were classified as breaches. The ratio matters because most regulators (GDPR, HIPAA, US state laws, SEC cyber disclosure rules) trigger only on **breach**, not incident.

## Sample-size discipline

The DBIR's methodology appendix is the gold standard for honest security statistics. Adopt these rules verbatim:

| n | What you may say |
|---|---|
| <5 | Nothing. Do not even mention the count. |
| 5–29 | Relative comparisons only ("more than", "less than", "in the majority of these cases"). **No percentages, no "most common."** |
| ≥30 | Quote the **95% confidence interval**, not the point estimate. Use slanted-bar charts: if two slanted bars overlap, you may not say one is bigger. |

When making time-trend claims, use the spaghetti-chart visualization: the threads represent the possible connections within each year's confidence interval. Loose threads = wide interval = small n = not actually a trend.

## The bias-acknowledgment template

Every honest incident report should include a section like this:

> **Acknowledgment of bias.** This dataset is sourced from [N] contributors whose populations are not random samples of the universe. Contributors are biased toward [forensic IR firms / cyber-insurance claimants / a single ISP's customer base / law enforcement notifications / etc.]. As a result, the dataset over-represents [breaches detectable via X] and under-represents [breaches detectable only by Y]. Specifically: [list the known directional biases — e.g., "ransomware is over-represented because actor disclosure makes it easy to track; quiet espionage is under-represented because it is harder to detect"].

Without this section, you are making "creative exploration" claims that look like "causal hypothesis testing." The DBIR explicitly calls itself the former.

## Patterns vs raw 4A

Once you have classified individual incidents, the next layer of abstraction is the **Incident Classification Pattern** — a clustering of incidents with similar 4A signatures. The eight current DBIR patterns:

1. **System Intrusion** — multistep with malware/hacking actions, complex attacker tradecraft.
2. **Social Engineering** — phishing, pretexting, BEC. Often just one or two steps.
3. **Basic Web Application Attacks** — single-step web exploitation, often using stolen credentials.
4. **Miscellaneous Errors** — non-malicious accidents (`Misdelivery`, `Misconfiguration`, `Publishing error`).
5. **Privilege Misuse** — internal actors abusing legitimate access.
6. **Lost and Stolen Assets** — physical loss/theft of devices or media.
7. **Denial of Service** — availability-only events; usually no Confidentiality or Integrity component, hence rarely "breaches."
8. **Everything Else** — the bucket for incidents that don't fit cleanly.

When writing a report, choose the right altitude: lay readers want pattern-level findings ("most breaches were Social Engineering"); technical readers want 4A-level findings ("the most common Action variety in this dataset was `Hacking/Use of stolen credentials`, n=4,287, 95% CI [38%, 42%]").

## Tooling

- **VERIS Webapp**: free JSON-based incident recording UI from Verizon. <https://github.com/vz-risk/veris-webapp>
- **VERIS Community Database (VCDB)**: open dataset of public-knowledge incidents recorded in VERIS. <https://github.com/vz-risk/VCDB>
- **VERIS↔ATT&CK mapping**: bidirectional translation maintained by the MITRE Center for Threat-Informed Defense, useful when integrating with detection content. <https://center-for-threat-informed-defense.github.io/mappings-explorer/external/veris/>
- **VERIS↔CIS Controls mapping**: maps each incident pattern to the relevant CIS Critical Security Controls (Implementation Group 1 = "Essential Cyber Hygiene"). Useful when translating findings into "what to do next."
