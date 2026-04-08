---
name: tigerbeetle-deterministic-simulation
description: Build or review deterministic simulation harnesses for distributed systems in the TigerBeetle and FoundationDB style. Use when the task mentions DST, VOPR, replayable fault injection, seed-based reproduction, liveness or safety regressions, quorum-core healing, storage or network or clock fault models, targeted subsystem fuzzers, or time-compressed cluster testing.
---

# TigerBeetle Deterministic Simulation

Use this skill when the real problem is "rare interleaving, partial failure, or long-uptime behavior that ordinary tests will never hold still." The job is not to approximate production politely. The job is to force the system through histories production almost never reaches, then replay the exact history until the bug is gone.

## Load Only What Applies

- Before designing liveness checks, read TigerBeetle's `Simulation Testing For Liveness`.
- Before designing seed capture or shrinking, read `Random Fuzzy Thoughts`.
- Before writing a targeted subsystem fuzzer, read `A Tale of Four Fuzzers`.
- Before modeling clocks or physical time, read `Three Clocks are Better than One`.
- Do NOT load broad protocol or storage papers unless the bug crosses that boundary; they add theory faster than signal.

## Before You Touch the Harness, Ask Yourself

- What exact property fails: safety, liveness, recovery, or latency under gray failure? If you cannot name it, you will inject the wrong faults.
- What is the oracle? "It did not crash" is not an oracle.
- Which failures must heal, and which must become permanent, to make the bug observable?
- What must be recorded besides the seed? If generator semantics might drift, seed alone is not a stable testcase.

## Pick The Right Harness

- Use whole-system DST when the bug lives in cross-layer interaction: quorum logic, failover, repair, clock coordination, or durability.
- Use a targeted subsystem fuzzer when whole-system DST can only say "it broke" but cannot tell whether the local algorithm is good. TigerBeetle isolates subsystems behind minimal interfaces and fuzzes them directly.
- Use a non-deterministic outer harness when the suspected gap is outside the deterministic core: real sockets, native bindings, process supervision, or OS integration. TigerBeetle added Vortex for exactly these escape hatches.

## Expert Rules

- Separate safety from liveness. Uniform random chaos is good at safety bugs because it explores bad states. It is bad at liveness bugs because the same chaos often heals the condition before you can prove the cluster is stuck. TigerBeetle's answer was two-phase testing: random chaos first, then liveness mode with a chosen core quorum healed internally while all non-core failures are frozen permanently.
- Force convergence after entropy runs out. A finite entropy budget is better than endless randomness. Once the budget is exhausted, the environment stops inventing new failures and the system must converge. If it does not, you have a liveness bug instead of a noisy test.
- Shrink entropy length, not just the seed value. A practical representation is `{length, seed}`: one half controls how much chaos exists, the other controls which chaos. Binary-searching shorter entropy often minimizes a liveness bug faster than hand-editing event scripts.
- Treat the seed as "reproduce at this commit", not "reproduce forever". Generator changes can preserve the numeric seed while changing the realized scenario. For long-lived regressions, save the derived event trace or minimized fault script with the seed and commit hash.
- Model the boundary that actually decides correctness. Heartbeat RTT is a seductive proxy because it is cheap, but it lies whenever throughput or `fsync` dominates. TigerBeetle's routing work measures end-to-end prepare or ack latency instead of pings because slow disks and skinny links hide behind pretty ping times.
- Inject byte-level storage corruption, not just sector loss. TigerBeetle found bugs by corrupting padding and checkpoint-adjacent bytes that sector-level models missed. If your model corrupts only application payload, you are giving metadata a free pass it does not deserve.
- Make gray failure first-class. Fail-stop crashes are easy. More interesting are asymmetric partitions, slow disks that delay acknowledgements, throttled links, misdirected I/O, and nodes that respond just enough to poison routing or repair.
- Keep interfaces minimal. A large fake world feels realistic but explodes state space and goes stale as the production API changes. TigerBeetle's rule is to cut the target behind the smallest interface that still expresses the bug.
- Keep one intentionally dumb workload. Sophisticated generators are great until they accidentally project away the states you needed. TigerBeetle's query fuzzers missed a real bug because pre-registered query families made intersections too orderly; random model-backed inserts and random queries found it immediately.
- Put seed emission outside the crashing process when possible. A test that segfaults before printing its seed is not reproducible. FoundationDB and Zig-style harnesses pass or report the seed from the parent process so crashes do not erase the breadcrumb.

## Oracles That Actually Catch Bugs

