---
name: gettys-bufferbloat
description: >-
  Diagnose and fix latency under load by locating the real queue, selecting the
  right AQM/FQ strategy, and avoiding dark-buffer traps across DOCSIS, DSL,
  Wi-Fi, tunnels, containers, and Linux qdiscs. Use when loaded RTT, jitter,
  or game/video-call lag appears only during traffic; when tuning fq_codel,
  CAKE, SQM, BQL, AQL, or tc qdiscs; or when fq_codel/CAKE "should help" but
  does not. Triggers: bufferbloat, latency under load, fq_codel, CAKE, SQM,
  AQM, BQL, AQL, DOCSIS, Wi-Fi lag, tc qdisc, dark buffer, jitter, loaded RTT.
---

# Gettys Bufferbloat

## Load Policy

This skill is self-contained for diagnosis and tuning.

- **MANDATORY**: Before touching Wi-Fi settings, get a wired under-load baseline first.
- **MANDATORY**: Before touching DOCSIS or DSL overhead/RTT knobs, read current `tc -s qdisc` output and confirm the actual access technology.
- Do NOT load generic QoS tutorials or TCP primers for this task; they dilute the real question, which is queue ownership.
- Do NOT load Wi-Fi-specific material if wired RRUL is still bad; the queue is probably not in Wi-Fi yet.

## Operator Frame

The job is not "enable smart queueing." The job is to make the first queue that fills be a queue you control.

Before changing anything, ask yourself:

- **Where is the bottleneck right now?** If the bottleneck moved, yesterday's AQM fix is now irrelevant.
- **Does the qdisc see the backlog?** If app latency is huge while qdisc delay stays low, the real queue is below or beside the qdisc.
- **Am I measuring under enough load to saturate the link?** Single-flow tests often miss cable, Wi-Fi, and ACK-path pathologies.
- **What fairness am I trying to enforce?** Per-flow, per-host, per-subscriber, or per-station lead to different modes.

## Fast Triage

| What you see | Most likely truth | First move | If it still fails |
|---|---|---|---|
| Wired and Wi-Fi both bloat | WAN/CPE, tunnel, or CPU is the queue owner | Run bidirectional RRUL, inspect `tc -s qdisc` on WAN | Check driver rings, modem/CMTS behavior, CPU saturation |
| Wired is clean, Wi-Fi is awful | Wi-Fi firmware/mac80211 queues, not WAN | Inspect airtime fairness and AQL support | Reduce aggregation only as a last resort; upgrade AP/driver |
| `tc` shows tiny backlog but user latency spikes | Dark buffer below qdisc | Check BQL/rings/offloads/tunnel path | Move shaping closer to hardware or use integrated shaper |
| fq_codel/CAKE helps upload but not download | Downlink queue is upstream of you | Lower ingress shaping rate and confirm framing compensation | Push AQM to sender/upstream; local ingress shaping is only damage control |
| VPN path is unfair even with fq_codel | Inner flows collapsed into one outer flow | Shape on the tunnel interface/endpoints | Change fairness domain; outer 5-tuple fairness is not enough |
| Containers/veth bloat despite fq_codel | veth ring is hiding backlog | Suspect missing BQL/driver queue ownership | Use shaper/AQM where bytes become visible, or upgrade kernel/driver |

## Measure Like an Operator

- Use RRUL-class testing, not idle ping plus one speed test. RRUL's 4 up + 4 down flows exist because one flow often fails to saturate real bottlenecks and can hide ACK-path artifacts.
- Always compare `idle_min_rtt` to `loaded_rtt` in both directions. Upload bloat is often the first failure on asymmetric links.
- Minimum evidence set:
  - `tc -s qdisc show dev $WAN`
  - `ethtool -g $DEV` and `ethtool -k $DEV | egrep 'gro|gso|tso'`
  - Wi-Fi only: `iw list` plus mac80211 airtime/AQL state if exposed
