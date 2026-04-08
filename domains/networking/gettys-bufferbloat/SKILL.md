---
name: gettys-bufferbloat
description: >-
  Engineer low-latency networks in the style of Jim Gettys, discoverer of
  bufferbloat. Provides expert decision frameworks for diagnosing "dark buffers,"
  choosing between fq_codel/CAKE/SQM, tuning AQM parameters for specific link
  types (DOCSIS, WiFi, fiber, cellular), and hunting hidden latency sources.
  Use when diagnosing network latency under load, configuring SQM/AQM,
  debugging why fq_codel or CAKE isn't helping, sizing buffers, fixing WiFi
  latency, or building real-time/interactive network applications. Triggers:
  bufferbloat, latency under load, fq_codel, CAKE, SQM, AQM, queue management,
  tc qdisc, network quality, loaded latency, jitter, WiFi lag, CoDel.
---

# Gettys Bufferbloat — Expert Networking Skill

## The Gettys Mindset

Before touching any network configuration, ask yourself:

1. **Where is the actual bottleneck right now?** Buffers are "dark" — they only
   cause harm when they sit directly before a saturated link. If the bottleneck
   moves (e.g., ISP upgrades bandwidth and now WiFi is the bottleneck), your
   previously-working AQM becomes irrelevant and latency gets *worse*.
2. **Am I measuring under load?** Idle RTT is a lie. The only metric that matters
   is RTT while the link is saturated in both directions simultaneously.
3. **How many places do buffers hide in this path?** NIC ring buffers, driver
   TX queues, WiFi firmware aggregation queues, tc qdiscs, socket buffers, VM
   layers, CPE/modem, CMTS/DSLAM, every router line card, VPN/tunnel endpoints,
   firewall appliances. Each is a latency landmine when it sits before a
   bottleneck. Gettys calls these "dark buffers" — invisible until saturated.

## Decision Tree: Which AQM Solution

| Scenario | Use | Why not the other |
|---|---|---|
| ISP CPE you control (OpenWrt) | **CAKE** with SQM | Handles per-host fairness, DOCSIS framing overhead, ATM compensation, DSCP priority in one qdisc |
| Linux server/router, no shaping needed | **fq_codel** as default qdisc | Zero-config, works at line rate with BQL, no bandwidth parameter needed |
| Must shape below link rate (asymmetric ISP) | **CAKE** with `bandwidth` set to 85-95% of real speed | fq_codel alone can't shape; needs HTB parent which is harder to configure correctly |
| Variable-speed link (Starlink, LTE, cable) | **CAKE + cake-autorate** | Static bandwidth settings oscillate between throughput loss and bloat; autorate measures and adjusts continuously |
| Enterprise gear (Cisco/Juniper/Arista) | Push vendor for FQ-AQM; fallback to WRED | Most commercial routers still lack fq_codel/CAKE; L4S is insufficient — it relies on cooperative endpoints |
| WiFi AP (OpenWrt mac80211) | Airtime Fairness + AQL + per-station fq_codel | Solves the "WiFi anomaly" where one slow client destroys everyone's airtime |

## Critical Parameters Most People Get Wrong

**fq_codel target/interval defaults (5ms/100ms) fail on DOCSIS links.**
CableLabs research shows DOCSIS adds inherent scheduling delay. Use target
10-20ms, interval 150ms for cable. The default 5ms target causes CoDel to
drop aggressively before the cable scheduler even delivers packets, destroying
throughput without improving latency.

**SQM bandwidth must be set BELOW the real link rate — but how far below
matters.** Set too close (>95%): the ISP's upstream queue still fills, your SQM
is useless. Set too low (<80%): you're leaving bandwidth on the table. Start at
85-90% of measured speed. For CAKE with `docsis` keyword, you can push to ~99%
because it accounts for DOCSIS framing overhead precisely.

**codel-quantum must not exceed MTU.** A common misconfiguration is setting
quantum to 15140 (10× MTU) — this lets single flows grab 10 packets per round,
destroying fairness. Keep at 1514 (Ethernet MTU) or lower.

**HTB burst values wreck AQM.** When using fq_codel under HTB shaping, the
default HTB burst of 15KB causes latency spikes at sub-100Mbps rates. Reduce
burst to ~3KB (2 packets) to let fq_codel actually control the queue.

**Inbound shaping is a hack, not a fix.** Shaping ingress via IFB/mirred is
defensive — you're dropping packets your upstream already transmitted. It works
but wastes upstream bandwidth. The real fix is AQM at the sender's egress.
Always ask upstream to deploy RFC 7567 AQM at their egress instead.

## WiFi: Where Bufferbloat Is Worst and Hardest

WiFi bufferbloat is fundamentally different from wired. The link rate varies
per-client per-moment (MCS rate adaptation), so static buffer sizing is
impossible. The mac80211 stack in OpenWrt addresses this with three mechanisms
that must ALL be present:

