---
name: jump-trading-fpga-hft
description: Expert playbook for FPGA-first electronic trading systems where wire-to-wire latency, determinism, and feed correctness dominate. Use when building or reviewing tick-to-trade paths, pre-trade risk gates, multicast A/B feed arbitration, 10G or 25G low-latency links, PTP or 1PPS timestamping, microwave-vs-fiber path choices, or FPGA/CPU partitioning for HFT. Triggers: FPGA trading, HFT, low latency, FEC, PTP, multicast gap recovery, cut-through, pre-trade risk, microwave, tick-to-trade.
tags: fpga, hft, low-latency, trading, hardware, networking, market-data, timestamping
---

# Jump Trading FPGA HFT

## Read Only What Matters

- Before changing links, optics, or switch choice, read `Latency Budget and Link Rules`.
- Before changing feed handlers or book state, read `Feed Arbitration and Recovery`.
- Before changing timestamps, audit clocks, or compliance timing, read `Timing Discipline`.
- Before changing WAN path selection, read `Microwave vs Fiber`.
- Do not spend tokens on `Microwave vs Fiber` for same-rack or same-switch work.
- Do not spend tokens on `Timing Discipline` for pure strategy math with no external time requirement.

## Operating Mindset

Before moving logic into FPGA, ask yourself:

- Is this rule latency-critical, or just frequently executed?
- Is the rule structurally stable enough to survive timing closure and production change control?
- If this logic is wrong for 200 ns, do I lose money once, or poison state for minutes?
- Can I prove correctness under gaps, duplicates, retransmits, and stale packets?

Jump-style principle: freeze only the part that must be deterministic at line rate. Everything else stays where it can change safely.

## Partitioning Heuristics

- Put in FPGA: fixed protocol parsing, A/B line arbitration, top-of-book or bounded-depth state, hard pre-trade gates, pacing, deterministic order construction.
- Keep on CPU or host software: fast-changing models, anything needing dynamic memory, full-history analytics, reconciliation, and slow-control logic.
- Counterintuitive rule: a smaller FPGA design that closes timing cleanly often beats a "richer" design that adds routing pressure, BRAM contention, and jitter.
- If the strategy trades only top-of-book, do not rebuild full depth on the hot path. Mirror full depth off-path for audit and research.

## Latency Budget and Link Rules

- Queueing kills more alpha than propagation inside a rack. A switch that adds milliseconds during a microburst turns fresh data into stale data; many handlers will mark it stale or drop it.
- 25G is not automatically faster than a good 10G trading link. On short, clean channels, 25G without FEC helps. Blindly enabling FEC does not.
- Practical 25G FEC costs matter: BASE-R adds about 80 ns, RS-FEC about 250 ns per link. A few careless hops can burn half a microsecond before strategy logic runs.
- Use BER to decide, not link-up vanity. Intel's guidance is a good engineering gate: no FEC only when BER is below `1e-12`; BASE-R when BER exceeds `5e-8`; RS-FEC when BER reaches `5e-5`.
- Real bursts are spiky, but not infinitely spiky. Arista measured BATS around `86 Mb/s` on a one-second average and about `382 Mb/s` over a one-millisecond slice: enough to punish bad buffering on 1G, not enough to justify fake `23:1` multicast fan-in demos on 10G.
- Prefer same-switch placement for feed handler and execution host. One fewer hop improves both median latency and tail stability.
- Deterministic L1 or L1+ devices exist for a reason. Single-digit-nanosecond switching and mux latencies on Arista 7130-class gear are materially different from general-purpose switch behavior when the rest of the stack is already tuned.
- Benchmark wire-to-wire and `p99.9`, not just median. A 20 ns faster median with rare 100 us outliers is a worse trading system.

## Feed Arbitration and Recovery

- Track last sequence independently for line A and line B.
- On a gap on one line, stop applying later packets, cache out-of-order traffic briefly, and wait for the missing sequence from the peer line.
- If both lines miss it, use retransmission. If state is no longer provably correct, clear and rebuild from snapshot or refresh before rejoining real time.
- Use dual-feed arbitration before retransmission whenever possible. Exchanges such as NYSE explicitly structure recovery in that order.
- Never let the strategy consume "best effort" book state after a gap. One poisoned book costs more than months of latency work.
- Keep recovery entirely in hardware or entirely behind a clean ownership boundary. Half-hardware, half-software gap logic creates heisenbugs that only appear under burst loss.

## Timing Discipline

