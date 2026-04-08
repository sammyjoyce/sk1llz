# Tuning Recipes⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​‌‌‌‌​‌‍‌​​​‌​​​‍​‌‌​​‌‌​‍‌‌‌‌‌​​​‍​​​​‌​‌​‍‌​‌‌‌​‌‌⁠‍⁠

Exact-value recipes for the environments practitioners actually deploy. **Apply one at a time and measure RTT-under-load after each change.** Values are starting points, not gospel — but they encode hard-won defaults, not arbitrary suggestions.

---

## Recipe 1: Home / SOHO gateway behind a cable modem (asymmetric, deep-buffered modem)

**The problem:** The cable modem has megabytes of bloat you cannot touch. Your AQM is useless unless you move the bottleneck queue *into* your gateway.

```bash
# Discover ISP's actual sustained rate (NOT advertised rate). Run a 60s
# saturating download/upload via iperf3 to a known-fast endpoint.
# Set the shaper at 85-95% of measured (start at 85% and walk up).

ISP_DOWN_KBIT=85000   # measured ~100 Mbit, shape at 85
ISP_UP_KBIT=8500      # measured ~10 Mbit, shape at 8.5

# Use CAKE if available (Linux >=4.19) — CAKE auto-tunes quantum, peels
# GRO superpackets, and handles diffserv. Always prefer CAKE over
# fq_codel on a home gateway.
tc qdisc replace dev wan root cake bandwidth ${ISP_UP_KBIT}kbit \
   docsis ack-filter
tc qdisc replace dev lan root cake bandwidth ${ISP_DOWN_KBIT}kbit \
   docsis ingress
```

If you must use fq_codel instead of CAKE:

```bash
tc qdisc replace dev wan root handle 1: htb default 11
tc class add dev wan parent 1: classid 1:11 htb rate ${ISP_UP_KBIT}kbit
tc qdisc add dev wan parent 1:11 fq_codel \
   limit 1000 quantum 300 target 5ms interval 100ms
```

**Validate** with the Waveform bufferbloat test or `flent rrul`. A grade < A means you are still queueing somewhere — usually because shaper rate is too high (queue still in modem) or you forgot ingress shaping.

---

## Recipe 2: Slow link (< 4 Mbit DSL, 3G fallback)

CoDel's 5 ms target is invalid here — one MTU's serialization time exceeds 5 ms. CoDel will drop everything.

```bash
# At 1 Mbit, one 1500-byte packet takes ~12 ms to serialize.
# Target must exceed that, otherwise sojourn is always > target.
tc qdisc replace dev ppp0 root cake bandwidth 800kbit \
   rtt 100ms                # widen interval, raises target proportionally
# Or with fq_codel:
tc qdisc replace dev ppp0 root fq_codel \
   limit 600 quantum 300 target 15ms interval 150ms
```

Also disable hardware offloads — TSO superpackets at this speed are absurd:

```bash
ethtool -K ppp0 tso off gso off gro off
```

---

## Recipe 3: Wi-Fi access point

Wi-Fi has *aggregation*: the radio batches frames into A-MPDUs, so ACKs arrive in clumps and ACK-clocking is broken. BBRv1 underestimates BDP catastrophically here. Two things matter:

```bash
# 1. Use the airtime-fair Wi-Fi scheduler (mac80211 fq_codel-on-airtime,
#    Linux 4.19+). It's the kernel default on modern ath9k/ath10k/mt76.
#    Verify it's active:
iw phy phy0 info | grep -i 'txq\|airtime'

# 2. On the AP itself, use BBRv3 + RACK-TLP for upstream TCP, and ensure
#    the *upstream* gateway is doing CAKE/fq_codel. The Wi-Fi scheduler
#    only fixes the downstream half.
sysctl -w net.ipv4.tcp_congestion_control=bbr
sysctl -w net.ipv4.tcp_recovery=1   # RACK-TLP enabled
```

Do **not** crank `txqueuelen` on a Wi-Fi interface — the bloat lives in the firmware, not the qdisc.

---

## Recipe 4: Datacenter / cluster (sub-ms RTT, ECN-capable fabric)

```bash
# DCTCP is purpose-built for this: fine-grained ECN-marking with very
# small ssthresh reduction proportional to mark fraction.
sysctl -w net.ipv4.tcp_congestion_control=dctcp
sysctl -w net.ipv4.tcp_ecn=1
sysctl -w net.ipv4.tcp_ecn_fallback=1

# Tighter CoDel for sub-ms RTTs (the bufferbloat.net DC recommendation).
tc qdisc replace dev eth0 root fq_codel \
   limit 10240 target 500us interval 20ms

# Use BQL — required for any AQM to work right.
ls /sys/class/net/eth0/queues/tx-*/byte_queue_limits/limit
```

