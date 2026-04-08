---
name: lamport-formal-distributed
description: "Formalize, model-check, and de-risk distributed protocols the Lamport way: isolate safety first, add liveness only with explicit synchrony assumptions, use refinement mappings to connect specs to optimized implementations, and choose the right Paxos-family variant for reconfiguration, leases, quorum shape, or Byzantine faults. Use when designing or reviewing consensus, replication, logical clocks, TLA+, PlusCal, TLC, TLAPS, Multi-Paxos, Fast Paxos, Flexible Paxos, Vertical Paxos, Cheap Paxos, leases, quorums, or refinement mappings."
tags: paxos, tla+, pluscal, tlc, tlaps, consensus, quorum-systems, refinement-mapping, logical-clocks, leases, reconfiguration, byzantine
---

# Lamport Formal Distributed

This skill is for tasks where informal reasoning is the bug. Use it when the work hinges on invariants, quorum math, reconfiguration, lease safety, or proving that an optimization still implements the original protocol.

## Load Only What You Need

- Before writing historical or philosophical material about Lamport, READ `references/philosophy.md`.
- Before showing or editing a Python logical-clock demo, READ `scripts/lamport_clock.py`.
- Do NOT load `references/philosophy.md` for quorum math, Paxos debugging, TLA+ modeling, or lease analysis.
- Do NOT load `scripts/lamport_clock.py` for consensus, reconfiguration, fairness, or proof work.

## Start Here

Before doing anything, ask yourself:

- What exact bad state must be impossible? Write that as an invariant before discussing algorithms.
- What assumption makes progress possible? Name the leader, timing, quorum, or recovery assumption explicitly; if you cannot state it, you do not have a liveness story.
- Which information must survive a crash? In Paxos terms, stable storage is usually the real boundary between a proof and a fantasy.
- Is the proposed optimization an implementation of the same abstract machine, or a different protocol that merely feels equivalent?

## Operating Procedure

### 1. Separate the problem you are solving

- If the task is `safety`, use the smallest finite model that can violate the invariant. Three processes and two values are usually enough to find real bugs.
- If the task is `liveness`, add fairness only after the safety model is clean. Treat fairness as an assumption about enabled actions, not as a vague promise that the scheduler will be nice.
- If the task is `operational design`, split the protocol into consensus, leader election, state transfer, and lease logic. Production failures usually hide in those seams, not in the textbook core.

### 2. Build a model TLC can actually check

- Prefer model values and symmetry sets for process identities and values when checking safety. This cuts equivalent states aggressively.
- Start with sets or bounded multisets of messages. Model queues or long sequences only when FIFO order is semantically required; otherwise TLC spends its budget exploring queue growth, not protocol bugs.
- Use state constraints to bound uninteresting growth. Lamport’s own tutorials show message sequences running forever unless you cap them.
- Turn symmetry off for liveness runs unless you have proved it is sound. Safety optimizations often erase the distinctions liveness depends on.

### 3. Add detail in refinement layers, not in one giant spec

- Keep the top spec abstract: chosen value, quorum intersection, and invariants.
- Add implementation detail with refinement mappings and auxiliary variables.
- Use history variables when the lower-level algorithm forgets information the abstract proof needs.
- Use prophecy variables only when an implementation choice depends on a future event; if you add them casually, the spec usually became too concrete too early.
- Use stuttering to align optimized multi-step implementations with coarse abstract steps instead of mutating the abstract spec until it matches the code.

## Paxos-Family Decision Tree

- Need crash-fault consensus with a stable leader and replicated log? Use classic/Multi-Paxos reasoning.
- Need lower steady-state quorum cost or even-sized acceptor sets? Consider Flexible Paxos. Only cross-phase quorum intersection is required; majority quorums in both phases are conservative, not fundamental.
- Need client-to-decision latency in two message delays? Only consider Fast Paxos when conflicting proposals are rare and you can afford the quorum cost. Lamport’s bound is `more than 2e + f` processes to tolerate `f` faults and still decide in two delays despite `e` fast-path faults.
- Need reconfiguration or primary-backup semantics? Use Vertical Paxos reasoning with an explicit configuration master.
- Need lower active hardware cost with spare processors? Cheap Paxos is viable only if the set of healthy processors does not "jump around" too fast while repaired nodes reacquire state.
- Need Byzantine tolerance? Do not bolt signatures onto crash Paxos and call it done. Use a refinement-based Byzantine design with byzquorums and explicit malicious-leader handling.

## Non-Obvious Lamport Moves

