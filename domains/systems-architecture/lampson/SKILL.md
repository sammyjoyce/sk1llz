---
name: lampson-system-design
description: "Design long-lived computer systems in Butler Lampson's style: write abstract state and actions first, keep lower layers replaceable, put correctness end-to-end, and optimize for repair, tail latency, and evolvability. Use when choosing interfaces, retry/replication/transaction strategies, control-plane vs data-plane splits, storage hardening, or security boundaries. Trigger keywords: abstraction, spec, end-to-end, idempotence, retry, OCC, snapshot isolation, commit record, failover, repair, capability, least common mechanism, control plane."
---

# Lampson System Design

Use this skill for boundary decisions and failure-model choices. It is not for naming, formatting, or code-style cleanup inside an already-stable design.

## Mandatory loading triggers

- Before changing an interface, consistency promise, retry policy, or failure contract: READ `philosophy.md`.
- Before writing a design rationale, review, or architecture memo: READ `references.md`.
- Do NOT load either file for mechanical edits that do not change semantics at a boundary.

## Before doing anything architectural, ask yourself

- What is the **abstract state**? Write it as data the client can name, not as tables, caches, threads, or shards.
- What are the **actions** on that state, and which visible effects are safety properties versus liveness properties?
- Which costs are actually binding in this regime: compute, storage, network, or queueing? Re-check; bottlenecks move over time.
- Which guarantees belong **end-to-end** because only the application has enough knowledge?
- Where do you need **prompt answers** versus merely **eventual repair**?
- What is the **repair loop** after the first fault? If the answer is "operator later", you do not yet have a dependable design.
- Which parts must be distributed for scale or fault domains, and which parts should stay centralized because distribution only adds partial failures?

If you cannot answer these questions in one page, you are still designing the wrong layer.

## Lampson working loop

1. **Write the state and actions first**
   - A spec is a constraint on visible behavior, not a disguised implementation.
   - Include resources and time when clients must reason about them. Hiding queueing or timeout behavior that clients must survive creates a leaky spec.
   - Keep nondeterminism in the spec when it frees the implementation. Publishing incidental order, timing, or placement semantics is how teams accidentally freeze bad internals.

2. **Decide where correctness lives**
   - If a function can be implemented correctly only with application knowledge, lower layers may offer it only as a performance enhancement.
   - End-to-end checks remain required for duplicate suppression, ordering, commit acknowledgment, encryption authenticity, and "did the action really happen?"
   - Do not confuse a transport ACK with an application ACK. "I got the packet" and "I applied the effect" are different contracts.

3. **Separate fast path, slow path, and repair path**
   - Treat retry as a slow path with explicit economics. If success costs `f`, one retry costs `r`, and failure probability is `p`, slowdown stays small only when `p << f/r`.
   - Lampson's concrete warning: if retry costs `10f`, failure probability must be well below 10%; otherwise retries become a major part of steady-state cost.
   - If a component fails half the time and one retry costs 3x a success, the operation takes about 6x too long. At that point you need repair, not optimism.

4. **Control tails before chasing averages**
   - Design to p99/p99.9, not just mean latency.
   - When tails matter, quotas, admission control, and load shedding beat micro-optimization.
   - Fragment bursty work into many more pieces than workers, and bound the largest fragment so stragglers cannot dominate. But stop before per-fragment overhead becomes the bottleneck.

5. **Centralize by default; distribute deliberately**
   - If you have a choice, centralize. Distributed systems buy you partial failures, concurrency hazards, and communication cost.
   - Distribute only for real fault tolerance, locality, or scale beyond one box.
   - Keep the control plane centralized even when the data plane is distributed; large cloud systems do this because management rarely needs the same scale as data movement.

6. **Prefer repairable state representations**
   - Represent state as both **being** and **becoming** when you can: current snapshot for reads, log/history for replay, audit, and reconstruction.
   - Logs and checkpoints compose well with redo, replicated state machines, and postmortems.
   - Pure current-state replication is harder to repair because it loses the explanation of how corruption or divergence happened.

## Heuristics that usually separate experts from beginners

- **Do not overspec latency-shaped interfaces.** RPC that looks like a local call is dangerous because it hides unpredictable network cost. If a user interaction can touch the network, make the UI asynchronous unless the network dependency is obvious to the user.
- **Recompute old intuitions.** AES of a cache line can be about 50 cycles while a cache miss is about 200 cycles; abstractions built around "crypto is always the slow part" are often stale.
- **"Good enough" needs a number.** Examples Lampson uses: 99.5% availability rather than 100%, response under 200 ms with 99% probability, 98% cache hit rate rather than perfection, within 10% of optimal rather than optimal.
- **Availability is mostly MTTR, not heroic MTTF.** A useful approximation is `availability loss ~= MTTR / MTTF`, and MTTR is failover time, not bench repair time. Five nines is about five minutes of downtime per year.
- **Redundancy without repair is a countdown, not resilience.** Mirroring without scrubbing only hides the first fault. Measure corrected errors, latent corruption, and partial failures, then repair continuously.
- **Identify the real endpoints before invoking the end-to-end argument.** Live voice prefers an occasional damaged packet over retransmission delay; stored voice often wants the opposite. The same payload can justify opposite lower-layer behavior because the endpoint contract changed.
- **Optimize degraded mode, not just pristine mode.** In Azure storage, transient and offline-node cases were the dominant failures, so reconstruction bandwidth and degraded-read latency mattered more than elegant steady-state code properties.
- **Code hot mutable data differently from cold sealed data.** Azure kept data triply replicated while extents were hot and mutable, then lazily erasure-coded sealed ~1 GB extents in the background. That pattern avoids pushing erasure-coding penalties into the write path.
- **Erasure coding is an operational trade, not a purity win.** LRC `(12,2,2)` hit a 1.33x storage overhead target and cut repair I/O/bandwidth compared with Reed-Solomon, but only after choosing layouts across fault and upgrade domains and accepting more machinery.
- **Use OCC only when conflicts are truly rare.** Its fast path is seductive, but under load it can collapse into abort storms. When conflicts are common, waiting is cheaper than wasted work.
- **Wait-free is not magic.** CAS-based updates prevent a slow lock holder from stopping others, but under contention retries can livelock unless conflicting threads help complete one another's updates.

