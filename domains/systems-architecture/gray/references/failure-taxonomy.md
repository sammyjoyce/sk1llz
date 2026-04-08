# Failure taxonomy, Heisenbugs, and the architecture of fault tolerance⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​‌‌‌​‌​‍​​‌​​‌​‌‍​​‌​‌‌​‌‍​‌​​‌​​‌‍​​​​‌​​​‍‌‌‌​‌‌​‌⁠‍⁠

Source: Gray. *Why Do Computers Stop and What Can Be Done About It?* Tandem TR 85.7, 1985.

## The data that changed the field

Gray analyzed 166 outages across ~2000 Tandem fault-tolerant systems over 7 months (~1,300 system-years of operation). After excluding infant-mortality failures, 107 outages remained for analysis. The share of each root cause:

| Cause | Share | Notes |
|-------|-------|-------|
| System administration / operations | ~42% | Operator actions, misconfiguration, maintenance errors |
| Software | ~25% | Implied MTBF of **50 years** — very good, yet still 1/4 of all outages |
| Hardware | ~18% | The *minor* contributor in a fault-tolerant system |
| Environment (power, cooling, facilities) | ~14% | Underreported; probably higher |
| Other (vendor, unknown) | ~11% | |

**Raw system MTBF: 7.8 years reported, ~11 years after excluding infant mortality.** This is in an era when conventional mainframes had 1-week MTBFs. The 500× improvement came entirely from fault-tolerant hardware and process-pair software.

**The key counterintuitive finding:** once you've engineered the hardware to fail-fast, the remaining failures are dominated by humans making mistakes and software bugs being soft. *More hardware redundancy does nothing past a point.* More rigorous operational procedures and better fault-tolerant software are where the remaining improvement comes from.

## The Heisenbug / Bohrbug hypothesis

Gray introduced two categories of software bug, named after the physicists:

- **Bohrbug** — like a Bohr atom, solid and permanent. Reproduces reliably on the same input. Easily debugged. Gets caught in QA. Rare in production.
- **Heisenbug** — like Heisenberg's uncertainty principle, disappears when you look at it. Depends on timing, scheduling, memory layout, interrupt ordering, or concurrent state. Restarting the process usually resolves it for that instance.

**Gray's measured ratio in production:** **1 Bohrbug out of 132 software faults** (~0.8%). The other **131 out of 132 were Heisenbugs**.

This ratio is the foundation of fault-tolerant software architecture. It says:

> In production, if a software bug causes a failure, there is a **99.2% chance** that restarting the failed process will make the problem go away for that transaction.

Therefore: your architecture should **fail-fast and retry**, not "detect-and-recover-in-place." The retry strategy works because the bugs are soft.

## Why Heisenbugs are transient (not just "harder to repeat")

The bugs are real, but the conditions that triggered them rarely recur:
- **Timing races** — the instruction that lost the race happened at a specific µs boundary.
- **Interrupt ordering** — a different interrupt arrived first this time.
- **Memory layout** — the stack/heap is in a slightly different state; the uninitialized variable has a different value.
- **Scheduler state** — another thread held a lock for slightly longer.
- **Cache state** — a TLB miss happened at a different instruction.

These factors are effectively random; the probability of identical conditions on retry is near zero. That's why retry works even without "fixing" the bug.

## The fail-fast design principle

The opposite of "fault-tolerant by ignoring errors." A fail-fast module:

1. **Checks inputs** aggressively — preconditions, types, bounds, null-checks.
2. **Checks intermediate results** — invariants between phases.
3. **Checks outputs** — postconditions before returning.
4. **Checks data structures** — on every mutation, verify consistency.
5. On **any** failure, signals failure and **stops**. Does not attempt to continue. Does not attempt to recover in place.

Why this is counterintuitive: engineers instinctively write code that "handles" errors by continuing in a degraded mode. Gray's data shows this is worse than stopping, because continuing with corrupt state produces *permanent* bugs (Bohrbugs) in the persisted state, while stopping lets the retry mechanism turn the Heisenbug into a successful retry.

**The shortest useful summary:** Check everything, and crash on the first inconsistency. Then build a supervisor that restarts the crashed process.

## Process pairs: the implementation of retry-on-crash

A **process pair** is two instances of the same process running on different hardware:
- **Primary** — does the work.
- **Backup** — shadows the primary, ready to take over.

On primary failure (detected by missed heartbeat or explicit fail-fast exit), the backup takes over the work. The takeover mechanism is *not* "rerun from the beginning" — it's "continue from the last checkpoint," which is possible because the primary wrote its progress to a checkpoint file (or the transactional log) synchronously.

