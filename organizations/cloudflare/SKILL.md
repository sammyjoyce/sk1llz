---
name: cloudflare-performance-engineering
description: Design and review latency-critical edge, proxy, cache, and packet-processing systems using Cloudflare-style heuristics for XDP/eBPF, connection reuse, tiered cache, anycast routing, and QUIC/HTTP/3. Use when building or debugging CDN, reverse proxy, DDoS mitigation, origin routing, or edge compute paths where tail latency and failure amplification matter. Triggers: xdp, ebpf, pingora, quic, http/3, anycast, tiered cache, origin shield, connection reuse, tail latency, ddos, cache collapse.
tags: cdn, networking, performance, edge, dns, http, tls, proxy, scale, systems, rust
---

# Cloudflare Performance Engineering

## Use this skill for

- Request paths where a local optimization can create a fleet-wide regression.
- Systems that mix edge cache, origin fetch, anycast routing, connection pooling, protocol tuning, or packet filtering.
- Design reviews where the real problem is tail behavior under skew, not average latency in a clean benchmark.

## Load only what matches the task

- Before changing XDP/eBPF packet handling, read `L4Drop: XDP DDoS Mitigations` and `Cloudflare architecture and how BPF eats the world`.
- Before changing reverse-proxy reuse, pool topology, or request lifecycle logic, read `How we built Pingora` and `Resolving a request smuggling vulnerability in Pingora`.
- Before enabling 0-RTT, HTTP/3, or QUIC transport shortcuts, read `Introducing 0-RTT` and `Defending QUIC from acknowledgement-based DDoS attacks`.
- Before tuning tiered cache, origin shielding, or route steering, read `Regional Tiered Cache`, `Orpheus`, and `Partial Cloudflare outage on October 25, 2022`.
- Do not load Workers or product-API material unless the bottleneck is isolate execution or cache semantics; this skill is about network-path engineering, not dashboard configuration.

## Mindset

- Treat performance as work avoided, not instructions executed. Ask: which packet, request, handshake, cache miss, or retry can disappear entirely?
- Optimize for p99 under skew. Ask: what happens when one core, one POP, one origin path, or one cache tier is unlucky?
- Treat every fast path as a separate correctness surface. Cache hits, resumed sessions, reused sockets, and coalesced connections are where invariants get skipped.
- Assume the control plane eventually lies. At this scale, a header normalization, verifier rewrite, or routing shortcut can turn a local change into global damage.
- Prefer reuse only when it reduces more handshake, origin, or queueing cost than the coordination it introduces.

## Before doing anything risky, ask yourself

- If this fast path misclassifies one request, does it fail closed, fail open, or poison the next request?
- If one lower tier loses a control header, can the upper tier still resolve and route the request correctly?
- If one origin path degrades, will the system retry smarter, route around it, or amplify the failure with more misses and more connections?
- Is the expensive part actually compute, or is it handshake, queueing, route instability, cache fan-out, or worker imbalance?
- Which deployment-kernel or client-protocol assumptions are baked into this change?

## Decision rules

### Where should the work happen?

- Use XDP only for decisions that can be made from early packet bytes and are stable enough to justify compile-and-roll operations.
- Stay in userspace when rules churn, matching logic spans arbitrary headers/options, or verifier/code-complexity limits will dominate your ops burden.
- Use socket-level BPF when XDP cannot fully protect a shared UDP service and you must stop one flooded IP from starving unrelated traffic on the same socket.

### Is connection reuse helping or hurting?

- Reuse aggressively across workers when origin handshake cost dominates. Cloudflare moved one customer from 87.1% to 99.92% reuse and cut new origin connections by 160x.
- Reevaluate keepalive on local hops if reconnect cost is tiny but worker imbalance is expensive. Cloudflare improved tail latency by disabling a local keepalive hop because landing on a hot worker cost more than reconnecting.
- Treat reused HTTP/1.1 connections as a security boundary. If the lifecycle can exit early, prove unread-body handling before you celebrate the latency win.

### Which cache topology?

- Use Smart Tiered Cache when protecting origin connections is the primary goal and a single best upper tier is acceptable.
- Add a regional tier when the chosen upper tier is far from major traffic regions; Cloudflare reported 50-100 ms tail improvements for that pattern.
- Avoid many-upper-tier topologies when origin fan-out is the real bottleneck. They reduce some remote misses by quietly recreating the origin-thundering problem tiering was meant to stop.

### Should you enable protocol fast paths?

- Enable 0-RTT only for requests you can prove are replay-safe in production, not merely "idempotent by convention".
- Treat ACKs, PINGs, and adaptive-window behavior as abuse surfaces, not just control signals.
- Use connection coalescing only when certificate coverage, reachability, and routing all line up; shared SANs alone are not enough.

## Practitioner heuristics

