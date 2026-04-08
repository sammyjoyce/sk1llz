---
name: jacobson-network-performance
description: Diagnose and fix TCP/IP throughput, latency, bufferbloat, and congestion-control problems the way Van Jacobson would. Use when investigating a fast link that feels slow, choosing or tuning a congestion control (CUBIC/BBR/BBRv3/Hybla/DCTCP), configuring fq_codel/CoDel/CAKE, sizing TCP socket buffers (rmem/wmem/tcp_notsent_lowat), interpreting RTT-under-load and packet captures, or debugging latency spikes on Wi-Fi, satellite, LFNs, datacenters, and CDN edge. Trigger keywords - bufferbloat, congestion control, BBR, CUBIC, fq_codel, CoDel, CAKE, AQM, RTT, BDP, slow start, retransmit, ssthresh, tc qdisc, sysctl tcp_, RACK, PRR, Karn, Wi-Fi latency, satellite TCP, head-of-line blocking, HTTP/2 prioritization.
tags: networking, tcp, congestion-control, bufferbloat, aqm, latency, performance, bbr, fq_codel
---

# Jacobson Network Performance

## The mental model that beats every checklist

Jacobson-style work rests on four sentences that take years to internalize:

1. **The bottleneck has a queue, and the queue's depth is the latency you feel.** Throughput is what's left over.
2. **RTT under load minus min RTT is the only honest bufferbloat number.** Speed-test "scores" lie because the queue may live in the modem, not in you.
3. **You only control AQM at the bottleneck.** If a slower device is downstream of your shaper, you've shaped nothing — the queue moved one hop.
4. **Reordering is not loss, application-limit is not congestion, and a retransmit's RTT is undefined.** Confusing any of these will silently halve your throughput.

Before touching a knob, ask yourself: *which of those four am I currently violating, and where is the actual bottleneck queue?* If you can't name the queue, stop and capture before tuning.

## The decision that matters most: which congestion control

Choosing wrong here outweighs every sysctl. Use this table — it encodes findings practitioners learn the hard way.

| Path characteristic | Pick | Why (and what NOT to use) |
|---|---|---|
| Datacenter, sub-ms RTT, ECN-capable fabric | DCTCP (or BBRv3 with ECN) | CUBIC's MD halving is catastrophic at µs RTTs; needs ECN marking, not drops. |
| LAN / wired internet, deep buffers, low loss | CUBIC | BBR over-runs deep buffers and starves CUBIC neighbors. |
| Shallow-buffer + large BDP (CDN edge, transit) | BBRv2/v3 | CUBIC misreads shallow drops as congestion and collapses. BBRv1 here causes **100× the retransmits** of CUBIC — never ship BBRv1. |
| Lossy Wi-Fi / cellular last mile | BBRv3 + RACK-TLP | CUBIC treats wireless retransmits as congestion and never recovers. |
| GEO satellite, RTT ≈ 600 ms | Hybla, or BBR with `initial_cwnd` raised | CUBIC's slow-start takes minutes to reach line rate at 600 ms RTT; Hybla scales by ρ²/cwnd. |
| Mixed traffic with CUBIC neighbors you must be fair to | CUBIC, *not* BBR | BBRv3 still has documented fairness/convergence problems with CUBIC, especially with ECN enabled where BBR can starve CUBIC. |

**Never enable BBR globally as a "performance upgrade"** — it shifts retransmit cost and bandwidth share onto your neighbors. Test on representative paths first.

## Critical numbers practitioners memorize

These are the values you'll otherwise rediscover by causing an outage:

- **Jacobson RTO** (RFC 6298): `RTO = SRTT + max(G, 4·RTTVAR)`, with `α=1/8`, `β=1/4`, clock granularity G. The constant 4 is empirical; lowering it increases spurious RTOs sharply.
- **Linux IW** is 10 segments (RFC 6928). Raising it past 10 risks bursty loss inside the first RTT — only do it for known-clean paths.
- **CoDel target/interval = 5 ms / 100 ms** is valid only above ~4 Mbit. Below that, target must exceed one MTU's serialization time (≈13 ms at 1 Mbit) or CoDel drops everything.
- **fq_codel default packet limit (10 000) is insane below 1 Gbit**. Use 1000–1200 at gigE, 600 at 10 Mbit. Quantum 300 below 100 Mbit, MTU+14 at higher rates.
- **Shaper rate must be 5–15% below the ISP's actual rate**, otherwise the bottleneck queue lives in the modem and AQM has nothing to manage. Yes, you give up bandwidth to win latency. That trade is the whole point.
- **`tcp_notsent_lowat = 16384`** is the missing piece for HTTP/2, gRPC, and any multiplexed protocol. The default lets megabytes commit to the kernel send buffer, defeating per-stream prioritization.
- **`net.ipv4.tcp_slow_start_after_idle = 0`** — the default of 1 silently restarts slow-start on long-lived idle keep-alive connections, killing the first response after every pause.
- **Receive buffer max ≠ "set it high and forget"**. Linux <6.5 had a bug where the autotuner could exceed `tcp_rmem max` and trigger `tcp_collapse`/OFO pruning, dropping throughput sitewide. Cap maximum at 2–3× BDP, never "as big as possible".
- **BDP** = `bandwidth × min_RTT`. Use **min** RTT, not average — averages already include queueing you're trying to remove.

