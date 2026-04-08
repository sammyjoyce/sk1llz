# Diagnosis Playbook

How to read what the network is telling you. Most "TCP performance" tickets are misdiagnosed because the engineer trusted a single number (throughput, average RTT, loss%) instead of reading the *shape* of the data.

---

## The four numbers that actually matter

For any path, you need:

1. **`min_rtt`** — propagation delay floor. Measure with `ping` while link is idle. This is the only RTT number you can trust.
2. **`rtt_under_load - min_rtt`** — bufferbloat budget. >100 ms severe, 30–100 ms uncomfortable, <5 ms healthy.
3. **Retransmit rate** (`ss -ti` field `retrans:`, or `nstat TcpRetransSegs`). Above ~1% on a wired path means real loss; above ~3% on Wi-Fi is normal noise (not a TCP problem).
4. **Sojourn time** at the bottleneck qdisc (`tc -s qdisc show` for fq_codel: `maxpacket`/`drop_overlimit`). This is the queue's perspective on itself.

If you don't have all four, you are guessing.

---

## Test 1: Where is the bottleneck queue?

The hop where RTT inflates *is* the bottleneck. Find it:

```bash
# In one terminal: saturate upstream
iperf3 -c <server> -t 60 -P 4

# In another: walk the path with TCP probes (ICMP can be deprioritized)
mtr --tcp --port 443 -i 0.5 -c 600 <server>
```

Read the per-hop "Best" vs "Avg" columns. The first hop where Avg jumps far above Best while load is on is the bottleneck queue. That hop is where your AQM must live — *or one hop before it*, shaped below its rate.

If the inflated hop is inside your ISP, you cannot fix it directly. Shape upstream of it on a device you control (your gateway), at 5–15% below the ISP's measured sustained rate. The whole point of doing this is to *move* the queue from "their device with no AQM" to "your device with AQM".

---

## Test 2: Reorder vs loss vs policer (they look identical at first glance)

A retransmit could be:
- **Real loss** — congestion or bit error
- **Reordering** — packet arrived but late, the dupack threshold spuriously triggered fast-retransmit
- **Policer drop** — token-bucket policer (not shaper) caused a synchronized burst loss
- **ECN bleach** — middlebox stripped ECN bits, congestion was signalled but never seen

Distinguish them with a capture:

```bash
tcpdump -i any -s 96 -w /tmp/cap.pcap host <peer>
# Then in wireshark:
tcp.analysis.retransmission         # candidate retransmits
tcp.analysis.out_of_order           # reorder events
tcp.analysis.spurious_retransmission # came back as DSACK
```

**Tells:**

- **Reorder**: DSACK comes back acknowledging the "lost" segment was in fact delivered. If `tcp.analysis.spurious_retransmission` count is significant, your loss detector is too aggressive — enable RACK-TLP (`sysctl net.ipv4.tcp_recovery=1`).
- **Policer**: retransmits cluster at *exactly* the same offset within each token-bucket refill window, often every 100–250 ms. Bandwidth is flat then crashes to zero then recovers. The Internet-wide POLICER16 study (Flach et al., SIGCOMM 2016) shows this is hugely common at carrier ingress.
- **ECN bleach**: send IP_TOS with ECT(0), capture at the receiver, see CE/ECT bits zeroed. Disable ECN on that path.
- **Real loss**: random distribution, RTT also rises, queue full at known device.

---

## Test 3: Wi-Fi specifically — ACK aggregation