- Read the qdisc stats, not just the grade:
  - Low `pk_delay`/`av_delay`, high app latency: wrong queue.
  - Rising `marks` with few `drops`: ECN is working.
  - High `overlimits` but no latency improvement: shaping above the true bottleneck or CPU bound.
  - In CAKE, `ack_drop` increasing can be good only when egress asymmetry is the problem.

## If the Graph Does Not Move

- High qdisc delay and high loaded RTT: you found the right queue, but the rate or overhead is wrong. Lower shaped rate a few percent and fix link-layer compensation before touching CoDel timers.
- Low qdisc delay and high loaded RTT: wrong queue. Go looking below the qdisc, not for new fq_codel parameters.
- Lower latency but collapsed throughput on slow or cable links: your target/RTT assumptions are too aggressive for serialization or MAC scheduling.
- Good latency but unfair multi-user behavior: switch fairness domain before touching AQM. This is usually a host-isolation problem, not a drop-policy problem.

## Dark Queues People Miss

- **Driver and hardware rings**: fq_codel only controls what it can see. RFC 8290 calls out lower-layer queues explicitly; BQL exists to push bytes back up into qdisc-visible space.
- **veth/virtual rings**: recent veth BQL work showed a 256-entry hidden ring adding about 22-24 ms RTT under load; moving buffering into the qdisc cut that to about 1.3-1.5 ms without throughput loss.
- **GSO/GRO/TSO super-packets**: fq_codel's quantum is 1514 bytes, but offloads can create 25-64 KB "packets" that monopolize dequeue time and inflate latency for competing sparse flows.
- **Wi-Fi airtime queues**: the queue is measured in airtime, not packets. A WAN shaper can look perfect on Ethernet while Wi-Fi still delivers second-scale delay.
- **CMTS/DSLAM scheduling**: DOCSIS request-grant delay and ISP metering can dominate. Your home qdisc may be innocent.

## Tuning Rules That Matter

### fq_codel

- Default `target 5ms` is wrong when one MTU takes longer than that to serialize. RFC 8290 says target should be at least one MTU serialization time and otherwise about 5-10% of `interval`; at 1 Mbit/s, one 1500-byte packet already costs roughly 12-15 ms.
- `limit` is a hard memory safety rail, not the control knob. Linux defaults to 10240 packets and expects CoDel to act long before the limit is hit.
- If lower layers are unmanaged, fq_codel alone is not enough. RFC 8290 explicitly recommends BQL or a software shaper such as HTB/HFSC when the queue is not at the qdisc.

### CAKE

- Use CAKE when you need shaping and queue control in the same place. That removes the HTB burst problem and keeps the owning queue simpler.
- On DOCSIS, the non-obvious question is "what size does the head-end meter?" not "what hits the wire?" CAKE's `docsis` keyword encodes `overhead 18 mpu 64 noatm` because CMTS metering semantics matter more than coax framing minutiae.
- On PTM/VDSL2, `ptm` is effectively a 64/65 derate. If you hand-roll a parent shaper, that 0.984 factor is the floor, not a guess.
- `lan` and `datacentre` RTT presets are easy to misuse. CAKE's own man page warns that `lan`-class time constants are near kernel jitter, so congestion gets signaled prematurely and flows go sparse; for most local shaping, `metro` is the safer floor unless you control the kernel and path precisely.
- In ingress mode CAKE deliberately keeps at least two packets queued per flow because retransmits are more expensive after the bottleneck has already been crossed. Do not chase zero queue on ingress.
- `ack-filter` is an egress-only tool for heavily asymmetric links. Conservative mode is the default reasoned choice; aggressive mode has bitten SACK/reordering edge cases and can damage ramp-up behavior.
- Keep `split-gso` unless the link is above roughly 10 Gbit/s and throughput, not latency, is the limiting problem. Below that, leaving super-packets intact is usually just queue inflation with a nicer CPU profile.
- `nat` only improves fairness if NAT happens on the same host running CAKE. If translation occurs elsewhere, it buys nothing.
- `wash` is a defensive choice on inbound traffic when you cannot trust DSCP markings. CAKE's own guidance calls out providers like Comcast; use `besteffort + wash` if inbound markings are polluted.
- `memlimit` is a memory ceiling, not a bandwidth-delay product calculator. CAKE documentation explicitly says not to size it from BDP.

