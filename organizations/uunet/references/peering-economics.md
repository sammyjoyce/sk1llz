# Peering Economics — Deep Reference⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​‌‌‌​‌‍‌‌‌​​​​‌‍‌‌​‌​​‌‌‍​​​‌‌​‌‌‍​​​​‌​‌​‍‌​​​​‌​​⁠‍⁠

Load this file ONLY when entering a peering negotiation, planning a depeering, sizing a new POP, or analyzing inter-carrier traffic flows.

## What "tier-1" actually means

A **tier-1 network** is one that can reach the entire internet (both IPv4 and IPv6) **using only settlement-free peering** — it never has to pay any other network for transit. The classical list (always changing): AT&T (AS7018), Lumen/Level 3 (AS3356), Verizon Business/UUNET (AS701/702/703), NTT (AS2914), Telia/Arelion (AS1299), Tata (AS6453), GTT (AS3257), Cogent (AS174), Zayo (AS6461), Telxius (AS12956), PCCW (AS3491), Orange (AS5511), Sparkle (AS6762).

This list **shrinks every year** because two things have happened since ~2010:

1. **Hyperscaler flattening.** Google (AS15169), Netflix (AS2906), Meta/Facebook (AS32934), Amazon (AS16509), Microsoft (AS8075), Apple (AS714), and Cloudflare (AS13335) now run their own global backbones. They reach end users via direct PNI (private network interconnect) with eyeball networks (Comcast, Verizon FiOS, Deutsche Telekom, Reliance Jio, etc.), bypassing the tier-1s entirely. Traffic that used to traverse three tier-1 backbones now traverses zero.
2. **CDN ubiquity.** A surprising amount of "internet traffic" never enters the public internet at all — it goes from Akamai/Cloudflare/Fastly's cache running inside the eyeball ISP's metro, directly to the user, over the eyeball ISP's metro fiber.

The practical consequence: being a tier-1 today is more about brand and legacy than capability. The companies with the most leverage are the ones with the most **eyeballs** (last-mile customers) — they can demand paid peering from the content providers, and the content providers usually pay.

## Settlement-free peering criteria (the unwritten rules)

Tier-1s do not publish formal peering policies but the de facto criteria are:

- **Backbone of similar reach**: roughly equivalent footprint, ideally on multiple continents.
- **Multiple geographically diverse interconnect points**: minimum 3 in North America, 3 in Europe, 2 in Asia. This forces hot-potato routing to actually distribute load.
- **Traffic ratio within ~1.8:1 to 2.0:1** (varies by network). Networks with too much outbound (content-heavy) get told to either bring more eyeball traffic or pay. This is the entire reason Netflix and Cogent fought every tier-1 for a decade.
- **Capacity at each interconnect**: typically a minimum of 100 Gbps per interconnect point as of ~2024, growing to 400 GbE.
- **24×7 NOC** with documented escalation procedures.
- **Mutual non-disclosure of the agreement** (this is the legal foundation of "settlement-free").

If you do not meet these, you pay for transit. If you barely meet them, you might get **paid peering** — you exchange traffic directly but the smaller party pays a per-Mbps rate that is cheaper than transit but not free.

## Famous depeering wars

These are the cases worth knowing because they shaped current peering practice.

| Year | Parties | Outcome |
|---|---|---|
| 2005 | Level 3 vs Cogent | Level 3 cut Cogent off; ~3 days of partition; Level 3 backed down after customer revolt. |
| 2008 | Sprint vs Cogent | Same playbook. ~3 days of partition; ended in mutual agreement to peer at lower ratios. |
| 2009→ ongoing | Cogent vs Hurricane Electric (IPv6) | Still not peered as of 2025. The IPv6 internet is partitioned for any path that depends on both. |
| 2010 | Cogent vs Telia | Ratio dispute. Settled within days. |
| 2014 | Comcast vs Netflix | Comcast deliberately let Cogent↔Comcast congest until Netflix paid for direct PNI. Triggered the entire net-neutrality debate of 2014–2015. |
| 2014 | Verizon vs Level 3 | Verizon refused to upgrade Level 3 ports until Level 3 paid for the upgrade — same congestion-as-leverage tactic. Level 3 published the port utilization graphs publicly to embarrass Verizon. |

**Lesson**: depeering is rarely about technology. It is a commercial negotiation conducted by deliberately degrading customer experience until the weaker party pays. As an engineer in the middle, you cannot fix it with config; the fix is at the contract layer.

## Hot-potato vs cold-potato in detail

**Hot-potato** (default for tier-1s): you hand traffic to your peer at the first interconnect you can. You minimize your own backbone load, your peer carries the long-haul cost. The hidden cost is asymmetric paths — return traffic comes back via a different exchange, so RTT is hard to predict and traceroute output is misleading.

**Cold-potato**: you carry the traffic on your own backbone all the way to the egress closest to the destination, only handing off at the last possible exchange. You eat the long-haul cost; in exchange you get full control over latency and can offer SLAs.

How to force one or the other:
- **You want hot-potato out**: advertise the same prefix with identical attributes (same MED, same communities, same AS-path length) at every interconnect with the peer. The peer's BGP best-path will pick the closest one.
- **You want cold-potato out** (i.e. you carry traffic to the closest egress to the destination): advertise more-specific prefixes only at the geographically appropriate exchange, and aggregate prefixes everywhere. Or use MED — if your peer accepts MED, set lower MED at the exchange you want them to send traffic to.
- **Get your peer to use cold-potato into your network**: ask them to honor MED and tag the prefixes with a community that means "send to this region." Some peers will, most will not.

**Cogent famously runs cold-potato** for content destined to its eyeballs — that is part of why ratio disputes hit them hardest, because they "carry too much."

## Geographic diversity and PoP siting

Where you locate POPs matters more than how many you have. The classical mistakes:

1. **Co-locating both "diverse" POPs in the same carrier-hotel building** (e.g. 60 Hudson Street, 1 Wilshire, Telehouse Docklands). The building IS the SRLG. Cooling failure or fiber-cut to the street takes both out.
2. **Routing both "east" and "west" trans-Atlantic via the same submarine cable** (TAT-14, AC-1, etc.). Submarine cables get cut by anchors and trawlers regularly; redundant capacity must be on different cable systems.
3. **Buying "diverse" metro fiber from the same incumbent**. They lit it on the same DWDM ring. The ring is the SRLG.
4. **Putting a peering exchange at sea level in a hurricane zone**. MAE-East (Tysons Corner, VA) and MAE-West (San Jose) became single-points-of-failure for the early commercial internet for exactly this reason.

The right pattern is **N+1 with explicit SRLG-disjointness**: any single POP, any single building, any single submarine cable, any single fiber pair, or any single power feed can fail and the rest of the network absorbs the load while staying below 50%.

## Peering at IXPs — the bilateral vs route-server choice

At an IXP (DE-CIX, AMS-IX, LINX, Equinix Ashburn, etc.) you can peer two ways:

- **Bilateral**: explicit eBGP session with each peer. Maximum control. Required for serious tier-1 peering.
- **Route-server (multilateral)**: one BGP session with the IXP's route server, which redistributes everyone's prefixes to everyone else. Convenient for small networks. **Tier-1s do not peer via route-servers** because they need to enforce per-peer policy.

Use both: route-server for the long tail of small peers, bilateral for the few big ones that actually carry your customer traffic.