- New leaders in Multi-Paxos do not need to serialize log repair before serving new work. A leader can fill holes with no-ops and resume assigning later instances while old gaps are being learned.
- Observation O4 from Lamport’s Paxos variants matters in practice: hashing large values reduces traffic, but at least one process must still be able to recover the full value. A leader that learns only hashes cannot legally send Phase 2a.
- The minimum persistent Paxos state is smaller than most implementations store: an acceptor needs two ballot numbers and one accepted value; a leader needs the largest ballot for which it executed Phase 2a. Extra persistence is an engineering choice, not proof necessity.
- In Chubby, fewer than 1% of instances needed full Paxos. This is why steady-state quorum design and lease behavior dominate performance discussions more than the cold-path proof does.
- Local linearizable reads are not "free reads". They are lease reads. If you cannot state the lease expiry and clock-drift assumption in the spec, the optimization is not justified.
- Vertical Paxos I and II make a real trade-off: one active configuration is easy to reason about, but it ties progress to state transfer; allowing multiple active configurations decouples service from copying large state.
- Vertical Paxos also fixes a nasty leader-failure edge case: without activation discipline, each failed reconfiguration can force the next leader to consult yet another old configuration.
- An external configuration master is not only organizationally neat; it can reduce the processors needed to tolerate `k` failures from `2k + 1` to `k + 1` for the replicated state machine it governs.
- Cheap Paxos trades hardware for an extra liveness assumption, not for safety. That trade is reasonable only when repair and state catch-up are fast relative to fault movement.
- Byzantine refinement changes quorum reasoning. A single reported prior vote is no longer trustworthy; byzquorums and cooperative emulation of leader actions are what preserve safety.

## Fairness and Liveness Traps

- Weak fairness on the wrong action is nearly useless. If a receive action is enabled only intermittently because messages can be lost or delayed, weak fairness may prove nothing.
- Strong fairness on an entire process is usually too blunt. Apply fairness to the exact action or PlusCal label whose repeated enablement matters.
- Do safety without fairness first. Fairness assumptions enlarge the search and can hide simple safety counterexamples under a mountain of liveness machinery.
- When a liveness argument depends on "eventually one leader gets timely responses", say that directly. Omega-style leader stability assumptions are often the actual theorem, not the protocol.

## Reconfiguration and Implementation Heuristics

- Reconfiguration is where English-only Paxos designs usually fail. Group membership, snapshot handles, log truncation, and operator procedures are part of the protocol boundary.
- Snapshot state must record its relation to the replicated log. Chubby used snapshot handles carrying Paxos-specific metadata because "snapshot file plus log" is otherwise not a recoverable state.
- Keep the consensus core as an explicit state machine separate from application code. Chandra et al. changed replica membership state in about an hour because the algorithm was isolated; the tests took days.
- Add runtime consistency checks even after proof work. Replicated checksums and replayable fault injection catch operator error, memory corruption, and implementation drift that the proof never modeled.

## Fallbacks When the First Pass Fails

- If TLC does not terminate, remove liveness, replace sequences with sets or bounded buffers, and shrink constants before adding more hardware or timeout detail.
- If the proof argument needs paragraphs of prose, step back and introduce a refinement layer; long English explanations are usually hiding a missing abstraction boundary.
- If reconfiguration logic dominates the discussion, stop pretending it is "just Paxos" and model the master, snapshot metadata, and activation rule as first-class state.
- If Byzantine support forces ad hoc exceptions into a crash-fault spec, split the abstraction: prove the Byzantine protocol refines a clean crash-fault one instead of mixing both stories in one model.

## Anti-Patterns

- NEVER start with a giant realistic model because it feels faithful. It is seductive because it looks production-like; the consequence is state explosion and no counterexample. Instead start with the smallest model that can falsify the invariant.
- NEVER model the network as FIFO queues unless FIFO is a contractual requirement. Queues feel intuitive, but the consequence is spending TLC on queue permutations and unbounded growth. Instead model messages as sets, bags, or tightly bounded sequences.
- NEVER mix safety and liveness in the first spec because it feels "complete". The consequence is that fairness assumptions mask basic safety bugs and make counterexamples unreadable. Instead freeze safety first, then add the minimum liveness machinery.
- NEVER claim a lease optimization is safe because clocks are "pretty synchronized". That wording is seductive operationally; the consequence is split-brain reads when drift or delayed renewal breaks the hidden bound. Instead specify the bound and prove the read path depends on it.
- NEVER assume majority is the only sensible quorum shape because that is what most textbooks use. The consequence is overpaying in steady state or mis-designing even-sized clusters. Instead reason from the actual intersection requirement for the phase structure you chose.
- NEVER treat reconfiguration as a side feature outside the proof. It is seductive to prove a fixed-membership core and hand-wave membership changes; the consequence is unsafe failover, stuck state transfer, or data loss during recovery. Instead model reconfiguration, snapshot metadata, and activation rules explicitly.
- NEVER translate a crash-fault proof into Byzantine settings by intuition. It feels like "just add signatures"; the consequence is trusting fake prior votes or malicious leaders. Instead use byzquorums and refinement to show the Byzantine protocol still implements the crash-fault abstraction.
- NEVER trust operator procedures that are not automated. Real systems lose data through rollout mistakes more often than through the clean failures in papers. Instead automate the dangerous transitions and make recovery replayable.

## Output Style Under This Skill

- Write invariants, assumptions, and quorum conditions before prose.
- Prefer statements of the form `safe if`, `live when`, `breaks if`.
- If proposing an optimization, state what abstract action it refines.
- If reviewing an implementation, ask for the missing spec boundary: stable storage, leader election, state transfer, lease expiry, or membership change.