### DOCSIS and Cable

- CableLabs had to retune CoDel for DOCSIS because request-grant latency makes empty queues look "late." Their early DOCSIS experiments raised CoDel `target` to 10 ms just to avoid dropping packets that were only waiting on the MAC scheduler.
- CableLabs also explored SFQ-CoDel at much larger `target/interval` values such as 50/200 ms in modem simulations to preserve TCP throughput despite DOCSIS scheduling. That is a modem-side trade-off, not a home-router preset to cargo-cult into Linux fq_codel.

### Wi-Fi

- Treat Wi-Fi as airtime contention, not bandwidth contention. One slow station can consume everyone else's airtime while byte counters still look fair.
- The mac80211 fix is a stack, not one knob: per-station fq, airtime fairness, and AQL. If any one is missing, WAN SQM will hide little.
- The default AQL thinking is "keep only a few milliseconds of airtime in hardware." If the driver does not expose AQL/airtime features, stop pretending WAN CAKE is the whole answer.

### Tunnels and Virtualization

- FQ fairness breaks when many inner flows collapse into one opaque outer flow. RFC 8290 calls this out explicitly for encrypted VPNs: you still get shorter queues, but not inner-flow prioritization.
- If the tunnel endpoint owns the bottleneck, shape the tunnel device or the endpoint egress. Shaping only the outer underlay after encapsulation is often the wrong fairness domain.

## NEVER

- NEVER shrink `txqueuelen` or NIC rings blindly because "smaller queue must mean less bloat." That is seductive because it sometimes helps instantly, but Bufferbloat's own Linux notes warn it can merely move loss into the driver or make some systems catatonic. Instead use BQL or a shaper/AQM that owns the bottleneck.
- NEVER declare victory from idle ping or a single TCP flow because those tests often fail to saturate the true bottleneck and miss ACK-path effects. Instead use bidirectional RRUL-class load and read qdisc stats while the link is actually full.
- NEVER copy fq_codel defaults onto very slow links because CoDel will punish serialization delay as if it were queue abuse. Instead ensure `target` is at least one MTU serialization time and keep it within about 5-10% of `interval`.
- NEVER paste DOCSIS simulation values such as 50/200 into a home fq_codel config because those values were chosen to tolerate modem MAC scheduling and 32-queue silicon trade-offs, not to optimize your router. Instead start from the actual bottleneck technology and measured RTT floor.
- NEVER use `ack-filter` on ingress because the ACKs already crossed the bottleneck before you saw them. Instead use conservative `ack-filter` on egress only when down/up asymmetry is the real limiter.
- NEVER disable offloads first just because you spotted giant packets. That is seductive because it makes graphs look cleaner, but it can turn CPU into the new bottleneck. Instead keep CAKE `split-gso`, and disable offloads only when you have proved the qdisc cannot otherwise see or split the bursts.
- NEVER use flow fairness when the real policy target is hosts or subscribers. That is seductive because `flows` is the classic fq_codel model, but it rewards the user who opens the most connections. Instead use CAKE host isolation modes such as `dual-srchost`, `dual-dsthost`, or `triple-isolate`.
- NEVER keep retuning CAKE/fq_codel when qdisc delay is low and user latency is high. That means the queue you are tuning is not the queue that hurts. Instead go hunting for dark buffers: rings, firmware, tunnels, modem scheduling, or CPU starvation.

## Fallbacks

- If CAKE fixes latency but costs too much CPU, verify CPU saturation before blaming the algorithm. The new bottleneck may be the shaper core, not the link.
- If Ethernet is clean and Wi-Fi is not, stop WAN tuning and switch to airtime/AQL work.
- If ingress shaping still wastes too much bandwidth, remember that local ingress control is inherently sacrificial; the real fix is AQM at the sender's egress or provider edge.
