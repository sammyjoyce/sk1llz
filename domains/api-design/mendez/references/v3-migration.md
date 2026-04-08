# AsyncAPI v2 → v3 Migration Guide

Only migrate if you need something v3 adds. v2 is supported indefinitely; don't migrate for aesthetics.

## The Three Real Reasons to Migrate

1. **Request-reply patterns** — impossible to express in v2; first-class in v3.
2. **Channel reuse** — v2 inlines messages into operations, preventing reuse. v3 separates channels, operations, and messages, so one channel can host many operations from many directions.
3. **Per-application specs** — v2's client/server confusion forced one spec per topic. v3's send/receive perspective lets each application own its own spec cleanly.

If none of these hurt: **stay on v2**. The migration is mechanical but large.

## The Perspective Flip (read this before touching any file)

In v2:
```yaml
# v2: "What can clients do to my server?"
channels:
  user/signedup:
    publish:      # clients publish to me
      message: ...
    subscribe:    # clients subscribe to my output
      message: ...
```

In v3:
```yaml
# v3: "What does MY app do?"
channels:
  userSignedUp:
    address: 'user/signedup'
    messages:
      userSignedUp:
        $ref: '#/components/messages/userSignedUp'

operations:
  sendUserSignedUp:
    action: send            # my app sends
    channel:
      $ref: '#/channels/userSignedUp'
    messages:
      - $ref: '#/channels/userSignedUp/messages/userSignedUp'
```

**The semantic flip is the hard part, not the syntax.** If your v2 spec was written from the "my server" viewpoint (`publish` = "what clients can send me"), then:
- v2 `publish` → v3 `action: receive` (your app receives what clients publish)
- v2 `subscribe` → v3 `action: send` (your app sends what clients subscribe to)

If your v2 spec was already written from the app's viewpoint (which many were, because the v2 model was confusing enough that people inverted it):
- v2 `publish` → v3 `action: send`
- v2 `subscribe` → v3 `action: receive`

**Audit each channel before flipping.** Look at the operationId and summary text. If it says "Handle user signup from frontend," the app *receives*. If it says "Publish user signup event," the app *sends*. Don't mechanically convert; read the intent.

## Structural Changes (mechanical part)

### Channels are now addressed, not keyed

```yaml
# v2
channels:
  user/signedup:   # key IS the topic
    ...

# v3
channels:
  userSignedUp:                     # arbitrary local key
    address: 'user/signedup'        # actual topic
    ...
```

Why: the key is now a reusable identifier for `$ref`. You can have multiple channels with the same address but different semantics, or rename the address without refactoring all refs.

### Messages live on channels, operations reference them

```yaml
# v2: message inlined in operation
channels:
  user/signedup:
    publish:
      message:
        payload:
          ...

# v3: message on channel, operation references
channels:
  userSignedUp:
    address: 'user/signedup'
    messages:
      userSignedUp:
        payload:
          ...

operations:
  receiveUserSignedUp:
    action: receive
    channel:
      $ref: '#/channels/userSignedUp'
    messages:
      - $ref: '#/channels/userSignedUp/messages/userSignedUp'
```

### Servers are referenced, not inlined

```yaml
# v3
channels:
  userSignedUp:
    address: 'user/signedup'
    servers:
      - $ref: '#/servers/production'     # list of refs, not inline
```

### Parameters got a `location` (the big new feature for IoT/multitenant)

```yaml
# v2 — static parameters only
channels:
  smartylighting/streetlights/{streetlightId}/lighting/measured:
    parameters:
      streetlightId:
        schema:
          type: string
        # No way to say "extract from payload"

# v3 — dynamic location
channels:
  lightingMeasured:
    address: 'smartylighting/streetlights/{streetlightId}/lighting/measured'
    parameters:
      streetlightId:
        description: 'ID of the streetlight'
        location: '$message.payload#/streetlightId'   # runtime extraction
```

This is the v3 feature IoT and multitenant teams should migrate for. It lets Microcks and other tools generate per-parameter mocks automatically.

