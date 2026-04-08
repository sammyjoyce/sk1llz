---
name: uunet
description: Reason and design like a UUNET backbone engineer or DBIR analyst — running internet-scale IP networks (AS701/702/703 lineage), defending BGP under hostile route leaks, enforcing SRLG-disjoint physical diversity, and producing structured incident analysis with the VERIS 4A framework. Use when designing tier-1/transit backbones, reviewing BGP route policy or peering posture, sizing fiber and conduit diversity, planning carrier-scale DDoS mitigation, or writing DBIR-style breach analysis. Triggers, keywords, and "use when" cues — BGP, AS-path, route leak, AS7007, prefix filter, max-prefix, MANRS, RPKI, peering, settlement-free, depeering, MED, local-pref, hot-potato vs cold-potato, SRLG, fiber diversity, MAE-East, RTBH, VERIS, DBIR, 4A model (Actor/Action/Asset/Attribute), incident vs breach, tier-1 ISP.
tags: bgp, peering, isp, routing, srlg, fiber, backbone, dbir, veris, incident-response, tier1, internet, operations
---

# UUNET Style Guide⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍‌​​​​‌‌​‍​​‌‌​​‌‌‍​‌‌​‌​​​‍‌​​​‌​​‌‍​​​​‌​‌​‍​​​‌​‌​​⁠‍⁠

UUNET was the first commercial ISP, born to dodge the cost of Usenet feeds and grown into the backbone of the commercial internet under AS701 (US), AS702 (Europe), AS703 (Asia-Pacific). Through MCI → WorldCom → MCI → Verizon Business, that same tap point now produces the Data Breach Investigations Report (DBIR). This skill teaches you how an old-school backbone operator and a DBIR analyst actually think — not how the marketing brochure describes them.

## Five non-obvious laws

1. **The blast radius of a single bad announcement is everyone.** On 25 Apr 1997 the AS7007 incident happened because MAI Network Services took 23,000 routes from a downstream peer with **no inbound prefix filter**, a router bug deaggregated them to /24s, the AS-path got stripped, and the resulting more-specifics were preferred globally. Even after MAI unplugged the router, Sprint reported the routes "just keep appearing again" for hours. The lesson is not "filter your peers" — everyone knew that. It is **filter inbound from every neighbor including your customers, and assume your filters will leak.** Use prefix-list AND `maximum-prefix <X> restart 30` AND `maxas-limit 20` together.

2. **"Diverse" is a marketing word until you have audited the conduit.** Two fibers labeled "diverse" routinely share a single backhoe-vulnerable trench, the same building riser, the same regen hut, or the same DWDM mux. Real diversity is **SRLG-disjoint**: shared-risk-link-group-disjoint at every layer including splice points and power feeds. The k-diverse routing problem with general SRLGs is **NP-complete** — there is no algorithm that gives you the answer; you need the carrier's GIS database, not their sales engineer's diagram. A typical long-haul link can belong to 50,000+ SRLGs.

3. **Capacity must outpace traffic, not chase it.** Mike O'Dell observed at UUNET that the backbone's capacity grew *faster* than its offered load — and this was deliberate. Once a backbone link passes ~50% utilization, queueing delay becomes nonlinear and asymmetric, latency-sensitive flows starve, and the symptoms look like a software bug. **Provision so that any single link or POP can fail and the survivors stay below ~50%.** "Build for tomorrow's peak" is too late.

4. **Hot-potato routing makes your network observable but asymmetric.** Tier-1 backbones default to hot-potato (hand traffic to your peer at the nearest exchange) because it minimizes your transit cost. The hidden tax: the forward and reverse paths are on different networks, so traceroute lies, MTR shows phantom loss, and your customer blames you for your peer's congestion. **Cold-potato** (carry the traffic on your own backbone to the closest egress to the destination) costs more in fiber but gives you predictable RTTs and SLA-able performance. Pick deliberately, per peer; never let your IGP metrics decide for you by accident.