Before you load `references/tuning-recipes.md`, the rule of thumb is: change one knob, measure RTT-under-load, repeat. Never change three at once.

## The diagnosis loop (run this before any fix)

1. `ping` while idle → record `min_rtt` (the only ground truth).
2. Saturate the link in one direction with `iperf3` or a long download.
3. `ping` again *during load*. The increase is your bufferbloat budget. >100 ms = severe; >30 ms = uncomfortable for VoIP/games; <5 ms = healthy.
4. If RTT spikes, the queue is somewhere. Walk hops with `mtr --tcp` or `tcpdump` from each side until the inflated hop appears. **The hop where RTT inflates is the bottleneck queue — that is where AQM must live.**
5. If you control that hop, apply AQM. If you don't, shape *upstream of it* at 5–15% below its true rate.

For deeper interpretation (sojourn time, ACK aggregation on Wi-Fi, packet-pair BW estimation, distinguishing reorder from loss), **READ `references/diagnosis-playbook.md` before you trust your numbers**.

## Anti-patterns: each one has burned a senior engineer

**NEVER set `tcp_rmem`/`tcp_wmem` max to "as big as possible".** It looks free but the autotuner can exceed it under bursty receivers, triggering `tcp_collapse` and OFO pruning that drops *new arriving packets across the whole socket* (Cloudflare 2023). Instead cap at `2 × BDP` and monitor `nstat TcpExtTCPRcvCollapsed`/`TcpExtRcvPruned`.

**NEVER measure RTT from a retransmitted segment (Karn's rule).** It is seductive because you have the data, but you cannot tell whether the ACK is for the original or the retransmit — feeding ambiguous samples into Jacobson's smoother corrupts SRTT permanently and causes retransmit storms. Instead skip the sample and double the RTO on every consecutive timeout until a clean ACK arrives.

**NEVER trust 3-dupack as your loss signal on a fast or reordering path.** The "3" was chosen for 1988 networks. On modern paths reorder >3 is routine, so dupack-based recovery causes spurious half-windows and tanks throughput. Instead enable RACK-TLP (RFC 8985, Linux 4.18+, FreeBSD `tcp_rack`) which uses per-segment timestamps and survives reorder, application-limited senders, and lost retransmits.

**NEVER run `netem` and another qdisc on the same machine.** netem's 1000-packet default queue silently becomes the bottleneck and your "200 ms emulated link" is actually a tail-drop FIFO. Instead run netem on a dedicated middle box.

**NEVER test bufferbloat through a switch you didn't characterize.** Some gigE switches have *50 ms* of internal buffering; your fq_codel results reflect the switch, not your qdisc. Instead either remove the switch or measure its buffering under saturating cross-traffic first.

**NEVER leave TSO/GSO/GRO on at sub-100 Mbit AQM points.** "Superpackets" of up to 64 KB defeat CoDel's head-drop and inflate per-packet sojourn time invisibly. Instead disable on the bottleneck NIC (`ethtool -K eth0 tso off gso off gro off`) or use CAKE which performs offload-peeling.

**NEVER raise the initial congestion window past 10 because "more is faster".** It is faster *only* when there is no loss in the first RTT; on shallow-buffer paths it causes a burst loss before any feedback exists, pushing the connection straight into RTO. Instead leave IW10 unless you have measured your path's first-RTT loss profile.

**NEVER disable Nagle (`TCP_NODELAY`) on bulk transfers "to be safe".** It interacts with delayed-ACK to cause the famous 200 ms write-write-read stall pattern only on small interactive writes; on bulk transfers Nagle does nothing harmful and disabling it just wastes packets. Instead disable Nagle only on request/response sockets, and prefer `TCP_CORK`/`MSG_MORE` for "I have more coming".

**NEVER fix bufferbloat by "increasing buffer size".** It is the literal opposite of the cure. Larger buffers absorb more packets before signaling congestion, raising the standing queue and the latency. Instead reduce buffers, install AQM, and shape below the bottleneck rate.

## Freedom calibration

The diagnosis loop is high-freedom: think, measure, hypothesize, repeat — there is no script that works for every path. The numeric knobs (sysctls, qdisc parameters, RTO constants) are low-freedom: use the values above or in `references/tuning-recipes.md` exactly, change one at a time, and measure RTT under load after each change. Mixing the two — being scripted about diagnosis or creative about RTO constants — is how outages happen.

## Loading triggers

- For exact `sysctl`, `tc`, and `ethtool` recipes per environment (Wi-Fi router, datacenter ToR, CDN edge, GEO satellite, home gateway): **READ `references/tuning-recipes.md`**.
- For packet-capture interpretation, sojourn-time math, ACK-aggregation on Wi-Fi, packet-pair bandwidth estimation, and deciding "reorder vs loss vs policer": **READ `references/diagnosis-playbook.md`**.
- **Do NOT load** either reference for a pure conceptual question ("explain BBR vs CUBIC") — the table above is sufficient.
- **Do NOT load** `tuning-recipes.md` until you have completed the diagnosis loop and identified the bottleneck queue. Tuning before diagnosis is the #1 cause of regressions.