## Request-Reply (the other big reason to migrate)

```yaml
# v3 only — impossible in v2
operations:
  requestQuote:
    action: send
    channel:
      $ref: '#/channels/quoteRequests'
    reply:
      channel:
        $ref: '#/channels/quoteResponses'
      # OR, for dynamic reply channels (reply topic named in request):
      address:
        location: '$message.header#/replyTo'
```

The `location: '$message.header#/replyTo'` form is critical for broker request-reply where the reply topic is generated per request (common pattern: publish to `quotes.requests`, include `replyTo: quotes.responses.<uuid>` in headers, consumer creates the reply topic at runtime).

**v2 had no way to express this.** Teams either documented it out of band or abused `publish`/`subscribe` in ways tools couldn't interpret.

## Schema Format Declaration Changed

```yaml
# v2
message:
  payload:
    # assumed JSON Schema
    type: object
    ...

# v3 — explicit
message:
  payload:
    schemaFormat: 'application/vnd.apache.avro+json;version=1.9.0'
    schema:
      $ref: 'http://registry/schemas/UserSignedUp'
```

v3 makes it easier to use Avro/Protobuf without wrapper ceremony. If your v2 spec had Avro shimmed into JSON Schema, v3 cleans this up.

## Migration Procedure

1. **Snapshot and freeze v2.** Tag the repo; the v2 version must stay consumable while migration happens.
2. **Audit channel perspective.** For each `publish`/`subscribe`, determine if the app sends or receives. Write it as a comment in the v2 file before touching anything.
3. **Extract messages to `components/messages`.** Even in v2, this is good hygiene and prevents copy-paste loss during the migration.
4. **Convert top-down**: `asyncapi: '3.0.0'` → then `channels` → then extract `operations`.
5. **Run the AsyncAPI parser** (`@asyncapi/parser`) after each conversion. It reports v3 errors clearly; fix them before moving on.
6. **Generate from both specs** into a scratch directory and diff the generated code. If the diff surprises you, the migration was wrong.
7. **Keep v2 and v3 in parallel for one full release cycle** so downstream consumers can pick their pace.

## Common Migration Mistakes

- **Leaving `publish`/`subscribe` keywords behind.** They don't exist in v3. Parser errors are loud, but CI might not run the parser.
- **Flipping send/receive without re-reading the intent.** "My app receives from Kafka and sends to HTTP" has two different operations in v3, often two different specs.
- **Forgetting the `address` field.** v2 implicit key-as-address stops working; the spec validates but runtime tooling can't find the topic.
- **Using `servers: [...]` as strings instead of refs.** v3 requires `$ref` objects, not bare strings.
- **Assuming Microcks/SpringWolf/Saunter/Glee support v3 fully.** Check the specific tool's v3 maturity before migrating production specs. Several generators lagged v3 by 18+ months.

## When to Create Two Specs Per Application

Fran's v3 guidance: one spec per application. But for public/partner consumers, you may want **two**:

1. **Internal spec** — describes what your app sends and receives, full fidelity, all channels, used by your own team.
2. **Consumer-facing spec** — describes what the partner should implement on their side (the "flipped" operations), potentially using a different protocol (HTTP, WebSocket) because the partner cannot reach your broker directly.

Keep the internal spec as the source of truth; generate or hand-write the consumer spec and keep it in sync via CI. This is the pattern Fran himself endorsed for public APIs: "you should probably craft the ideal client experience there."

## Post-Migration Checklist

- [ ] No `publish` / `subscribe` keywords remain in any file.
- [ ] Every channel has an explicit `address`.
- [ ] Every operation has explicit `action: send` or `action: receive` from the app's own perspective.
- [ ] Messages are defined on channels and referenced from operations (not inlined).
- [ ] Parser validates cleanly against v3 schema.
- [ ] Code-gen produces the same integration behavior as v2 did (or differences are intentional and documented).
- [ ] All `$ref` targets resolved; no dangling references.
- [ ] v2 spec kept available for consumers still on old tooling.
