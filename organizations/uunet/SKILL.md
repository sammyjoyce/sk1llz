---
name: uunet
description: "Think like a UUNET/Verizon backbone operator or DBIR analyst when internet-scale routing, interconnection, physical diversity, or breach-classification mistakes have large blast radius. Use when judging BGP route leaks, RPKI/ROV scope, BGP Role/OTC rollout, hot-potato vs cold-potato delivery, SRLG diversity claims, DDoS blackholing, peering disputes, or VERIS/DBIR-style incident analysis. Triggers: AS701, AS702, AS703, route leak, hijack, max-prefix, OTC, BGP Role, RPKI, maxLength, hot potato, cold potato, peering, depeering, SRLG, diverse circuits, RTBH, VERIS, DBIR, incident vs breach."
tags: bgp, routing, peering, rpki, srlg, backbone, tier1, rtbh, veris, dbir, incident-response
---

# UUNET⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​​​‌‌​‍​​‌‌​​‌‌‍​‌‌​‌​​​‍‌​​​‌​​‌‍​​​​‌​​‌‍‌‌​​​​‌​⁠‍⁠

UUNET thinking is about economic routing under unsafe failure modes. Assume every control-plane mistake can become a global event, every "diverse" circuit hides shared facility risk, and every breach statistic will be challenged by someone with money or lawyers.

## Core Lens

- Route leaks are usually policy failures, not origin-auth failures. ROV catches forged origins; it does not stop a policy leak that rides on a legitimate origin or a permissive ROA `maxLength`.
- Hot-potato asymmetry is the normal state of the core. CAIDA measured only about `2-5%` symmetric tuples on Tier1 backbone links, so one-sided traces are weak evidence and strict uRPF on public peering edges is usually wrong.
- Interconnect congestion is often a business decision before it is a physical limit. Many policies still augment around `70%`; Google and Microsoft publish `50%`-style interconnect triggers. Size N-1 capacity so a failure does not turn a contract dispute into packet loss.
- "Path diversity" fails at building entry, MMR, regen hut, and power bus. Intertubes found `63.28%` of conduits shared by at least three major ISPs; a carrier diagram without SRLG proof is marketing, not engineering.
- BGP Role and OTC help only on normal relationships. RFC 9234 says complex peerings should not use Roles; split the relationship into separate sessions or stay with explicit per-prefix policy.
- DBIR discipline is about defensible claims, not elegant prose. The 2025 DBIR had more than `22,000` incidents but only `12,195` confirmed breaches, and third-party involvement doubled from `15%` to `30%`.

## Before You Act

### Before approving eBGP or leak-mitigation changes, ask yourself:

- Is this an RFC 7908 Type 1-4 policy leak, a Type 5 re-origination, or a Type 6 internal more-specific leak? Your safe first move depends on the class.
- If the route is RPKI-valid, is that because it is actually safe, or because a broad `maxLength` made the harmful more-specific look legitimate?
- If `maximum-prefix` fires, do I want a warning, dropped excess, or a hard reset? `warning-only` preserves convenience and also preserves poison.
- Is the relationship truly Provider, Customer, or Peer, or is it complex for some prefixes? If it is complex, do not paper over it with Roles or OTC.
- Will RTBH or blackhole specifics stay ROV-valid? If you expect `/32` or `/128` mitigation, pre-authorize those specifics before the attack.

### Before accepting "physically diverse" or planning a new POP, ask yourself:

- Where do the circuits diverge: street conduit, building entry, meet-me room, rack row, power feed, regen site, or only on the sales slide?
- Does one contractor, easement, or carrier hotel still take out both paths?
- After any single failure, do surviving ports stay below your augment threshold, or do you just move congestion to the next handoff?
- Is the request for cold-potato delivery about latency, or is the other party asking you to subsidize their backbone?

### Before publishing DBIR-style analysis, ask yourself:

- Can I fill Actor, Action, Asset, and Attribute without hand-waving? "Infrastructure issue" and "user mistake" are not VERIS classes.
- Is this an incident or a breach? If confidentiality loss is not confirmed, do not upgrade the noun.
- Is the sample big enough to survive scrutiny? `<5` means keep quiet; `5-29` means counts or relative comparisons only; `>=30` is when percentages and confidence intervals become defensible.