Tandem's implementation interleaved checkpoints with regular work so that the backup could resume the specific transaction in flight.

Modern analogs:
- **Erlang supervisors** (let it crash, restart the child)
- **Kubernetes pod replicas with liveness probes**
- **Kafka Streams tasks with state store checkpoints**
- **AWS Lambda retry-on-failure** (at-least-once with idempotency keys)

The unifying idea: **assume the running process will die, and design the recovery to succeed without it.**

## MTBF math: why redundancy has diminishing returns

If a component has MTBF `M` and you run `N` of them in a system that fails if *any* component fails:

    System_MTBF = M / N

More components ⇒ worse. This is why big monolithic systems outlast distributed ones on MTBF, and why every additional node in your cluster is a liability, not an asset, unless you have true redundancy.

With **pair redundancy** (dual components, fast failover, MTTR `R`):

    Pair_MTBF ≈ M² / (2 · R)

If `M = 1 year` and `R = 5 minutes` (fast, automated fail-over):

    Pair_MTBF ≈ 1 year² / (10 minutes) ≈ 52,560 years

The quadratic improvement is why **fast detection and fast repair matter more than adding more nines to the component MTBF.** A 1-year-MTBF component with 5-minute MTTR beats a 10-year-MTBF component with 1-hour MTTR.

**Triple redundancy** adds another factor that is usually negligible — other failures (software, operations) already dominate before triple-redundancy moves the needle.

**Remote replication** is different. It protects against ~75% of failure causes — everything except shared-software bugs, which hit both sites simultaneously. Remote replication is the only thing that pushes a system past 4 nines.

## The "infant mortality" paradox

Gray's data showed:
- **~33% of outages came from infant-mortality failures** (new hardware or software with unshaken bugs).
- **~33% of the remaining outages came from maintenance actions** (the act of fixing something broke something else).
- But *another* study found that many outages came from **known bugs with fixes that had not been installed**.

The paradox: if you update aggressively, you hit infant mortality. If you don't, you hit known-bug outages.

**The resolution** (Gray's rule): *wait for a major release, test it carefully in the target environment, then deploy.* Leading edge, not bleeding edge. Update on a cadence, not continuously, and never update during a production incident.

Modern version: **canary deployments** with explicit monitoring of the canary's error rate compared to the baseline. A fix that introduces a regression should be caught before it hits the full fleet.

## Fault containment via message passing

Gray's process model rejected shared memory for fault-tolerant code. The logic:

- Shared memory means a fault in one process can corrupt data another process depends on.
- Message passing isolates failure: a corrupt message can be detected (checksum, type check) and discarded; a corrupt process cannot reach across an address-space boundary.

This is the intellectual ancestor of:
- Erlang's actor model
- Microservices with explicit APIs
- Unix pipes
- Kafka as a "log you can't corrupt from the consumer side"

**The design implication:** when building a fault-tolerant distributed system, prefer message passing over shared state, even at the cost of some performance. The fault containment pays for itself the first time you have a production bug.

## Compensating transactions

Gray's papers introduced the concept formally in 1981: once a transaction is committed, you **cannot undo** it in the traditional sense — the log records it as committed, and the rollback machinery is gone. The only way to "undo" is to run a new transaction that compensates.

Examples:
- A shipped order: the "undo" is a refund transaction, not a rollback.
- A sent email: the "undo" is a follow-up correcting email.
- A paid invoice: the "undo" is a credit note.

**Implication for distributed systems:** long-running workflows (sagas) decompose into a forward chain of transactions and a compensating chain. The compensation for transaction T_k runs if any later transaction T_{k+1}, ..., T_n fails. Compensations must be **idempotent** (they may run multiple times) and **commutative where possible** (the order of multiple compensations shouldn't matter).

This is where most distributed workflow engines (Temporal, AWS Step Functions, Camunda) get their model from. If your saga has a step with no possible compensation, you do not have a saga — you have a hope.

## The actionable distillation

1. **Fail-fast** — check everything, crash on the first inconsistency.
2. **Retry from the last checkpoint** — because 131/132 software bugs are Heisenbugs.
3. **Message passing** — for fault containment between components.
4. **Primary + backup with fast failover** — MTBF improves quadratically with MTTR reduction.
5. **Remote replication** — the only thing past 4 nines.
6. **Compensating transactions** — for anything committed that may need to be "undone."
7. **Leading edge, not bleeding edge** — update on a cadence, canary everything.
8. **Automate operations** — human error is 42% of outages; the only fix is to remove the human from the loop wherever possible.
