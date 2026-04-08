---
name: mendez-async-api
description: "Apply Fran Mendez's application-centric AsyncAPI style to event-driven APIs. Use when designing or reviewing AsyncAPI v2/v3 contracts, choosing schema-evolution rules, modeling request-reply over brokers, deciding thin vs fat events, scoping bindings, or deciding whether CloudEvents belongs in the envelope. Trigger keywords: AsyncAPI, event contract, send receive, request-reply, channel address, Kafka binding, MQTT retain, schema registry compatibility, CloudEvents, event versioning, schema evolution."
---

# Mendez AsyncAPI Philosophy

AsyncAPI is where event systems decide which coupling becomes permanent. Use this skill to make the long-lived contract decisions correctly, not to relearn syntax.

## Operating Mode

- This is a philosophy skill.
- Use high freedom for event boundaries, consumer experience, and spec decomposition.
- Use low freedom for schema evolution, bindings, request-reply wiring, and v2 to v3 migration. Those are contract-surgery tasks; small mistakes survive for years.

## Load Boundaries

- Before proposing any schema change, READ `references/schema-evolution-matrix.md`.
- Before writing any binding block, READ `references/binding-cookbook.md`.
- Before migrating v2 to v3, READ `references/v3-migration.md`.
- Do NOT load those references for naming debates, thin-vs-fat decisions, or basic `action: send` versus `action: receive` questions.

## Review Order

1. **Perspective**: Is this document from one application's point of view, or did you accidentally describe the broker?
2. **Failure story**: Must this contract survive replay, partial rollout, partner consumption, broker hops, or offline audit?
3. **Payload boundary**: Which fields are business facts, and which fields are delivery scaffolding that belong in headers or bindings?
4. **Consumer ergonomics**: Can an external consumer act without a synchronous callback to the publisher?
5. **Transport truth**: Do `address`, `servers`, and bindings match the real wire behavior?
6. **Drift enforcement**: If nothing breaks when the spec drifts, the spec is decoration. Wire parser, mocks, and compatibility checks into CI.

## Before Editing, Ask Yourself

- Are you documenting one application, or are you drifting toward a shared "platform spec" that no real consumer can implement?
- What must this event survive: cold replay, a month of schema drift, a partner team with a different protocol, or cross-broker forwarding?
- If a new consumer appeared tomorrow with no database access, would this event still be actionable?
- If a field disappeared tomorrow, would anyone page you? If yes, it is already public contract, even if you hoped it was "internal."
- Who should pay the coupling cost: the publisher once, or every consumer forever? In fan-out systems, prefer publisher cost.

## High-Leverage Heuristics

- One application, one spec. If a partner cannot reach your broker or needs HTTP/WebSocket instead of Kafka, publish a second consumer-facing contract. Do not leak internal topology into external docs.
- The channel key is local handle; `address` is wire truth. Renaming the key should be cheap. Renaming the address is a migration.
- In multi-protocol specs, channel availability is global unless you scope `channels.*.servers`. If you omit that in mixed Kafka/MQTT/WS systems, generators assume the channel exists everywhere.
- Migrate v2 to v3 only for real wins: request-reply, channel reuse, or application-centric clarity. Do not pay migration cost just to replace keywords.
- When the reply destination is runtime-defined, keep the reply channel `address` null or omitted and use `reply.address.location`. Hard-coding temporary inboxes into the document defeats request isolation.
- AsyncAPI describes topology and contract shape. CloudEvents is the envelope choice when events cross transports, clouds, or generic routers. AsyncAPI alone is enough for single-team, single-broker systems.

## Event-Shape Tree

- Same bounded context: thin notification.
- Different bounded context, cheap callback allowed: thin event.
- Different bounded context, callback forbidden or fan-out high: fat aggregate snapshot.
- Audit, billing, or rebuild requires point-in-time truth: fat snapshot plus aggregate version or sequence.

Rules:

- Fat means aggregate snapshot, not entity graph dump.
- The moment consumers depend on an "extra" field, it is public contract.
- If payload size approaches Kafka's default `max.message.bytes` of `1048588`, you are tunneling a document, not publishing an event. Send a durable object reference plus integrity hash.

## Compatibility Heuristics

