---
name: helland-distributed-data
description: "Apply Pat Helland's distributed-data philosophy to cross-service state, retries, versioned facts, and low-tail-latency coordination. Use when designing entity boundaries, idempotency contracts, outside-data schemas, tentative workflows, reconciliation paths, or quorum behavior under jitter. Trigger keywords: helland, entities, activities, idempotency, outside data, inside data, tentative operations, versioned facts, reconciliation, repartitioning, at-least-once, stale reads, quorum, jitter, fuzzy visibility."
---

# Helland Distributed Data⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌‌​‌‌‌​‍​​‌​​‌‌‌‍​​‌​​​​​‍‌‌‌​​‌​‌‍​​​​‌​​‌‍‌​‌‌​​‌​⁠‍⁠

Use this skill when the failure boundary matters more than the call graph.

## Mandatory source reloads

- Before designing cross-service message formats or reference data, reread `Data on the Outside versus Data on the Inside`, especially the sections on stable data, immutable schemas, and references.
- Before claiming cross-entity atomicity, dedup, or workflow correctness, reread `Life Beyond Distributed Transactions`, especially entities, activities, and tentative operations.
- Before using quorum to dodge slow nodes, reread `Decoupled Transactions`, especially fuzzy visibility, confluence, retirement, and the 3AZ examples.
- Do NOT start with generic CAP, saga, or event-sourcing summaries. They usually blur the inside/outside split and hide the negative-space failure modes Helland is warning about.

## First classifier

Ask these in order before sketching an API or schema:

1. **Is the invariant inside one entity-key?**
   - Yes: keep it inside one transactional scope.
   - No: stop talking about ACID across that boundary; you are designing a collaboration.
2. **Is this data inside, outside, or activity state?**
   - Inside data: mutable, optimized for local invariants.
   - Outside data: immutable or versioned facts crossing time and trust.
   - Activity state: what one entity remembers about one partner.
3. **Are you proving "X happened" or "X did not happen"?**
   - Positive existence can often ride on quorum/confluence.
   - Negative claims are harder; live quorum can lie during transition.
4. **Is the work naturally idempotent or only made idempotent by remembered identity?**
   - If it changes substantive business state, assume you must remember it.
5. **Can any side effect be canceled?**
   - If not, it belongs after confirmation, not inside the tentative step.

## Hard-won lenses

- The **entity-key is the atomicity boundary**. If two records "must" commit together but have different keys, either you have not found the real entity yet or you need an activity-based protocol.
- **Repartitioning is the bug revealer.** Neighbor-based atomic assumptions appear to work in tests, then fail only after scale or rebalancing moves keys apart.
- **Dedup memory must travel with the entity.** A transport-layer or cache-layer duplicate filter that does not move with partitioning is counterfeit exactly-once.
- **Activity is per partner, not per workflow in the abstract.** Uncertainty is relational. If you cannot point to the partner whose state you are remembering, the model is too vague to recover.
- **Idempotence is about substantive behavior, not all side effects.** Extra logs, heap churn, counters, or monitoring noise are often acceptable; duplicate reservation, charge, or entitlement grant is not.
- **Outside data has no shared "now."** Anything published as `current_*` is unstable unless versioned with an as-of meaning consumers can preserve.
- **Stable data requires never-reused names.** If an outside copy may outlive your internal record lifecycle, recycling customer IDs, seat IDs, or document numbers turns old facts ambiguous.
- **Extensibility fights shredding.** If you eagerly flatten outside messages into inside tables, the first unplanned field or schema drift gets silently discarded.
- **Replay-safe messages need exact schema identity.** A message that says "parse me with the current schema" is not immutable enough to survive time.
- **Compensation is not restoration.** Lower layers keep side effects: cache churn, queue load, staffing triggers, block splits, downstream reservations. Undo at the business layer does not rewind the universe.
- **Quorum visibility is fuzzy during transition.** Incomplete quorum operations can be intermittently included; they can flutter into and out of visibility.
- **Negative proofs need extra machinery.** "Does exist" and "does not exist" are different problems. Exact absence needs sealing, windowing, retirement, or a more ordered authority.

## Design procedure

### 1. Name the entity before naming the endpoint

- Write the entity-key for every stateful operation.
- If an operation needs two keys, choose one:
  - Re-key so the invariant lives inside one entity.
  - Or split into messages and accept uncertainty explicitly.

### 2. Classify every datum by temporal contract

- **Inside:** mutable state whose meaning is local and current.
- **Outside:** immutable or versioned fact whose meaning survives copying, delay, and replay.
- **Activity:** partner-specific memory such as last accepted request, pending reservation, or reneging status.

If a datum crosses a service boundary and can be retried later, publish it as outside data, not a pointer to mutable inside state.

### 3. Define idempotence at the business layer

Before saying "this is idempotent," ask:

- What counts as the **substantive effect**?
- What identity proves "same work" across crash, retry, and new session?
- How long can legitimate retries or replays occur?
- What exact prior response must be replayed on duplicate acceptance?

