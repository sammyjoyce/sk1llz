---
name: jacobson-network-performance
description: "Diagnose queueing, RTT, and TCP control-loop pathologies with Jacobson-style measurement discipline. Use when a path is fast but feels slow, RTT explodes under load, fq_codel/CoDel/CAKE or BBR/CUBIC tuning is on the table, or you must separate real loss from reordering, policers, ACK pathologies, or hidden buffers. Trigger keywords: bufferbloat, standing queue, min_rtt, BDP, fq_codel, CoDel, CAKE, BQL, RACK, DSACK, ACK compression, tcp_notsent_lowat, delayed ACK, BBRv3, CUBIC, ECN bleach, tc qdisc, tcp_rmem, tcp_wmem, Wi-Fi aggregation, satellite TCP."
tags: networking, tcp, congestion-control, bufferbloat, aqm, latency, performance, bbr, fq_codel
---

# Jacobson Network Performance

This is a control-loop skill, not a "tune every TCP knob" skill. If you cannot name the queue absorbing the extra packets, you are not tuning the network; you are moving blame between layers.

## Before you tune, ask yourself

- **Where is the queue I can actually control?** qdisc backlog is only the visible queue. The latency tax often lives in a modem, a NIC TX ring, Wi-Fi firmware, or the reverse ACK path. If the queue is invisible, fix queue visibility first with shaping, BQL, or airtime/AQL.
- **Am I looking at standing queue or true congestion?** CoDel's core insight is that a standing queue adds delay without adding throughput. If the path is just holding a permanent queue, "more bandwidth policy" and "bigger buffers" do nothing except preserve the delay.
- **Is the signal really loss?** Modern paths routinely produce reorder, ACK aggregation, delayed ACK artifacts, and policer drops that masquerade as congestion. Treat "3 dupacks" as a legacy hint, not a truth source.
- **Am I reasoning in time or bytes?** Static byte buffers are nonsense on variable-rate links. Even static time targets are only a compromise on Wi-Fi or cellular where link rate can move by 10-100x in seconds.

## Fast Path Decision Tree

| Symptom | Likely pathology | What to do next |
|---|---|---|
| RTT under load explodes but qdisc counters stay quiet | Dark buffer outside the qdisc | Check modem/NIC/Wi-Fi firmware first. On Linux, BQL exists to move bytes out of opaque TX rings into the qdisc where AQM can act. |
| Retransmits later show up as DSACK or Wireshark "spurious retransmission" | Reordering, not congestion | Switch to RACK-TLP before touching cwnd or buffers. RACK starts with `min_rtt/4` reorder tolerance and grows from DSACK evidence up to `SRTT`. |
| Losses arrive at a fixed cadence and throughput flat-lines, then cliffs | Token-bucket policer | Stop "tuning TCP." Pace or shape to the policer; otherwise every algorithm will keep rediscovering the same drop edge. |
| HTTP/2 or gRPC priorities fail even though the link is not full | Send-buffer commitment | Use `tcp_notsent_lowat`; bytes already committed to the send buffer are outside the app's control. |
| CoDel/fq_codel on a slow link makes throughput collapse | Target smaller than one MTU serialization time | Raise `target` so it is at least one MTU time on the egress rate, then keep it in the 5-10% of `interval` range. |
| BBR looks unstable on Wi-Fi | ACK aggregation is corrupting delivery-rate estimation | Prefer BBRv3 or CUBIC and inspect whether `delivery_rate` arrives in bursts rather than as a smooth clock. |

## Numbers That Matter More Than Most Dashboards

- **Jacobson/Karels RTO math is conservative on purpose.** RFC 6298 keeps `RTO = SRTT + max(G, 4*RTTVAR)` with `alpha = 1/8`, `beta = 1/4`, and a 1-second initial RTO because lowering the variance multiplier sharply increases spurious timeouts.
- **A retransmitted packet has no usable RTT sample.** Karn's rule is not trivia; violating it poisons the estimator exactly when the path is least stable.
- **CoDel defaults are about time, not magic.** `target = 5 ms` and `interval = 100 ms` are open-Internet defaults, but RFC 8290 says `target` must be at least one MTU serialization time and otherwise sit around 5-10% of `interval`. At 1 Mbps and MTU 1500, that is roughly 15 ms, not 5 ms.
- **FQ-CoDel defaults assume fast links.** RFC 8290 says Linux's `limit 10240` is suitable up to 10 GbE and `quantum 1514` is the Ethernet MTU plus L2 header. Treat those as upper-speed defaults, not sacred constants for a 20 Mbps edge.
- **Offloads distort the scheduler's unit of work.** RFC 8290 notes TSO "packets" can reach 64 KB and GRO can reach about 25 KB. If your AQM result only appears with offloads on, you may be benchmarking superpackets, not queue control.
- **RACK's reorder window is adaptive, not fixed.** RFC 8985 starts at `min_rtt/4`, grows linearly for each round trip that produces DSACK, caps at `SRTT`, and resets after 16 recoveries. That reset matters: a path can oscillate between transient and persistent reorder.
- **BBRv1's shallow-buffer problem is concrete.** Google's IETF data for a 100 Mbps, 100 ms path with a buffer of 5% BDP showed roughly `14-15%` retransmit for BBRv1, `~0.06%` for CUBIC, and `~1.3%` for BBRv2. If the buffer is shallow, never treat BBRv1 as a harmless default.
- **BQL is worth a small CPU tax when the NIC ring is the dark buffer.** Linux BQL work reported about `1-3%` CPU/pps overhead, while cutting queued bytes from megabytes to hundreds of kilobytes and massively improving high-priority latency. If the hardware queue is opaque, AQM is blind.
- **ACK filtering is not free bandwidth.** RFC 3449 warns that ACK filtering/decimation creates stretch ACKs, increases sender burst size, and can break DupACK, SACK, and ECN semantics unless paired with burst mitigation or ACK reconstruction.
- **`tcp_notsent_lowat = 16384` is a prioritization control, not a generic throughput tweak.** Cloudflare's HTTP/2 work used 16 KB because data written deep into the send buffer cannot be preempted by higher-priority streams once committed.

