---
name: mendez-async-api
description: "Design event-driven architectures using Fran Méndez's AsyncAPI philosophy. Treats message contracts with REST-grade rigor: application-centric specs, decoupled producers/consumers, and bindings as the truth layer between spec and broker. Use when building Kafka/AMQP/MQTT/NATS systems, writing or auditing AsyncAPI v2/v3 documents, migrating v2→v3, debating thin vs fat events, designing schema evolution strategies, or integrating CloudEvents with AsyncAPI. Trigger keywords: AsyncAPI, async spec, event-driven API, message contract, channel design, event schema, pub/sub API, message broker contract, event-first, CloudEvents, schema registry, Kafka topic contract, AMQP exchange spec, MQTT binding, event carried state transfer, upcasting, schema evolution, Confluent compatibility."
---

# Méndez AsyncAPI Philosophy

AsyncAPI is not OpenAPI-for-brokers. Messaging is many-to-many, consumers may reach the app through a *different* protocol than producers, and there is no request-time handshake to catch schema drift. Design with those facts first.

## Before Writing Anything, Answer These

1. **Whose perspective is this document?** An AsyncAPI doc describes what **your** application sends and receives — not what others can do to you. One spec per application, not one shared spec per topic. (Fran's own correction of the v2 model.)
2. **Can the consumer even reach this channel?** Producer publishes to Kafka; consumer may be on WebSocket because a forwarder republished the event. Never assume "subscriber flips publisher." For public/partner consumers, hand-craft their spec.
3. **Is the spec the source of truth, or decoration?** If nothing breaks when the spec drifts, it will drift. Wire it to code-gen, runtime validation (Glee/Spectral), and mock generation (Microcks). Make the build fail on drift.
4. **Which layer owns protocol specifics?** Payload schemas live in `components/messages`; broker details live in `bindings`. If a broker name or partition count leaks into payload, you have coupled forever.
5. **What's the replay story?** Before picking a compatibility mode, answer: "Do I need to rewind consumers to offset 0 someday?" That single question flips the default from `FULL_TRANSITIVE` to `BACKWARD`.

## The v3 Perspective Shift (the fix for v2's biggest mistake)

In v2, `publish`/`subscribe` described what *clients* could do to the server — inverted from the messaging reality where everyone is a client and the broker is an intermediary. v3 replaces them with `action: send` / `action: receive` from the **app's own** point of view.

| Artifact | v2 mental model | v3 mental model |
|---|---|---|
| `publish` | "Clients can publish to me" | Gone. Use `action: send` when your app sends. |
| `subscribe` | "Clients can subscribe to me" | Gone. Use `action: receive` when your app receives. |
| Channel key | Often conflated with address | Arbitrary local identifier — set `address` separately |
| Channel reuse | Impossible | Messages live on channels, operations `$ref` them |
| Request-reply | Not expressible | First-class via `reply` object; reply topic may be generated at runtime |
| Parameters | Static only in most tools | v3 supports `location: $message.payload#/field` for runtime extraction |

**Only migrate v2→v3 when you need request-reply, channel reuse, or per-app specs.** Fran's own advice: v2 is supported for years; don't migrate for aesthetics.

## Schema Evolution — Where Event Systems Actually Die

Producers and consumers deploy independently. There is no handshake. Old events may be replayed weeks later. Assume every "safe" change is unsafe until proven otherwise.

**The non-obvious rules every practitioner learns the hard way:**

- **`BACKWARD` is Confluent's default for a reason, not `FULL_TRANSITIVE`.** `FULL_TRANSITIVE` feels safer but *prevents* rewinding consumers to offset 0 because you cannot guarantee compatibility with schema v1 after a few evolutions. If you ever need to reprocess the full topic, you want `BACKWARD` (or `BACKWARD_TRANSITIVE`). Pick deliberately.
- **Kafka Streams inverts the upgrade order.** A normal consumer with `BACKWARD`: upgrade consumer first, then producer. Kafka Streams reads from its own changelog (old schema), so **Streams app must be upgraded first**, then the upstream producer. Forget this and Streams crashes on its own state store.
- **"Add an optional field" is only universally safe in Avro/Protobuf lenient JSON Schema.** In **strict JSON Schema** policy (Confluent), adding an optional field is "closed" for backward and "open" for forward — partial, not full. A strict consumer with `additionalProperties: false` will reject the new event. This is the real "2 AM pager" incident.
- **Widening a scalar is FULL in Protobuf, only BACKWARD in Avro.** `int32 → int64` works transparently in Protobuf but breaks forward compatibility in Avro.
- **Adding a required field is never backward compatible, in any format.** Not even with a default in Avro — defaults only help the *reader*, not the writer. A producer emitting without the field blows up the new consumer.
- **Rename is always a full break** in every format. Use "add new field + deprecate old + dual-write + drain + remove" — a 4-deploy dance, not one PR.

For the full compatibility matrix per format, upcasting patterns for event-sourced stores, and the exact Confluent REST calls, read `references/schema-evolution-matrix.md` **before proposing any schema change**.

## Anti-Patterns (with the seduction and the cost)

**NEVER** type payloads as `Map<String, Object>` / `interface{}` / `Record<string, unknown>`. It's seductive because it skips design friction, but it makes the spec worthless: no validation, no useful code-gen, no evolution path, and tooling like Microcks cannot generate mocks. **Instead**: define every field in `components/schemas`, even for "obvious" fields. The pain is upfront; the value compounds.

**NEVER** create a single `global-events` topic. It feels efficient (one topic, one connection), but every consumer must parse every event type, partition keys become meaningless, consumer groups cannot scale independently, and access control is all-or-nothing. **Instead**: one channel per bounded-context event, hierarchical names (`orders.placed`, `orders.shipped`), one schema per channel.

**NEVER** put `correlationId`, `timestamp`, `source`, `traceparent`, or `eventVersion` in the payload body. It leaks infrastructure concerns into business data, breaks CloudEvents interop (which expects these in the envelope), and blinds observability tooling that reads headers. **Instead**: use message `headers` / `correlationId` / CloudEvents context attributes. Payload is pure business data.

**NEVER** version by duplicating topic names (`user-created-v1`, `user-created-v2`). It seems pragmatic, but replay becomes impossible (which version is the truth?), producers must dual-write (with partial-failure risk), and consumers must subscribe to both. **Instead**: evolve the schema additively under one topic, or use a header-based version discriminator with upcasting on the read side.

**NEVER** set `retain: true` on MQTT for transaction-like events. It's tempting because "then new subscribers get the last state" — but the broker will replay that "last event" to every new subscriber as if it were fresh, triggering duplicate charges, duplicate emails, or duplicate orders. **Instead**: reserve `retain: true` for **last-known-value** patterns only (device online/offline, sensor readings); never for domain events.

**NEVER** define queues (AMQP) or consumer groups (Kafka) in the spec. These are consumer-side implementation details. A second consumer team cannot use the spec if you hard-code your queue name. **Instead**: spec the exchange/topic and routing key; leave `queue` / `groupId` out or mark them as examples only.

**NEVER** omit `bindingVersion`. Parsers silently accept any version and interpret fields by the default — which changes between releases. Kafka binding is at `0.5.0`; AMQP at `0.3.0`; MQTT at `0.2.0`. Pin them explicitly.

## Thin vs Fat Events — The Real Decision Tree

This is not "thin is better." It is a coupling trade.

```
Is the consumer in a different bounded context?
├── YES → Is the consumer allowed to call back to the publisher's API?
│         ├── YES, low volume → Thin event (event notification)
│         ├── NO, or high fan-out → Fat event, but scoped to the AGGREGATE
│         │                         (never to entities outside the aggregate)
│         └── Needs point-in-time snapshot (billing, audit) → Fat, include sequence/version
└── NO  → Thin event (same DB often available; avoid payload coupling)
```

**The aggregate-vs-entity dilemma nobody warns you about**: embedding entity-level data explodes event count and leaks DDD encapsulation; embedding aggregate-level data overexposes fields consumers didn't ask for and creates implicit coupling the moment a consumer reads a field you didn't intend to publish. When in doubt: scope to the aggregate boundary **and** add `x-visibility: internal` comments on fields consumers shouldn't rely on — not enforced, but a clear social signal during review.

## AsyncAPI + CloudEvents: When Each Wins

| Situation | Use |
|---|---|
| Internal microservices, same team, same broker | AsyncAPI alone; define your own headers |
| Cross-platform / serverless / FaaS | AsyncAPI **+** CloudEvents envelope |
| Multi-vendor event mesh (Knative, Kafka, EventBridge in one flow) | CloudEvents is mandatory; AsyncAPI documents the topology |
| You need replay across brokers | CloudEvents (the envelope survives broker hops) |

**Structured mode** (whole event is CloudEvents JSON): `allOf` a `$ref` to the CloudEvents schema and your `data` schema. **Binary mode** (CloudEvents attributes in headers, payload is your schema): define a reusable `messageTrait` with the CloudEvents headers and `$ref` it from every message. Never mix both in one channel.

## Bindings Are Where Specs Become Real

Bindings are the layer most teams get wrong because the spec *validates* without them — but the **runtime** silently does the wrong thing. For the exact field cheat-sheet per protocol (Kafka, AMQP 0-9-1 vs 1.0, MQTT vs MQTT5, WebSocket, NATS, SQS, Pulsar), load `references/binding-cookbook.md`. Do not improvise bindings from memory; field names and versions drift between binding releases.

## Fallbacks

| Problem | Fallback |
|---|---|
| Tool doesn't support your protocol binding | Write the spec anyway; use bindings as documentation; validate manually |
| Code generator produces broken stubs | Spec → Microcks mocks + hand-written integration code |
| Team resists spec-first | Start code-first (SpringWolf for Spring, Saunter for .NET, AsyncAPI for Node) — generate spec from annotations, then flip |
| No schema registry available | Embed `schemaVersion` header; consumers switch on version with upcasters |
| Producer/consumer schema disagree in prod | Deploy a **translator** service in between that upcasts on the fly while teams fix their code |

## When NOT to Load the References

- Simple question about v3 `action` vs v2 `publish`? Stay in this file.
- Writing a schema change? **Load `references/schema-evolution-matrix.md` first.**
- Writing any binding block? **Load `references/binding-cookbook.md` first.**
- Migrating from v2 to v3? **Load `references/v3-migration.md` first.**
- Just debating thin vs fat? Stay in this file — the tree above is sufficient.
