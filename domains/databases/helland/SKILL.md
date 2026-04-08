---
name: helland-distributed-data
description: >-
  Design scalable data systems applying Pat Helland's patterns from "Life Beyond
  Distributed Transactions," "Data on the Outside vs Inside," "Building on
  Quicksand," and "Immutability Changes Everything." Use when building systems
  that must scale beyond single-node ACID, designing entity boundaries for
  partitioning, implementing idempotency and deduplication, choosing between
  sagas and eventual completion, or deciding what data crosses service
  boundaries. Triggers: distributed transactions, entity partitioning, cross-
  service consistency, idempotent operations, outbox pattern, saga compensation,
  inside vs outside data, at-least-once messaging, scale-agnostic design.
---

# Pat Helland — Distributed Data Design

## Thinking Framework

Before designing any cross-entity interaction, ask yourself:

1. **"What is the entity?"** — The entity is whatever fits in one transactional scope. If two things must update atomically, they are ONE entity, not two. Stop fighting this.
2. **"What if this message arrives twice? Three times? After a week?"** — If you can't answer, you haven't finished designing.
3. **"Is this data inside or outside?"** — Inside data is mutable, rich, query-able. Outside data is immutable, versioned, simple. Mixing them up is the #1 source of distributed bugs.
4. **"Am I managing uncertainty in the business logic or hiding it?"** — If hiding it, you're building a fragile system. Uncertainty (tentative holds, pending reservations, allocation against credit) must be explicit in your domain model.

## Decision Tree: How to Handle Cross-Entity Operations

```
Do both things NEED atomic consistency?
├─ YES → They are one entity. Merge them. Stop here.
└─ NO → Can the operation be naturally idempotent (read-only, or set-to-value)?
    ├─ YES → No dedup infrastructure needed. Use it.
    └─ NO → Is this a business-level decision that can be reversed?
        ├─ YES → Saga with compensating actions (business errors only)
        └─ NO → Eventual completion with retry + idempotency keys
            └─ NEVER use saga compensation for technical failures.
                Technical errors need retry-to-completion, not rollback.
```

## Expert Knowledge: What Takes Years to Learn

### Entity Sizing — The Hard Part Nobody Talks About

- Entities grow in **number**, not in **size**. Helland's key observation: almost-infinite scaling increases entity count while individual entities stay small enough for one machine.
- **Alternate indices cannot be transactionally consistent** at scale. If you reference a customer by SSN, credit card number, AND email, those indices will live on different machines. Accept eventual consistency for lookups or unify under one canonical key.
- Two objects that need transactional coupling must share a key prefix. If you're assigning them different keys but wrapping them in one transaction "just in case" — you've created a scaling time bomb that works today and explodes at 10x load.

### Activities — The Concept Most Teams Miss

Helland's **activities** are per-partner state within an entity. Each entity tracks what it knows about each partner entity (messages received, messages sent, current protocol state). This is NOT just a dedup table — it's a fine-grained workflow state machine per relationship.

- Structure idempotency state per-partner, not as a global flat table. At scale, a single `processed_requests` table becomes a bottleneck. Partition dedup state by the *partner entity key*.
- Activities encode the protocol: "I sent an offer, they accepted, I confirmed." This makes debugging distributed interactions possible because each entity holds its complete view of each relationship.

### Idempotency — Where Teams Actually Fail

- **The dedup-check-then-process race condition**: Two identical messages arrive simultaneously. Both check the dedup table, both find nothing, both process. Fix: the dedup record and the business effect MUST be in the same atomic transaction. Not "check then do" — "do with unique constraint".
- **Idempotency keys in Redis with TTL will lose guarantees**. The second Redis evicts the key or a node fails, you lose dedup state. For anything financial, idempotency belongs in the source-of-truth database with a unique constraint, not in a cache.
- **Return the stored result, not just "already processed"**. The original caller may not have received the first response. Returning the same result on retry is what makes the operation truly idempotent from the caller's perspective.
- **Natural idempotency is underused**. Helland distinguishes "naturally idempotent" (reads, set-to-value) from "substantively changing" messages. Before building dedup infrastructure, ask if you can restructure the operation to be naturally idempotent (e.g., "set balance to X" instead of "decrement by Y").

### Sagas — The Critical Distinction Teams Get Wrong

Sagas handle **business errors** (credit limit exceeded, inventory unavailable). They CANNOT handle **technical errors** (database timeout, network partition, service crash).