- **Per-station TX queues with fq_codel** — isolates each client's queue
- **Airtime Fairness (ATF)** — allocates by time, not bytes; prevents a slow
  client at MCS0 from consuming 100× the airtime of a fast client at MCS11
- **Airtime Queue Limits (AQL)** — caps per-station hardware queue to ~12ms
  (high limit) / ~5ms (low limit) of airtime. Tunable via debugfs:
  `/sys/kernel/debug/ieee80211/phy0/aql_txq_limit`

Check if your WiFi driver supports these: `iw list | grep -E 'TXQS|AIRTIME_FAIRNESS|AQL'`

Without all three, WiFi latency under load exceeds 1 second — 100× worse than
with them (confirmed in Höiland-Jørgensen's measurements).

## NEVER

- **NEVER assume "more bandwidth" fixes latency.** Gettys' own ISP doubled his
  bandwidth unasked — latency got 10× worse because the bottleneck moved from
  the shaped broadband link to his unmanaged WiFi. More bandwidth just moves
  the bufferbloat to the next unmanaged queue.

- **NEVER deploy fq_codel defaults on DOCSIS without tuning target/interval.**
  The 5ms target is calibrated for Internet-scale RTTs over low-jitter links.
  DOCSIS MAP scheduling adds 5-15ms of inherent delay that isn't bufferbloat —
  CoDel can't distinguish this, so it drops useful packets.

- **NEVER use traditional QoS (DSCP priority without FQ) as a bufferbloat fix.**
  Classification without flow-isolation just moves which traffic gets bloated.
  Gettys: "Traffic classification cannot help you. These are stupid devices."

- **NEVER trust vendor claims of "AQM support" that means RED/WRED.** RED
  requires manual tuning per-link, and Kathie Nichols showed Van Jacobson it
  has two fundamental bugs that make it oscillate. Network operators distrust
  RED for good reason — but the answer is fq_codel/CAKE, not disabling AQM.

- **NEVER test with a single flow and declare "no bufferbloat."** A single
  TCP flow may not fill the buffer. Gettys observed 250ms of hidden bloat on a
  "2 Gbit" peering link that only appeared with 16+ concurrent flows. Test with
  RRUL (realtime response under load) or Crusader, not single-flow speedtests.

- **NEVER set packet limits too low on fq_codel.** People set `limit 200`
  thinking "smaller buffer = less bloat" — but limit counts *packets* regardless
  of size. 200 × 64-byte ACKs ≠ 200 × 64KB GRO-aggregated frames. CAKE fixes
  this by limiting in bytes with overhead accounting.

## Diagnostic Procedure

Before configuring anything:

1. **Establish wired baseline**: Ethernet-connected device → `flent rrul` or
   Waveform bufferbloat test. This separates WiFi issues from WAN issues.
2. **Measure idle RTT** (minimum of 20+ pings) — this is your physical floor.
3. **Measure loaded RTT** (during simultaneous upload + download saturation).
4. **Calculate bloat**: loaded_avg - idle_min. Grade: <5ms=A, <30ms=B,
   <100ms=C, <300ms=D, >300ms=F.
5. **Locate the bottleneck**: If wired is fine but WiFi is bad → WiFi is
   bottleneck (check ATF/AQL). If both are bad → WAN/CPE is bottleneck.
6. **After applying AQM, re-test** with simultaneous up+down. A single-direction
   test hides half the problem (upload bloat is often worse due to asymmetric
   provisioning and smaller upstream buffers).

| Symptom | Likely cause | Fix |
|---|---|---|
| Good bandwidth, terrible loaded latency | Classic bufferbloat | Enable CAKE/fq_codel with SQM |
| SQM enabled but latency still bad | Bandwidth set too high, or bottleneck moved | Reduce SQM bandwidth; check WiFi path |
| fq_codel kills throughput on cable | target/interval too aggressive | Raise target to 10-20ms, interval to 150ms |
| WiFi fine for one device, bad with many | Missing airtime fairness | Verify ATF/AQL support; upgrade to OpenWrt |
| Latency spikes only at certain hours | Shared medium congestion (DOCSIS) | Nothing you can do locally; push ISP |
| VPN/tunnel adds latency under load | Tunnel endpoint has unmanaged buffers | Apply fq_codel inside the tunnel interface |

## Key Insight: Buffers Are Whack-a-Mole

Gettys' deepest lesson: fixing one buffer reveals the next. Any unmanaged
static buffer anywhere in the path is a landmine waiting to detonate when that
link becomes the bottleneck. The goal isn't "fix the buffer" — it's "ensure
every buffer in the path is either managed (AQM) or tiny (BQL)." This requires
thinking end-to-end, not point solutions.

> "Once we fix one problem, it's whack-a-mole on the next, until the moral
> sinks home: Any unmanaged buffer is one waiting to get you if it can ever
> be at a bottleneck link." — Jim Gettys