- Default Kafka subjects to `BACKWARD` when rewind to offset 0 matters. Confluent makes that the default for that reason.
- Use `BACKWARD_TRANSITIVE` for long-lived topics and as the safer default for Protobuf subjects. Confluent explicitly recommends it for Protobuf because adding new message types is not forward compatible.
- `FULL_TRANSITIVE` feels safest because upgrades look independent, but it couples you to every historical schema. Use it only when full-history independent rollout matters more than replay flexibility.
- Non-transitive checks compare only with `latest`. If retention or replay spans many versions, test against every retained version, not just the current head schema.
- Kafka Streams is special: upgrade the Streams app before upstream producers, because Streams must read old changelog state and new input at the same time.
- JSON Schema "add optional field" is only safe when unknown fields are tolerated. Strict readers with `additionalProperties: false` turn that "safe" change into a live incident.
- Renames are not schema evolution. They are staged migrations. If you cannot afford a dual-read and dual-write rollout, create a new subject or topic.

## CloudEvents Choice

- Use structured mode when events will be archived, forwarded across brokers, or passed through systems that may not preserve headers reliably. The envelope survives intact.
- Use binary mode only when you must preserve the raw business payload shape and you control header preservation end to end.
- Never mix structured and binary forms on one logical channel without an explicit contract split. Consumers end up branching on transport trivia instead of business meaning.
- Keep tracing, correlation, source, and version in envelope or headers. Payload is for domain facts.

## NEVER List

- NEVER publish schemaless payloads because skipping design friction feels fast, but you lose compatibility automation, mock generation, and safe evolution. Instead model every field explicitly.
- NEVER create a `global-events` channel because one topic feels operationally tidy, but ACLs, partition keys, retention, and consumer scaling all become coupled. Instead publish one channel per bounded-context event family.
- NEVER version with topic suffixes for routine evolution because rollback feels easy, but replay splits across topics and dual-write failures create contradictory truth. Instead evolve the schema in place; create a new topic only for semantic breaks.
- NEVER put transport metadata in payload because "everyone can read it there" feels convenient, but CloudEvents, tracing, and broker tooling can no longer reason over it consistently. Instead use headers, `correlationId`, and bindings.
- NEVER spec queue names or Kafka `groupId` because it gets your local setup working fast, but it bakes one consumer deployment into a producer-owned contract. Instead spec exchange, topic, and routing behavior and leave consumer deployment details out.
- NEVER set MQTT `retain: true` for domain events because it looks like a free catch-up mechanism, but every new subscriber receives the last transaction as if it were fresh. Instead retain only last-known-value state.
- NEVER upgrade upstream producers before Kafka Streams apps under backward modes because "consumers first" sounds universal, but Streams must still parse its old changelog. Instead upgrade Streams first, then producers.
- NEVER use Protobuf `Any` in public event contracts because it feels future-proof, but common debugging paths and even Confluent's console consumer cannot inspect it cleanly. Instead use explicit `oneof`, references, or versioned wrapper messages.
- NEVER omit `bindingVersion` because parsers accepting defaults feels harmless, but meaning changes underneath you as bindings evolve. Instead pin every binding explicitly.
- NEVER skip channel-level `servers` in mixed-protocol specs because the document still validates, but generators assume the channel exists on every server. Instead scope each channel to the servers that truly carry it.

## Fallbacks

- Tooling stuck on v2: keep v2, adopt application-centric language in prose, and migrate only the documents that need reply or channel reuse.
- No schema registry: put explicit version in headers, add reader-side upcasters, and test against fixtures from real retained events.
- Partner needs HTTP or WebSocket while you run Kafka internally: keep the internal AsyncAPI as source of truth and publish a separate partner-facing contract. Do not export broker topology.
- Existing payload is already polluted with transport fields: freeze the shape, stop adding more leakage, and add an edge translator that moves new metadata into headers or envelope.
- Need a breaking rename immediately: create a new subject or topic, or insert a translator service. Do not compress a four-deploy migration into one release.

## Done Means

- The contract reads from one application's perspective.
- Replay, rollout order, and partner-consumption story are explicit.
- Payload, header, and binding boundaries are clean.
- The spec is wired tightly enough to fail CI when it drifts.