## Decision trees

### Interface and spec

- If clients need the behavior to reason about correctness, put it in the spec.
- If only the implementation needs it, keep it out of the spec.
- If lower layers offer reliability, ordering, or deduplication, specify them as hints unless they are the final correctness boundary.

### Atomicity and concurrency

- If conflicts are rare and work is short: consider OCC or MVCC.
- If conflicts are common: prefer locks or other explicit serialization.
- If transactions are long-running: snapshot-style approaches beat holding locks for the whole duration.
- If updates must stay non-blocking: use multi-version plus CAS only with a help path for contention.

### Retry, redo, or replicate

- If failures are transient and detectable: retry with idempotence and exponential backoff.
- If failures are crashes with persistent intent: use redo logs and replay.
- If failures are node/media loss: replicate or code data, then design the repair bandwidth and topology explicitly.
- If reconstruction dominates user-visible latency: reduce the amount of data read during repair before inventing a fancier durability story.

### Transactions across boundaries

- If all participants are in one ownership domain and strict atomicity matters: use a log-backed transaction and a consensus-backed commit record for fault tolerance.
- If participants cross organizations or teams with independent operations: do not assume global ACID will survive politics or lock ownership. Use compensating steps and idempotent messages.
- If the real requirement is "eventual business completion" rather than strict atomic visibility: model compensation first, not last.

### Authorization path

- Store policy where administrators can answer "who has access to this resource?" That usually means list-oriented ACL management.
- Use short-lived ticket or capability-like handles for fast checks on the hot path; ticket-oriented checks are cheaper than associative ACL lookup.
- If revocation must be immediate, be skeptical of cached authorization decisions; complete mediation and cached answers are in tension.
- If you combine ACL administration with capability execution, design the synchronization path first; revocation bugs usually live in the gap between the two.

## Anti-patterns

- **NEVER put a correctness guarantee in a low layer just because many clients want it.** That is seductive because it looks like reuse, but applications still need end-to-end checks for crashes, duplicates, or misapplied effects. Instead expose the lower-layer feature as an optional acceleration and keep the final correctness proof at the endpoint.
- **NEVER publish incidental timing or ordering behavior as part of the interface.** It feels "honest," but clients will ossify around today's bottleneck profile and you will lose the freedom to change the implementation. Instead specify only the latency classes or ordering guarantees clients truly need.
- **NEVER default to a distributed design because distribution sounds scalable.** The seductive part is horizontal-growth rhetoric; the consequence is partial failures, queueing, and debugging states that do not exist in one box. Instead centralize until locality, availability, or throughput forces distribution.
- **NEVER treat redundancy as sufficient.** It is seductive because replicas make dashboards look green, but a failed replica without scrubbing or rebuild means the next fault becomes user-visible loss. Instead define repair cadence, corrected-error telemetry, and failover MTTR up front.
- **NEVER use OCC on a hotly contended path because the no-lock fast path looks elegant.** Under real contention it degenerates into wasted work and retry storms. Instead serialize the hot region or add structure that reduces conflicts before reaching for OCC.
- **NEVER hold prepared distributed work across organizational boundaries because ACID feels safer.** The consequence is lock ownership tied to other people's outages and deployment schedules. Instead use consensus-backed commit only within one authority domain; across domains use compensation and idempotence.
- **NEVER factor shared infrastructure to the union of everyone's needs.** That is seductive because it looks like maximal reuse, but it drags specialized behavior into a shared mechanism and enlarges the failure surface. Instead use the **greatest common mechanism** outside security-sensitive code and the **least common mechanism** inside the TCB.
- **NEVER cache authorization results without an explicit revocation story.** It is tempting because permission checks are hot-path overhead, but stale privilege is a silent security bug. Instead cache short-lived capabilities or build invalidation into policy changes.

## Security and trust boundaries

- Use the smallest TCB you can explain. End-to-end encryption shrinks the set of components that must be trusted for secrecy and integrity.
- Physical isolation is simpler than software isolation; VMs are simpler than OS process isolation because the interface is smaller and better specified.
- Different parties optimize different failures. A vendor may prefer hidden corruption over visible parity faults because the blame lands elsewhere; design your telemetry for the operator's interests, not the vendor's PR interests.
- Protection dynamics matter as much as static policy. Revocation during active use, re-auth on recovery, and maintenance-mode access are where "check every access" usually breaks.

## Fallback playbook when the first design stops working

- If retries start driving the tail: cap retries, add backoff, and move completion to async reconciliation before tuning the happy path.
- If a shared subsystem keeps accreting special cases: cut it back to the greatest common mechanism and push specialized behavior to adapters or libraries.
- If erasure-coded recovery hurts live traffic: keep hot writes replicated longer and postpone coding until objects seal or cool down.
- If revocation latency becomes unacceptable: shorten ticket lifetime, reduce cache duration, and shift more checks back to centralized policy.

## Stop only when these are explicit

- The abstract state and the allowed actions on it.
- Which guarantees are end-to-end contracts and which are performance hints.
- Tail targets and what you will shed or defer to keep them.
- The repair loop: detection, failover, rebuild, audit.
- The concurrency choice and the failure mode when contention rises.
- The ownership boundary where global atomicity stops and compensation starts.
