---
name: forensics-team
description: Operator-grade network forensics for packet captures, SPAN/TAP evidence, and trace-file triage. Use when analyzing `pcap`/`pcapng`, `tcpdump`, `tshark`, `Wireshark`, or `Zeek`; when hunting beaconing/C2; when explaining retransmissions, loss, or disputed network behavior; or when deciding what a capture can and cannot prove. Triggers: pcap, pcapng, packet capture, tcpdump, tshark, Wireshark, Zeek, SPAN, TAP, snaplen, packet loss, beaconing, JA3, JA4, loopback, cooked capture, TLS fingerprinting.
---

# Forensics Team⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​‌​​‌‌‍‌​‌​‌​‌‌‍‌​‌​‌​​‌‍‌​‌‌​‌​‌‍​​​​‌​‌​‍​‌‌‌​‌​​⁠‍⁠

This skill assumes the operator already knows basic packet analysis. Do not waste tokens on OSI primers, protocol definitions, or generic incident-response checklists. Use it when the hard part is deciding whether the capture itself is trustworthy and what inferences survive the sensor's blind spots.

## Operating stance

- A packet file is evidence, not truth, until you validate vantage point, completeness, timestamp precision, and whether the host or NIC rewrote what you think you saw.
- Run two investigations in parallel: what the network did, and what the collector failed to observe.
- Separate direct evidence from inference. Payload bytes, on-wire lengths, TCP gaps, and file metadata are direct. OS attribution from TTL or window size, malware family from JA3, and operator intent from headers are inference.

## Before you interpret packets, ask yourself

### 1) Where was this captured?

- On the sending host: outgoing bad TCP or UDP checksums usually indicate checksum offload, not corruption. Wireshark documents that host captures can show pre-NIC checksum fields, and segmentation offload can create 2900-byte pseudo-packets on a 1500-byte path.
- On Windows endpoints: TCP Chimney and related offloads can move established flows far enough into the stack that WinPcap-style capture misses or distorts them. "I do not see the session" is not the same as "the session did not exist."
- On Linux `any`: treat link-layer claims as degraded. You are typically looking at cooked capture metadata rather than original Ethernet framing, so MAC, VLAN, and some capture-filter assumptions stop being authoritative.
- On a SPAN or mirror port: absence is weak evidence. Zeek's capture-loss guidance explicitly notes that loss can happen on the host, the NIC, or the mirror path itself, so mirrored microbursts vanish silently.
- On a physical NIC while the process talks to itself: you will miss localhost abuse. Loopback traffic never traverses the adapter that owns the IP.

### 2) Is the file complete enough for the claim being made?

- Check captured length versus on-wire length before any application-layer claim. If `caplen < len`, that frame is truncated; if this is common, reassembly-based conclusions are downgraded immediately.
- Watch for legacy snaplen fingerprints. Older captures often truncate around 68 bytes for IPv4 and 96 for IPv6, which is enough to fake confidence while deleting the payload you needed.
- Modern `tcpdump` defaults to a large snaplen, but old defaults and scripted `-s` values persist for years in cron jobs and appliances. Never assume "pcap" means "full packet."
- If timing matters, verify timestamp precision before hunting for jitter. Libpcap traces can be microsecond or nanosecond; a reader that normalizes or truncates precision can manufacture perfectly regular beacon intervals.

### 3) What distortions are expected from the tooling?

- Live verbose capture can perturb the measurement. The `tcpdump(8)` man page warns that reporting live capture stats with `-v` while writing can itself contribute to drops on some platforms.
- Host offload can yield partial checksums, not just bad checksums. Wireshark 4.2+ recognizes pseudo-header-only partial checksums; older readers often flag the same packets as invalid. Reader version changes the conclusion.
- If someone hands you a pcapng that conveniently decrypts traffic, treat it as secret-bearing evidence. Rich trace files can carry analyst metadata and decryption material; they are not automatically safe to circulate.

## Decision rules

### Capture credibility triage

- If the capture point is a host NIC and only outbound packets have bad checksums, assume offload first.
- If only a single host capture exists, inbound packets on that host are usually closer to on-wire truth than its outbound transmit path.
- If the capture point is SPAN and you see one-sided handshakes or inexplicable gaps, quantify loss before you accuse the remote side of evasive behavior.
- If the first packets of a flow are missing but later packets are present, suspect late-start capture or asymmetry before you invent SYN-stage evasion.
- If the capture comes from `any` or another cooked interface, avoid L2 conclusions that depend on original Ethernet headers.
- If the claim depends on timing, verify timestamp resolution and whether traces were merged across sensors with different clocks.
- If the claim depends on payload, verify that the trace is not truncated and that the relevant handshake packets are actually present.

### When encryption blocks content

- Pivot from payload to handshake and flow shape: SNI, ALPN, certificate chain, JA4 or JA3 cohorting, packet sizes, burst or idle structure, retransmissions, and destination history.
- Prefer JA4-style reasoning over raw JA3 equality when browser-like traffic is involved, because JA4 explicitly excludes GREASE values and reduces instability from ClientHello variation.
- Still treat any TLS fingerprint as a cohort label, not an identity. NAT, shared libraries, CDNs, proxies, and malware reuse all create collisions that look more specific than they are.