- Safety: check invariants after every state transition, not just at the end. End-of-run checks let transient violations slip away.
- Liveness: define a bounded progress condition such as "the healed core quorum commits within N protocol windows after chaos stops". If you cannot bound progress, you will normalize livelock.
- Recovery: assert that post-crash state is one of the explicitly allowed durability outcomes, not "whatever replay happened to accept".
- Performance-sensitive algorithms: compare the metric the system optimizes in production. TigerBeetle optimizes median quorum-ack latency, not abstract link quality.

## When The Simulator Lies

- Treat every red as either a product bug or an oracle bug. TigerBeetle had to remove VOPR false positives where the protocol was correct and the simulator's understanding was wrong.
- If a failure will not survive trace minimization, suspect the oracle before suspecting the protocol.
- When replay says "impossible", dump the realized event trace and the oracle inputs. Most simulator bugs live in stale assumptions about what states are actually legal.

## Fault Patterns That Pay Rent

- Asymmetric partition plus view-change pressure. This is the classic "can send but cannot hear" poison that passes many failover tests.
- Freeze-and-heal splits. Heal links inside the chosen core and freeze failures outside it to prove the healthy quorum can ignore broken peers.
- Resonance patterns. Bugs often need two reasonable policies to phase-lock. TigerBeetle found one where aggressive parallel repair plus round-robin target selection caused permanent non-progress.
- Swizzle-clogging style network faults. FoundationDB exposed deep bugs by clogging a random subset of nodes one link at a time and then unclogging in shuffled order; simple all-or-nothing partitions were too blunt.
- Byte corruption near padding, manifests, checkpoint metadata, or repair bookkeeping. The bugs are often in the bytes everyone assumes are inert.

## Decision Tree

- If the question is "can the system ever return a wrong answer?" use safety mode with aggressive random faulting and invariants after every transition.
- If the question is "can a healthy quorum get stuck because of unhealthy peers?" use two-phase testing: random chaos first, frozen-failure liveness second.
- If whole-system DST finds failures but diagnosis is mushy, cut a targeted subsystem fuzzer with a smaller interface and a stronger local oracle.
- If DST is green but the shipped binary or client wrappers still fail, add a non-deterministic outer harness instead of contaminating the deterministic core with OS behavior.
- If the code under test cannot yet live inside DST, use lighter-weight fault injection first. FoundationDB's client-side `Buggify` starts with 25 percent activation and 25 percent fire probability, which is cheap coverage for error paths before you earn a full simulator.
- If a seed stops reproducing after a refactor, replay the persisted trace first. If the trace still fails, the bug is real; if only the seed mattered, your generator contract changed.

## NEVER

- NEVER keep healing every fault because it feels more realistic. The seductive part is that eventually-healing chaos produces fewer reds. The consequence is that livelocks disappear when a restart resets counters or a partition heals. Instead switch from chaos to a frozen-failure phase with an explicitly healthy core.
- NEVER use pretty proxy signals like heartbeat RTT when the algorithm is judged by commit latency. The seductive part is that pings are easy to measure. The consequence is route or repair logic that looks optimal in test and collapses under slow disks or thin links. Instead measure the exact end-to-end quantity the production algorithm optimizes.
- NEVER let the workload pre-coordinate its own inputs just because that simplifies checking. The seductive part is a tiny model and easy assertions. The consequence is blind spots: joins never need to probe, repairs never contend, and queues never reorder in the interesting way. Instead keep at least one model-backed workload with arbitrary inputs and exact-result checking.
- NEVER treat a numeric seed as the full regression artifact because it is compact and chat-friendly. The consequence is unreproducible bugs after the generator changes. Instead store seed plus commit hash, and persist the realized minimized trace for bugs you care about.
- NEVER bring host threads, blocking calls, or real scheduler timing into the deterministic path because it is convenient for reusing production code. The consequence is deadlocks or scheduler-dependent failures that masquerade as simulator issues. Instead keep the deterministic core single-threaded, or mark the escape hatch as non-deterministic and test it separately.
- NEVER stop at crash or partition faults because they are the easiest failures to imagine. The consequence is missing the bugs hiding in gray failure, clock drift, padding corruption, torn writes, and misdirected I/O. Instead fault the exact contracts your system claims to survive.
- NEVER chase an arbitrary ticks-per-second number because it feels quantitative. The consequence is a fast but toothless model. Instead optimize for bug-finding state transitions per CPU and for stronger oracles.

## Freedom Calibration

- Be rigid about determinism boundaries, seed capture, oracle definition, and safety-versus-liveness separation.
- Be flexible about exact fault rates, shrinker mechanics, and scenario scheduling. Those are tuning knobs, not doctrine.
- If a simplification makes the harness smaller but reduces reachable state space, reject it unless you can name the bug classes you are giving up.