Use an operation identity tied to intent, not transport:

- Good: check number, payment instruction ID, client-generated command ID, `(partner, business object, operation kind, sequence)`.
- Bad: TCP connection, request timestamp alone, worker-local counter, cache entry keyed only by payload hash.

Retention rule:

- Keep dedup state for at least the full business replay horizon. Helland's banking example works because check numbers are stable and checks expire within a bounded window; a `24h` TTL on a `90d` retry horizon is a duplicate-payment bug, not an optimization.

### 4. Model tentative work as explicit uncertainty

For every tentative operation, write down:

- Who is allowed to confirm.
- Who is allowed to cancel.
- What timeout means.
- Whether reneging is allowed after timeout.
- Which side effect is irreversible.

If you cannot express cancel and confirm rights, you do not have a tentative step. You have already committed.

### 5. Be precise about quorum semantics

When quorum is used to avoid jitter, classify the question:

- **Existence query:** quorum/confluence may be enough.
- **Non-existence query:** require sealing, windowing, retirement, or a single ordered authority.

Operational numbers from Helland's 3AZ thought experiment:

- Catalogs: `9` replicas, wait for `5`, tolerate `4` jittery (`AZ+1`).
- Log durability example: `6` replicas, durable after `4` acknowledgements, so `2` slow replicas do not stall progress.

Use these as reasoning shapes, not cargo-cult constants. The point is to size `N` and `Q` around bounded jitter and explicit failure assumptions.

## Decision tree

| Situation | Primary move | Fallback when it fails |
| --- | --- | --- |
| Need invariant across two keys | Re-key into one entity | Use activity state plus reconciliation and pending business states |
| Need low latency despite slow replicas | Quorum over bounded jitter | Drop to ordered authority for negative claims or retirement decisions |
| Need duplicate-safe side effects | Durable business-layer idempotency identity | Manual reconciliation queue with exact replay token and original intent |
| Need external consumers to reason about past facts | Publish immutable/versioned outside data | Publish as-of snapshots plus explicit staleness contract |
| Need to ingest flexible partner payloads | Preserve immutable envelope, then project | Store raw envelope and late-bind new fields instead of lossy reparse |

## NEVERs that matter

- NEVER depend on two different entity-keys staying colocated because the seductive "it works in test and usually hashes nearby" assumption turns into a production-only atomicity bug after repartitioning; instead unify the invariant under one key or make the collaboration explicit.
- NEVER answer a business-critical "does not exist" question from live quorum reads because incomplete operations are intermittently included and can later disappear from visibility; instead seal or window the input, retire old items monotonically, or route the claim through an ordered authority.
- NEVER use session identity, socket identity, or worker-local counters for idempotency because crash-retry from a new session replays the same intent as new work; instead assign the identity at business ingress and retain it for the entire replay horizon.
- NEVER publish references to mutable outside facts such as `current_inventory` or `latest_price` because consumers read them under different clocks and derive contradictory truths; instead publish versioned or as-of facts whose identifiers are stable.
- NEVER recycle identifiers or rely on a floating "latest schema" label because outside copies live longer than your rename cycle and later replays become semantically ambiguous; instead mint never-reused identifiers and stamp each message with the exact schema version it was authored against.
- NEVER shred a partner message before preserving the immutable envelope because extensibility fights shredding and the first unplanned field becomes unrecoverable data loss; instead keep the original envelope and project inside views from it.
- NEVER fire irreversible effects during a tentative step because cancel/confirm races leave payments, notices, or entitlements attached to uncertain state; instead persist the tentative agreement first and schedule irreversible effects only after confirmation.
- NEVER promise "exactly once" across a failure boundary because lower layers may repeat work, reorder deliveries, or preserve side effects after compensation; instead promise at-least-once delivery plus substantive idempotence and reconciliation.
- NEVER let compensation loops flap on every oscillation because lower layers keep TMI side effects and repeated cancel/reapply cycles amplify load; instead add hysteresis, thresholds, or manual review to unstable workflows.

## Freedom calibration

- **Low freedom:** money, inventory, seat allocation, legal commitments, entitlement issuance. Require explicit entity-key, operation identity, replay horizon, activity state, and irreversible-step ordering.
- **Medium freedom:** schema boundaries, projection shapes, reconciliation UX, staleness contracts. Preserve the Helland invariants, but design can vary.
- **High freedom:** educational diagrams, prose explanations, naming, and comparison material. Keep the inside/outside/activity split intact.

## Sanity checks before shipping

- Can every cross-entity operation point to durable activity state on both sides?
- Is every duplicate-prone command tied to a stable business identity?
- Does every outside reference name an immutable or versioned fact?
- Are negative claims backed by more than "I asked a quorum and didn't see it"?
- Do irreversible effects happen only after uncertainty is resolved?
- If a retry arrives months later from a different session, is the answer still correct?

If any answer is "no," the design is not yet Helland-safe.