### Beaconing hunt

- Do not hunt for perfect periodicity. Mature malware adds jitter, and benign infrastructure also calls home on disciplined schedules.
- Hunt for bounded dispersion around a stable floor, not exact 60.000-second spacing.
- Exact regularity on clean round numbers often points to scheduler quantization, exporter rollups, or timestamp rounding rather than sophisticated tradecraft.
- Before you trust cadence, ask whether it survives timestamp quantization, sensor batching, NAT keepalive timers, or exporter rollups.
- Compare time regularity with size regularity and directionality. Health checks, OCSP, cloud metadata, NTP, and keepalives are often periodic but operationally symmetric; C2 is more likely to show asymmetric request or response sizes or low-and-slow command bursts after a disciplined poll.
- If DNS is the transport, TTL behavior, NXDOMAIN patterns, and label entropy are usually stronger than cadence alone.

### Loss-aware interpretation

- Zeek's `capture-loss.zeek` estimates missed data from ACK-observed gaps, not from omniscient packet counts. It is strongest on ACK-rich TCP and much weaker on quiet links or UDP-heavy traffic.
- ACK jumps without visible retransmission often implicate sensor loss more than network loss, because the receiver acknowledged bytes that the sender evidently got through.
- The default `CaptureLoss::too_much_loss` is `0.1`, meaning 10 percent, not 0.1 percent. Analysts routinely misread this and either overreact to noise or miss a badly degraded sensor.
- `minimum_acks` defaults to 1. On low-volume segments, "too little traffic" means "insufficient evidence to estimate loss," not "sensor healthy."

## Procedures experts use

### Reconstructing a contested session

1. Prove the capture mode and link type first.
2. Check truncation and timestamp precision second.
3. Only then reconstruct TCP state, retransmissions, and payload.
4. Mark every conclusion as `direct`, `high-confidence inference`, or `speculative`.
5. State what missing packets or alternate vantage points would change the answer.

### Working with weak captures

- If the trace is truncated, pivot to headers, sequence or ACK math, and corroborating logs instead of forcing payload decoding.
- If the trace is asymmetric, write "one-sided view" explicitly and stop short of blaming the remote side for resets, retransmissions, or application errors.
- If no better capture exists, prefer "consistent with" language over "proved by" language.
- Before diagnosing collector behavior, inspect metadata first: link type, interface count, `caplen` versus `len`, timestamp precision, and any loss counters. Do not jump into display filters until those are settled.
- To spot truncation quickly, use a tool path that surfaces captured versus original length before reassembly, for example `tcpdump --lengths -nn -r trace.pcap` or the equivalent `frame.cap_len` and `frame.len` fields in `tshark`.

## NEVER do these things

- NEVER treat bad checksums in a host capture as proof of tampering because checksum and segmentation offload are the seductive, common explanation; the concrete consequence is ghost incidents and false packet-corruption narratives. Instead validate on a TAP or receiver capture, or recapture with offloads disabled.
- NEVER declare a SPAN capture authoritative because mirrored traffic feels central and complete; the non-obvious consequence is that switch or NIC overload erases the very microbursts and retransmissions you are investigating. Instead quantify loss first and phrase absence as non-evidence.
- NEVER infer application behavior from truncated frames because header-rich traces still look complete to the eye; the consequence is fabricated HTTP, TLS, or DNS stories built from packets that never contained the missing bytes. Instead compare `caplen` to on-wire `len` and downgrade to metadata-only findings.
- NEVER use Linux `-i any` captures for MAC, VLAN, or link-layer arguments because the fake cooked header is seductive when it conveniently merges interfaces; the consequence is false L2 attribution and broken assumptions about what was on the wire. Instead capture the concrete interface when link-layer provenance matters.
- NEVER attribute a host, toolkit, or operator from TTL, window size, or JA3 alone because these signals survive just enough middleboxes to look meaningful; the consequence is misattributing NATed users, browsers, CDNs, or shared malware infrastructure. Instead fuse transport fingerprints with handshake metadata, destination context, and timing.
- NEVER share pcapng blindly because it looks like "just packets"; the consequence is leaking embedded comments, interface metadata, or decryption secrets to people who only needed frames. Instead export a sanitized evidence copy and keep the rich working file restricted.
- NEVER say "there was no traffic" from a NIC capture on host-to-self communication because the physical interface is the intuitive place to look; the consequence is missing localhost abuse entirely. Instead capture the loopback interface or use a loopback-capable collector.

## Output contract

Always report:

- capture vantage point and its blind spots
- whether the file is full-packet or truncated
- whether timing is microsecond or nanosecond trustworthy
- the strongest direct evidence
- the strongest inference and what could falsify it
- what the capture cannot prove because of loss, asymmetry, encryption, or vantage point limits