`tcp_notsent_lowat` is **mandatory** on multiplexed L7 servers (gRPC, HTTP/2, Envoy):

```bash
sysctl -w net.ipv4.tcp_notsent_lowat=16384
```

Without it, megabytes commit to the kernel send buffer and your prioritized streams get stuck behind low-priority bytes you already wrote.

---

## Recipe 5: CDN edge / web server tier (mixed clients, public internet)

```bash
# BBRv2/BBRv3 gives the best end-user latency on public internet
# (shallow buffers + variable BDP). BBRv1 retransmits 100x more — never use.
sysctl -w net.ipv4.tcp_congestion_control=bbr
sysctl -w net.core.default_qdisc=fq            # sch_fq required for BBR pacing

# Long-lived idle is common (HTTP/2, keep-alive). Disable the slow-start
# restart that silently kills the first response after every idle.
sysctl -w net.ipv4.tcp_slow_start_after_idle=0

# Receive buffer cap: 2-3x BDP, NOT "as big as possible".
# For 1Gbit @ 100ms RTT: BDP = 12.5 MB. Cap rmem max at 32 MB.
sysctl -w net.ipv4.tcp_rmem="4096 131072 33554432"
sysctl -w net.ipv4.tcp_wmem="4096 131072 33554432"

# HTTP/2 prioritization fix.
sysctl -w net.ipv4.tcp_notsent_lowat=16384
```

Validate with `nstat -az | grep -E 'TcpExtTCPRcvCollapsed|TcpExtRcvPruned'` — if either counter rises, your `tcp_rmem max` is wrong (too high, paradoxically).

---

## Recipe 6: GEO satellite (RTT ≈ 600 ms)

CUBIC's slow-start takes ~30 seconds to fill a 100 Mbit/600 ms pipe. Use Hybla or BBR with raised IW.

```bash
# Linux:
modprobe tcp_hybla
sysctl -w net.ipv4.tcp_congestion_control=hybla
# Or BBR with raised initial window for short-object speed:
ip route change default ... initcwnd 32 initrwnd 32

# Window scaling MUST be enabled (it is by default; don't let
# legacy "tuning guides" turn it off).
sysctl -w net.ipv4.tcp_window_scaling=1

# tcp_rmem max for 100 Mbit @ 600 ms BDP = 7.5 MB. Cap at 16 MB:
sysctl -w net.ipv4.tcp_rmem="4096 131072 16777216"
```

**Avoid BBRv1 here** — its `PROBE_RTT` state every 10 s causes a noticeable throughput dip on satellite. BBRv3 is better.

---

## Persisting and verifying

```bash
# Persist
cat >/etc/sysctl.d/99-jacobson.conf <<EOF
net.core.default_qdisc = fq
net.ipv4.tcp_congestion_control = bbr
net.ipv4.tcp_notsent_lowat = 16384
net.ipv4.tcp_slow_start_after_idle = 0
net.ipv4.tcp_recovery = 1
EOF
sysctl --system

# Verify the qdisc actually applied (a stacked qdisc from NetworkManager
# can silently override yours):
tc -s qdisc show dev eth0
```

## Decision: which recipe?

| Symptom you observed | Recipe |
|---|---|
| Home internet feels slow when someone uploads | Recipe 1 |
| VoIP/games unusable on DSL | Recipe 2 |
| Wi-Fi laggy under load even with fast upstream | Recipe 3 |
| Datacenter incast / microbursts | Recipe 4 |
| Web server tail latency, HTTP/2 streams blocked | Recipe 5 |
| Starlink / GEO sat slow page loads | Recipe 6 |

## Fallback if a recipe makes things worse

1. Save the previous qdisc: `tc qdisc show dev <iface> > /tmp/qdisc.bak`
2. Revert: `tc qdisc del dev <iface> root` then re-apply previous.
3. Most common cause: shaper rate too high → bottleneck queue stayed upstream. Drop another 5–10% and retry.
4. Second most common: hardware offload still on. Run `ethtool -k <iface> | grep -E 'tso|gso|gro'`.
5. Third most common: NetworkManager / systemd-networkd reapplied default qdisc. Disable the relevant unit's qdisc management.