- XDP is valuable partly because it can stay always-on. Threshold-triggered mitigations create cliff behavior exactly when attack traffic ramps.
- Dynamic rules in eBPF maps are seductive, but complex packet predicates often hit verifier or expressiveness limits. Cloudflare compiled specialized programs when arbitrary cross-header matching mattered.
- Sample before drop if downstream analytics or auto-mitigation depends on seeing the attack. Dropping first blinds the detector that is supposed to refine the rule.
- Kernel version is part of eBPF program semantics. The verifier can rewrite instructions for safety, so validate generated or JITed behavior on the target kernel, especially around non-trivial 64-bit arithmetic.
- Reuse fewer, better connections. The win is not only RTT; it is reduced origin accept pressure, lower TLS CPU, and less pool fragmentation across workers.
- Treat cache-hit code as separate code, test, and security surface. Pingora's request-smuggling flaw existed because cache hits skipped cleanup logic that miss paths already had.
- Route around bad Internet weather instead of retrying the same broken path harder. Many 522-style failures are path failures, not origin failures.
- Canary observability on live traffic is often more truthful than synthetic load. Even low-overhead eBPF instrumentation can distort a hot path enough to hide the real bottleneck.
- Metadata contracts deserve the same rigor as data contracts. Cloudflare had a partial outage because header clearing that looked harmless broke lower-tier to upper-tier routing.
- QUIC defenses need explicit invalid-ACK handling. If you trust invented ACK ranges or optimistic ACK behavior, a peer can manufacture tiny RTT signals, inflate send rate, and turn fairness logic into a DDoS vector.

## NEVER do these

- NEVER push packet classification into XDP just because kernel-space is faster, because verifier constraints and program-complexity ceilings turn every rule change into brittle compile-and-roll risk. Instead keep only high-volume, early-reject predicates in XDP and leave churn-heavy logic in userspace.
- NEVER assume keepalive is always good, because reuse can pin requests to hot workers and worsen p99 while the median improves. Instead compare reconnect cost versus scheduler imbalance on the exact hop you are tuning.
- NEVER enable 0-RTT for "probably safe" endpoints, because replay failures are state-corruption bugs wearing a latency-optimization costume. Instead whitelist explicitly replay-tolerant endpoints and propagate replay signals to the application.
- NEVER treat ACKs, PINGs, or adaptive-window hints as trustworthy, because a peer can convert them into CPU amplification or unfair send-rate growth. Instead validate ranges, rate-limit liveness signals, and close on protocol abuse.
- NEVER widen tiered-cache fan-out to shave a few far-region misses without checking origin connection budgets, because multiple upper tiers quietly recreate the origin-flood pattern tiering was supposed to remove. Instead choose topology from the dominant constraint: origin protection or remote-region tail latency.
- NEVER clear or normalize internal routing headers on shared paths "for safety", because tier-to-tier traffic often depends on metadata the request still looks valid without. Instead document the invariant and test lower-tier to upper-tier flows as first-class paths.
- NEVER reuse an HTTP/1.1 connection after an error or early-exit path unless request-body consumption is proven, because leftover bytes become the prefix of the next request. Instead drain, refuse reuse, or terminate the connection.

## Failure playbooks

### Rising 522 or 530 rates while origins look healthy

- Suspect path selection, tier metadata, or internal header loss before blaming the origin.
- Compare failing POP and origin-route pairs, not just aggregate origin health.
- If smart routing exists, verify it is selecting alternate paths instead of hammering the degraded route.

### CPU spikes after enabling a "performance" feature

- Check whether you created more handshakes, more retries, more ACK or PING work, or more origin connections.
- For eBPF/XDP changes, inspect verifier-driven rewrites and map-access patterns before blaming the NIC or driver.

### Cache hit ratio improved but tail got worse

- Look for geographically remote upper tiers, regional miss penalties, or shield placement that helps the median while hurting outlier regions.
- Consider adding a regional tier or changing the "single best" upper-tier heuristic.

### Origin load fell but correctness incidents rose

- Audit every shortcut taken on cache-hit, reused-connection, and resumed-session paths.
- Look for skipped cleanup, skipped validation, or stripped metadata on the "fast" branch.

## What good looks like

- A packet or request is either dropped early, served locally, or forwarded once with maximal safe reuse.
- Fast paths preserve routing and security invariants instead of bypassing them.
- Tail latency remains bounded when traffic is skewed, routes flap, or origins partially degrade.
- Instrumentation tells you whether the dominant cost is handshake, queueing, routing, cache topology, worker imbalance, or origin behavior before you touch code.

## Load-on-demand sources

- `How we built Pingora, the proxy that connects Cloudflare to the Internet`
- `Keepalives considered harmful`
- `L4Drop: XDP DDoS Mitigations`
- `Cloudflare architecture and how BPF eats the world`
- `Introducing Zero Round Trip Time Resumption (0-RTT)`
- `Defending QUIC from acknowledgement-based DDoS attacks`
- `Reduce latency and increase cache hits with Regional Tiered Cache`
- `Improving Origin Performance for Everyone with Orpheus and Tiered Cache`
- `Partial Cloudflare outage on October 25, 2022`
- `Resolving a request smuggling vulnerability in Pingora`