- When a compensating action fails due to a technical error, you cannot "compensate the compensation." This is an infinite regress. Technical errors in distributed systems are non-deterministic — a one-shot deterministic approach like saga compensation will eventually leave you with corrupted data.
- The correct architecture: build a **reliable technical layer** that retries operations to eventual completion, then run sagas on TOP of that layer for business-level rollback.
- **If you need sagas everywhere, your service boundaries are wrong.** Pervasive saga usage is a design smell. Services organized around entities (Order Service, Customer Service, Inventory Service) inherently require cross-service transactions. Reorganize around use cases instead.
- Non-compensatable operations exist: sent emails, printed documents, fired missiles. Design your saga step ordering so non-compensatable steps execute LAST, after all fallible steps.

### Inside vs Outside Data — Practical Consequences

- **Outside data lives in the past**. The moment data leaves a service boundary, it's a snapshot from a prior point in time. Design for staleness: include version identifiers, timestamps, and explicit "as-of" semantics.
- **"Anything called 'current' is not stable"** (Helland's rule). `current_inventory`, `current_price` — these are not safe to pass across boundaries. Pass `inventory_as_of_2024_03_15T10:00:00Z` instead.
- **Don't recycle identifiers**. `customer_id = 42` must mean the same customer forever. Reusing IDs after deletion creates semantic corruption across every system that cached that reference.
- Outside data forms a DAG (Directed Acyclic Graph) of immutable items generated independently by different services. New data references old data but never mutates it. Schema must travel with the data (self-describing messages).

### Immutability — The Operational Insight

Helland: "The truth is the log. The database is a cache of a subset of the log."

- **Immutable data eliminates eventual-consistency anomalies in distributed storage.** When storing immutable blocks in a consistent-hashing ring, you cannot get stale versions — each block has the only version it will ever have. This is why append-only designs tolerate weaker storage guarantees.
- **You can know WHERE you are writing or WHEN the write will complete, but not both.** By preallocating immutable file IDs from a strongly consistent catalog, writes to weakly consistent storage become predictable.
- **Immutability enables idempotent retry of computation**, not just storage. MapReduce, Dryad, and modern batch jobs work because functional computation over immutable inputs is inherently idempotent. Apply this to your own data pipelines.
- Dark side: denormalization consumes storage, copy-on-write layers compound, and you must still solve garbage collection ("DOES NOT EXIST" queries require knowing all references are gone — a coordination problem immutability doesn't solve).

## NEVER

- **NEVER use saga compensation to recover from technical errors** — it will eventually corrupt your data. Technical errors need retry-to-completion on a reliable lower layer. Sagas are for business rule violations only.
- **NEVER store idempotency keys only in a cache (Redis/Memcached)** — TTL expiry or node failure silently destroys your exactly-once guarantees. Use a durable store with a unique constraint.
- **NEVER design two entities that must coordinate for basic operations** — if they must be atomic, they are one entity. If you're adding distributed locking between entities, you're rebuilding 2PC without admitting it.
- **NEVER pass mutable or "current" references across service boundaries** — outside data must be immutable and timestamped. "Current price" is meaningless to a receiver who processes it 200ms later.
- **NEVER assume exactly-once message delivery exists** — at-least-once with idempotent processing is the only practical guarantee. Building on exactly-once is building on quicksand.
- **NEVER implement check-then-act idempotency outside a transaction** — the race window between "check if processed" and "mark as processed" is exactly where duplicate processing hides.

## Fallbacks When Things Go Wrong

| Situation | Primary approach | Fallback |
|---|---|---|
| Saga compensation fails | Retry compensation with backoff | Escalate to dead-letter + human review queue |
| Outbox publisher falls behind | Increase polling frequency / batch size | Switch to CDC (Change Data Capture) from the outbox table |
| Entity too large for one node | Identify sub-entities with independent lifecycle | Split into parent + child entities connected by messages |
| Idempotency table growing unbounded | TTL-based cleanup of old entries | Archive to cold storage; keep last N days in hot path |
| Stale reference data causing bad decisions | Increase refresh frequency | Add version-check callback before committing ("is offer still valid?") |

## Key Helland Papers (Read These)

- **"Life Beyond Distributed Transactions"** (2007/2016) — entities, activities, idempotency
- **"Data on the Outside vs Data on the Inside"** (2005) — inside/outside, temporal domains, stable data
- **"Building on Quicksand"** (2009) — memories, guesses, apologies; accept uncertainty
- **"Immutability Changes Everything"** (2015) — append-only at every layer
- **"Decoupled Transactions"** (2022) — quorum-based snapshot isolation on jittery servers