## Decision Tree

| Situation | First discriminator | Safe first move | If that fails |
|---|---|---|---|
| New detour, origin still looks valid | More-specific appeared? | Check ROA `maxLength`, export policy, and RFC 7908 leak class before calling it a hijack | Clamp the neighbor with prefix policy or `maximum-prefix`, then inspect OTC and Role propagation |
| Sessions fail after Role/OTC rollout | Capability missing or strict-mode mismatch? | Revert to non-strict Role negotiation and keep local filters | Split complex relationships into separate sessions; do not force Roles onto mixed-policy links |
| RTBH announcement ignored | Blackhole prefix ROV-valid and community correct? | Verify prebuilt ROAs for blackhole specifics and provider community semantics | Fall back to provider-managed scrubbing or aggregate blackhole with explicit customer approval |
| Latency complaint with clean local optics | Two-sided evidence or one-sided traces only? | Measure both directions and assume hot-potato asymmetry until disproven | Renegotiate interconnect location, MED/community handling, or paid cold-potato delivery |
| Security write-up stuck between outage and breach | Confirmed disclosure and adequate `n`? | Classify as incident, state the unknowns, and use 4A gaps as the worklist | Escalate evidence collection instead of polishing the summary |

## NEVER Do This

- NEVER treat RPKI-valid as "safe" because broad `maxLength` values authorize harmful more-specifics and policy leaks still propagate on valid origins. Instead combine ROV with exact-match ROAs, prefix filters, `maximum-prefix`, and BGP Role or OTC where the relationship is normal.
- NEVER turn on BGP Role strict mode everywhere because it feels cleaner; RFC 9234 warns sessions may fail to return after a software update or with peers that do not advertise the capability. Instead stage Roles in non-strict mode, inventory peer support, then enforce selectively.
- NEVER use `warning-only` or lazy `maximum-prefix` headroom on customer or transit edges because it avoids noisy turn-up failures while letting poisoned tables stay in session. Instead set hard ceilings from observed steady state plus explicit blackhole and maintenance allowance, and prefer automatic restart over indefinite acceptance.
- NEVER revive RFC 2439-style damping defaults because they feel like hygiene but suppress legitimate reachability long after the flap is over. Instead use RFC 7196-style bounds: suppress threshold at least `6000`, conservative `12000`, router max penalty at least `50000`, and test mode before live damping.
- NEVER believe a carrier's "diverse" label because shared conduits, MMRs, regen huts, or power buses survive every procurement slide and still create one failure domain. Instead demand SRLG evidence down to facility ingress and write the residual shared assets into the SLA.
- NEVER run strict uRPF on IXP or hot-potato-facing interfaces because Tier1 cores are intentionally asymmetric; CAIDA's backbone measurements show that symmetric flows are the exception, not the rule. Instead use strict or feasible-path uRPF on customer edges and ACL plus source policy on public peering.
- NEVER let network automation push route-policy changes everywhere at once because a single bad model turns into network-wide state instantly. Instead canary on a small peer set, diff intended vs observed advertisements, and keep a fast rollback that does not depend on the same automation plane.
- NEVER call something a breach because the incident feels severe. That wording is seductive in executive summaries and legally expensive when confidentiality loss is still unconfirmed. Instead keep "incident" until disclosure is proven, and keep percentages out of `n<30` datasets.

## Loading Triggers

- Before route-leak forensics, hijack review, or "internet was down" postmortems, read [route-leak-forensics.md](./references/route-leak-forensics.md).
- Before peering negotiations, depeering analysis, interconnect augments, or hot-potato vs cold-potato decisions, read [peering-economics.md](./references/peering-economics.md).
- Before VERIS classification, DBIR-style writing, or incident-vs-breach language, read [veris-classification.md](./references/veris-classification.md).
- Do NOT load the references for router CLI syntax, app-layer security triage, or generic "how BGP works" tasks.

## Freedom Boundary

- Treat BGP safety controls, ROV or ROA scope, and incident labeling as hard constraints; improvisation here creates outages or legal problems.
- Treat peering posture, cold-vs-hot potato, and capacity targets as heuristics; the right answer depends on geography, contracts, and who owns the eyeballs.