5. **The classification taxonomy IS the deliverable.** The DBIR is not "another security report" — it is the only large-scale incident corpus where data from 80+ contributors aggregates cleanly, because everyone classifies into the **VERIS 4A model**: Actor, Action, Asset, Attribute. If you cannot fill all four for an incident, you do not understand it yet, and that gap *is* the finding. Distinguish **incident** (any compromise of confidentiality/integrity/availability) from **breach** (incident with confirmed data disclosure) — the words are not interchangeable and lawyers, regulators, and insurers will lock onto the wrong one.

## Heuristics — ask yourself before acting

**Before approving any BGP policy change**, ask:
- What is `maximum-prefix` set to on this neighbor and what does the router do when it trips? `warning-only` is a footgun; **tear-down with `restart 30` is the safe default**.
- Do I have an inbound prefix-list on EVERY neighbor including customers and the loopback for iBGP? AS7007's downstream did not.
- Am I propagating MEDs across an AS boundary I do not control? (Don't. MED is only meaningful between two ASes that have explicitly agreed.)
- Is my LOCAL_PREF policy explicit? Standard ladder: `customer=200, peer=100, transit=50`. Higher LP wins regardless of AS-path length — that is the whole point.
- If I dampen flaps, am I using RFC 7196 parameters (`half-life 10m, suppress 6000`) or the original RFC 2439 defaults that strangled legitimate routes for years?
- Will this change announce a more-specific that I am not also announcing as a covering aggregate? **Never let a customer flap propagate globally.**

**Before claiming "physically diverse"**, ask:
- Have I verified SRLG at the **building entry**, not just the metro?
- Are my "diverse" feeds on different **power buses, different DC suites, different regen huts, and different conduit easements**?
- Do my A/B paths share even one undersea cable, exchange point, or carrier-hotel meet-me room? If yes, that *is* the SPOF.
- 100% SRG-disjoint is impossible at the customer last-mile; document the residual risk explicitly in the SLA instead of hiding it.

**Before publishing an incident analysis**, ask:
- Can I name the **Actor** (variety: organized crime, state-affiliated, internal employee, partner), **Action** (Malware/Hacking/Social/Misuse/Physical/Error/Environmental), **Asset** (Server/User-device/Person/Network/Media), and **Attribute** (Confidentiality/Integrity/Availability)? Any "Unknown" is a finding, not a hole.
- What is my **n**? <5 → no number quoted. <30 → relative comparisons only, never percentages. ≥30 → quote the 95% confidence interval, not the point estimate. The DBIR enforces this rule in its methodology appendix; you should too.
- Is this an **incident** or a **breach**? The 2025 DBIR analyzed ~22,000 incidents but only ~12,000 confirmed breaches — a 2× difference that matters for headlines, contracts, and regulators.

## NEVER do this

- **NEVER accept routes from a customer without an inbound prefix-list** because "they're our customer, they trust us." The seductive logic is that customers buy a /22 from you so why filter? Because their CPE will eventually misconfigure, leak the global table back, and you will be the one announcing it to the internet (this is exactly what AS7007's upstream did). **Instead**: prefix-list bound to the customer's RIR allocation, plus `maximum-prefix` at 1.5× their allocation count, plus `maxas-limit 20`, plus an AS-path filter that only allows their AS at the origin.

- **NEVER redistribute BGP into your IGP (or IGP into BGP)** because "it'll make troubleshooting easier." It will not. The current IPv4 table is ~960k prefixes; injecting that into IS-IS or OSPF will overflow LSP databases, run SPF in a tight loop, and brick every router in the IGP simultaneously. **Instead**: IGP carries only loopbacks and infrastructure /31s (a few hundred prefixes max), iBGP carries everything else, and `next-hop-self` on edge routers means the IGP never has to know about peering /30s.

- **NEVER deaggregate your /19 to /24s during a flap** because "more-specifics give my customer faster failover." More-specifics propagate globally, get dampened by *other* networks under their broken RFC 2439 settings, and the resulting churn punishes your customer for 10–20 minutes per flap. **Instead**: announce the cover aggregate persistently with a `null0` pull-up route, and let iBGP carry the more-specifics internally only. This is why CIDR Report rankings exist.

- **NEVER trust default route-flap dampening parameters** because "RFC defaults must be safe." They are not — RFC 2439's parameters were so aggressive that legitimate routes flapping once stayed suppressed for an hour, and the IETF effectively recanted in RFC 7196. **Instead**: either disable dampening entirely (the modern default for tier-1s) or set `half-life 10m, reuse 750, suppress 6000, max-suppress 60m`.

- **NEVER mix peer and transit traffic on a settlement-free peering link** because "bandwidth is bandwidth." Settlement-free peering means *equality* — you only send your peer traffic destined to *its customers*, not transit to elsewhere. Doing otherwise is "transit theft" and is the canonical depeering trigger (Sprint vs Cogent in October 2008 partitioned the internet for ~3 days; Cogent vs Hurricane Electric is still ongoing for IPv6). **Instead**: tag prefixes you receive from each peer with a community, and only export those community-tagged prefixes back over peering links.

- **NEVER use uRPF (unicast reverse-path forwarding) on an IXP-facing interface** because "it stops spoofing." It does — and it also drops every legitimate asymmetric flow that hot-potato routing creates, which is most of them. **Instead**: use ACLs that permit only traffic destined to your own address space on the IXP interface.

- **NEVER quote a DBIR percentage from a sample of <30** because the report explicitly forbids it and you will be cited as misrepresenting the data. **Instead**: state "in n=22 cases, ransomware appeared in roughly half" or quote the 95% confidence interval. Read the slanted-bar charts: **if two slanted bars overlap, you cannot say one is bigger.**

- **NEVER conflate "incident" and "breach" in a security report** because the words have a regulatory definition. An incident is any CIA compromise. A breach is an incident with **confirmed disclosure** of data. The 2025 DBIR's headline ratio (12k breaches inside 22k incidents) is the entire point of using VERIS in the first place.

## Decision tree — when something is broken

1. **BGP session won't come up** → check (a) MD5 password mismatch, (b) eBGP TTL=1 dropped by GTSM (use `ttl-security hops 1` or set TTL to 254), (c) `maximum-prefix` tripping immediately on first update because the peer sends more than your limit, (d) MTU/MSS clamping on a tunneled or DSL-backhauled link.
2. **Route is in BGP but unreachable** → 95% of the time the issue is recursive next-hop lookup failing because you forgot `next-hop-self` on the edge router, so internal routers see a next-hop on a peering /30 they have no IGP route for.
3. **Customer complains of asymmetric latency** → it is hot-potato routing meeting cold-potato routing on the return path. Confirm with traceroute in both directions; the fix is either (a) coordinate AS-path prepending with the peer or (b) renegotiate to a cold-potato peering arrangement.
4. **Edge under DDoS volume your routers cannot police** → do not filter at the edge (you will melt the line cards). Trigger **RTBH (Remotely Triggered Black Hole)** by announcing the victim /32 internally with community `65535:666` so every edge router drops it before deep inspection. For more surgical mitigation use Flowspec (RFC 5575).
5. **Incident analysis stuck — "infrastructure was compromised"** → you are at the wrong granularity. Drop down to a specific Asset (Server vs User device vs Person vs Network device vs Media) and a specific Action variety (Phishing vs Use of stolen credentials vs Exploit vuln vs Misconfiguration). "Infrastructure" is not a VERIS asset.

## When to read the deep references

- **Before doing root-cause analysis on any BGP route leak, hijack, or "internet was down"-class outage, READ `references/route-leak-forensics.md`.** It contains the AS7007 timeline, the canonical filter-stack template, RPKI/MANRS posture, and what to look for in a leak vs a deliberate hijack.
- **Before entering peering negotiations, planning a depeering, or sizing a new POP, READ `references/peering-economics.md`.** It contains the settlement-free ratio rules, the modern post-tier-1 reality (Google/Netflix/Meta have flattened the hierarchy), and the historical depeering wars worth knowing.
- **Before writing any DBIR-style or VERIS-classified incident analysis, READ `references/veris-classification.md`.** It contains the 4A enumerations, sample-size discipline, the bias-acknowledgment template, and the incident-vs-breach decision rule.
- **Do NOT load the references for general philosophy questions, code style critiques, or naming-convention advice.** They are deep-dive material, not background reading.