## Procedures That Prevent False Fixes

1. **Locate queue visibility before choosing a queue algorithm.**
   - If qdisc backlog tracks the RTT blow-up, the queue is visible and qdisc tuning is reasonable.
   - If RTT blows up while qdisc counters stay quiet, assume a dark buffer in the modem, NIC ring, firmware, or reverse path.
   - Before changing `tc`, **READ `references/tuning-recipes.md`**.

2. **Classify the signal before calling it congestion.**
   - DSACK or spurious retransmission markers mean reorder.
   - Periodic drop cliffs mean policer.
   - Rising RTT with no delivery gain means standing queue.
   - Bursty `delivery_rate` on Wi-Fi usually means ACK aggregation, not sudden capacity changes.
   - Before trusting a pcap or `ss -ti`, **READ `references/diagnosis-playbook.md`**.

3. **Only then choose the control.**
   - Hidden queue: move the bottleneck with shaping or expose it with BQL/AQL.
   - Reordering: enable RACK-TLP rather than lowering throughput on false loss.
   - Multiplexed application starvation: cap unsent bytes with `tcp_notsent_lowat`.
   - Datacenter ECN fabric: use DCTCP or BBRv3-style ECN logic; do not copy WAN defaults into a microsecond fabric.

## Anti-Patterns Practitioners Learn the Hard Way

**NEVER tune only the qdisc because qdisc graphs are the easiest thing to see.** That is seductive when `tc -s` is already in your hand, but the real queue may be in a modem, NIC ring, or Wi-Fi firmware, so you "fix" the wrong layer and keep paying the same latency. Instead prove the queue is visible or move it upstream with shaping, BQL, or airtime control.

**NEVER keep CoDel at `target 5ms` on a slow link because the default looks authoritative.** RFC 8290 explicitly says target must be at least one MTU serialization time; below that, CoDel becomes an overdropping machine and your "latency fix" becomes throughput collapse. Instead size `target` from egress serialization time and keep it in the 5-10% of `interval` range.

**NEVER treat 3 dupacks as proof of loss because modern reorder and ACK aggregation regularly exceed that threshold.** It is seductive because old tooling still centers DupThresh, but on today's paths it creates spurious fast recovery and self-reinforcing retransmits. Instead use RACK-TLP and DSACK evidence.

**NEVER deploy ACK filtering or decimation as a generic Internet optimization because it seems to save reverse-path bandwidth.** RFC 3449 shows why this is dangerous: stretch ACKs enlarge sender bursts and can suppress DupACK, SACK, or ECN semantics. Instead use it only on controlled asymmetric reverse links with explicit burst mitigation or ACK reconstruction.

**NEVER increase `tcp_rmem`, `tcp_wmem`, or device buffers "for safety" because larger buffers are often just larger standing queues.** That feels good in a single throughput test, but Cloudflare's receive-buffer work showed collapse and pruning pathologies once autotuning and slow readers interact badly. Instead cap around `2-3x` BDP and watch the collapse/prune counters.

**NEVER disable ECN after the first CE storm because marking is often the only honest evidence that the queue is real.** Turning it off is seductive when `noecn` appears to quiet the graph, but it pushes you back to drop-based signaling and can hide ECN bleaching or AQM misplacement. Instead verify that ECT/CE survives the path and only disable after proving a middlebox is broken.

**NEVER assume a static buffer size works on Wi-Fi or cellular because rate can move by orders of magnitude inside one flow.** That is seductive because the math is clean at one chosen rate, but Jacobson's own bufferbloat discussion makes the failure mode explicit: a buffer sized for 100 Mbps becomes a latency disaster at 1 Mbps. Instead control time in queue, not bytes in queue.

## Freedom Calibration

- Diagnosis is high-freedom: capture, compare time-series shapes, and change your hypothesis when the signal points somewhere else.
- Numeric tuning is low-freedom: do not improvise on RTO constants, CoDel targets, or BBR generation choice without path-specific evidence.
- If you are tempted to change more than one control at once, you have not isolated the pathology yet.

## Loading Triggers

- Before interpreting packet captures, `ss -ti`, DSACK, reorder, policers, or ACK aggregation, **READ `references/diagnosis-playbook.md`**.
- Before changing `sysctl`, `tc`, or `ethtool` settings on a real host, **READ `references/tuning-recipes.md`**.
- **Do NOT load** `tuning-recipes.md` for a conceptual question like "why is BBRv1 bad in shallow buffers?"
- **Do NOT load** either reference until you have answered the first question in this skill: "Where is the queue I can actually control?"

If you remember one rule, make it this: Jacobson-style performance work starts by locating the queue in time, not by twiddling throughput knobs in bytes.