On 802.11n/ac/ax, the AP batches ACKs into A-MPDUs. To the sender it looks like ACK-clocking has stopped, then a flood of ACKs arrives at once. This destroys delivery-rate estimators (BBRv1's BDP estimate is wrong by 2–4×).

Symptoms:
- Throughput steady-state is much lower than the radio rate
- `ss -ti` shows `delivery_rate` in bursts, not steady
- BBR's `pacing_rate` chases the wrong number

Fix:
- Use **BBRv3** (tolerates aggregation) or **CUBIC** (doesn't care about delivery_rate).
- Make sure the AP runs **mac80211 airtime fair queueing** (Linux 4.19+, on by default for ath9k/ath10k/mt76). Verify: `iw phy phy0 info | grep -i txq`.
- On the *upstream* side of the AP, use CAKE/fq_codel — Wi-Fi's airtime AQM only handles the radio.

---

## Test 4: Bandwidth-delay product, the right way

```
BDP_bytes = bandwidth_bps * min_rtt_seconds / 8
```

Use **min_rtt**, never average. Average RTT already includes queueing you're trying to remove — using it will inflate your buffer sizing, which causes more bufferbloat, which inflates RTT further. This positive feedback loop is the #1 footgun in TCP tuning guides on the internet.

For asymmetric paths (most home connections), compute **separate** BDPs for upload and download — they have different bottleneck rates.

For Wi-Fi, BDP is meaningless during aggregation events. Take min_rtt during a sustained transfer, not during idle.

---

## Test 5: Is the receive buffer wrong?

Cloudflare 2023 found Linux's TCP autotuner could exceed `tcp_rmem max` and trigger collapse pathology. Symptoms:

```bash
# These should NOT be increasing on a healthy server:
nstat -az | grep -E 'TcpExtTCPRcvCollapsed|TcpExtRcvPruned|TCPZeroWindowDrop'

# Per-socket inspection: skmem_r should be < skmem_rb
ss -tim | grep -A1 ':<port>'
```

If `TcpExtTCPRcvCollapsed` or `TcpExtRcvPruned` are climbing:
1. Lower `tcp_rmem max` to ~2–3× BDP, *not* "as big as possible". Setting it huge makes this worse.
2. If on kernel ≥ 6.5, the upstream window-shrinking patch (commit `b650d953cd39`) is in. Earlier kernels need `tcp_rmem max` capped tighter.
3. Check that the application is actually `recv()`-ing — a slow reader makes the kernel hold receive bytes indefinitely until autotuner blows the limit.

---

## Test 6: Packet-pair bandwidth estimation (for capacity planning)

Send back-to-back packet pairs and measure their inter-arrival time at the receiver. The receiver's dispersion is bottleneck-limited because the bottleneck spaces them out:

```
bandwidth ≈ packet_size / dispersion
```

Take the **median of ≥20 pairs** (not mean — outliers from cross-traffic dominate). This gives you bottleneck capacity *without* needing to saturate the link.

Pitfalls:
- Pacing on the sender (sch_fq, BBR) destroys back-to-back-ness. Use a pre-paced raw socket.
- QoS in the path that processes pairs differently (rare, but ATM-cell links and some DOCSIS are guilty).
- Hardware offload (TSO) can collapse the pair into one segment. Disable.

---

## Test 7: Is your test rig lying to you?

Common test-rig errors that invalidate everything:

- **netem on the same machine as the qdisc you're testing.** netem's default 1000-packet limit becomes the actual bottleneck queue. Use a separate middle box.
- **Switch in the path with uncharacterized buffering.** A "gigabit" switch can have 50 ms of buffer. Either remove it or measure under saturating cross-traffic first.
- **TSO/GSO/GRO leaving "packets" of 64 KB.** AQM math fails because sojourn time is per-superpacket, not per-MTU. `ethtool -K <iface> tso off gso off gro off`.
- **NAPI batching at low rates.** NAPI's default 64-packet weight is too high below 100 Mbit; receive-side latency is dominated by it.
- **NetworkManager replacing your qdisc.** After every up/down, NM may reinstall pfifo_fast. Check after each link event with `tc qdisc show`.

---

## Reading `ss -ti` like a Jacobson

```
cubic wscale:7,7 rto:212 rtt:11.234/2.456 ato:40 mss:1448
cwnd:42 ssthresh:28 bytes_sent:84M bytes_acked:83M
segs_out:58000 segs_in:30000 data_segs_out:58000
send 43.3Mbps lastsnd:8 lastrcv:8 lastack:8
pacing_rate 51.9Mbps delivery_rate 41.7Mbps
busy:18000ms rwnd_limited:0 sndbuf_limited:120ms
unacked:14 retrans:0/87 reordering:3 rcv_rtt:11.45 rcv_space:131072
```

What to look at:

| Field | Meaning | Red flag |
|---|---|---|
| `rtt:A/B` | SRTT / RTTVAR in ms | RTTVAR > SRTT/4 = jittery path |
| `cwnd` vs `ssthresh` | growth phase | cwnd capped at ssthresh forever = stuck in CA after a loss |
| `retrans:X/Y` | current/total retransmits | Y/segs_out > 1% = real loss problem |
| `rwnd_limited` | time blocked by receiver window | non-zero = receiver buffer too small |
| `sndbuf_limited` | time blocked by sender buffer | non-zero on wmem-tuned host = wmem max too low |
| `pacing_rate` vs `delivery_rate` | requested vs actual | gap = path bottleneck below sender's estimate |
| `delivery_rate` vs link speed | how full the pipe is | far below link = something's bottlenecking |

A connection with `cwnd:10 ssthresh:7` and `retrans:0/200` has hit a loss event and is now in congestion avoidance — totally healthy. A connection with `rwnd_limited:30000ms` is being throttled by the receiver, no amount of sender tuning helps.

---

## When in doubt: capture, then think

`tcpdump -i <iface> -s 128 -w /tmp/<peer>.pcap host <peer>` for 30 seconds during the symptom, then analyze offline. *Reading bytes is faster than guessing.* Every senior network engineer has, at some point, spent two days debugging a "TCP problem" that a 30-second capture would have answered in five minutes.