- Hardware timestamps are mandatory for nanosecond claims. Software timestamps are for ops dashboards, not trading truth.
- Do not discipline the FPGA clock by syncing only a host NIC with PTP. Without `1 PPS` or PTP packets flowing through the FPGA, the NIC, motherboard, and FPGA clocks drift independently.
- Commodity oscillators around `20-30 ppm` can drift by about `20 us` per second and roughly `1.7 s` per day if left uncorrected. If holdover matters, bring in `1 PPS` or a `10 MHz` reference.
- Never step time backwards. Some capture and trading software breaks on backward timestamps. Slew the clock rate with a servo instead.
- Use boundary or transparent clocks when cross-box correlation matters; ordinary Ethernet switches degrade PTP accuracy.
- Read MAC, PHY, and FEC errata before trusting a timing path. Example: Intel Stratix 10 25G IP with IEEE 1588 plus RS-FEC had an RX timestamp shift bug in Quartus `<=21.3` that caused about `10 ns` error.

## FPGA Implementation Rules

- Implement only the protocol variation and fields the hot path actually needs. A cut-down parser is often dramatically faster than a generic decoder that handles every optional branch.
- Emit as soon as decisive fields are known, but only if all mandatory risk and compliance checks still complete before the order leaves the box.
- Optimize for the hot operation mix, not textbook purity. Arbitrary delete by scanning is fatal; constantly updating an on-chip hashmap can also be a trap. Data structures that privilege top-of-book mutation often win, and order-book mutation around `80 ns` is achievable when the structure matches the workload.
- Measure full fabric, not modules in isolation. Sub-microsecond FPGA trading paths have still lost meaningful latency in stream interconnect and glue logic, even when the business blocks themselves looked fast on paper.
- Every BRAM lookup, clock-domain crossing, and "nice to have" normalization stage must earn its place in nanoseconds. Clean architecture that misses timing closure is not clean architecture.

## Microwave vs Fiber

- Air beats glass on propagation, but microwave is not a free win. If the route is materially longer than the fiber path, or weather fade forces retransmission or failover, the edge disappears into jitter.
- Use microwave where the propagation advantage is real and failure handling is preplanned. Use fiber as the deterministic fallback, not as the thing you discover during a storm.
- Weather-aware routing is trading infrastructure, not telecom garnish. Rain fade and line-of-sight issues turn "fastest path" into "fastest path until the market moves."
- Do not choose a WAN path on one-way latency alone. Choose on worst-case arrival time of tradable data.

## Anti-Patterns

- NEVER enable RS-FEC by default because clean link-up is seductive. Instead qualify BER and disable FEC on short, clean trading links, or accept that each hop now carries roughly `250 ns` of avoidable latency.
- NEVER benchmark contrived many-to-one multicast fan-in and call it "real trading" because it flatters mediocre switches. Instead test real A/B fan-out, arbitration, retransmission, and stale-packet handling.
- NEVER sync only the host NIC and assume the FPGA clock is now correct because dashboards look aligned. Instead drive `1 PPS` or `10 MHz`, or pass PTP through the FPGA and servo that clock directly.
- NEVER rebuild full-depth book state in hardware when the strategy only trades top-of-book because "complete" feels safer. Instead keep only latency-critical depth on the hot path and mirror the rest off-path.
- NEVER let mutable business logic accrete in FPGA because "one more branch" feels cheap. Instead keep deterministic gates in hardware and move frequently changing logic out before routing pressure explodes.
- NEVER trust mean latency because it hides the failures that lose money. Instead track wire-to-wire tails, stale-data drops, and gap-recovery behavior under burst.
- NEVER step timestamps backwards because it seems like the simplest correction. Instead slew time with a servo so downstream software never sees time reversal.

## Failure-Driven Decisions

- If timing closure fails: remove optional protocol branches, narrow state, split mutable logic to CPU, cut clock-domain crossings, and only then chase a faster device or clock.
- If feed gaps outgrow the arbitration window: stop trading, snapshot-rebuild, and widen observability before tuning queues.
- If PTP accuracy misses budget: verify hardware timestamps first, then switch type, then `1 PPS` or `10 MHz` presence, then vendor errata. Blaming software first is usually wrong.
- If a 25G link requires RS-FEC to stay up: accept the new latency class explicitly, or shorten and clean the channel. Do not pretend it behaves like a no-FEC trading link.

## What Good Looks Like

- Fresh packets arrive, gaps are detected immediately, and no strategy sees state that is not provably ordered.
- Clock discipline is explicit and measurable, with no confusion between host time and FPGA time.
- The FPGA bitstream contains only stable, latency-critical logic.
- The network path is chosen for worst-case tradable arrival time, not marketing bandwidth.
