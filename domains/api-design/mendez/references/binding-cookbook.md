# AsyncAPI Binding Cookbook

Exact field names, binding versions (as of 2025), and the traps each binding hides. Pin `bindingVersion` explicitly — parsers silently accept any value and apply defaults.

## Kafka (`bindingVersion: 0.5.0`)

### Server binding
```yaml
servers:
  kafkaServer:
    host: kafka.prod.internal:9092
    protocol: kafka-secure        # or 'kafka' for plaintext
    protocolVersion: '3.5'
    bindings:
      kafka:
        schemaRegistryUrl: 'https://registry.prod.internal'
        schemaRegistryVendor: 'confluent'   # or 'apicurio'
        bindingVersion: '0.5.0'
```

### Channel binding (this is where you spec the topic)
```yaml
channels:
  orderPlaced:
    address: 'orders.placed.v1'   # actual topic name; channel key is just local
    bindings:
      kafka:
        topic: 'orders.placed.v1' # redundant with address; set both for clarity
        partitions: 12
        replicas: 3
        topicConfiguration:
          cleanup.policy: ['delete']
          retention.ms: 604800000       # 7 days
          max.message.bytes: 1048588    # 1MB, Kafka's default hard limit
        bindingVersion: '0.5.0'
```

### Operation binding
```yaml
operations:
  sendOrderPlaced:
    action: send
    channel:
      $ref: '#/channels/orderPlaced'
    bindings:
      kafka:
        clientId:
          type: string
          enum: ['order-service']
        bindingVersion: '0.5.0'
```

### Message binding (the `key` field is a schema, not a value)
```yaml
components:
  messages:
    orderPlaced:
      bindings:
        kafka:
          key:
            type: string
            description: 'customerId — partitions by customer for ordering'
          schemaIdLocation: 'header'        # or 'payload'
          schemaIdPayloadEncoding: 'confluent'  # or 'apicurio-new', 'apicurio-legacy'
          schemaLookupStrategy: 'TopicIdStrategy'
          bindingVersion: '0.5.0'
```

### Kafka traps
- **`key` is documentation of partition-key shape, not the literal key.** Consumers must still read the key from the ConsumerRecord. Don't put it in the payload "for convenience."
- **`groupId` should NOT be in the spec.** It's a consumer deployment decision. Specifying it forces every consumer team to use your group (wrong) or ignore the spec (also wrong).
- **`max.message.bytes: 1048588`** is Kafka's default 1 MB hard limit. If your fat event approaches this, you've crossed into "should be an S3 reference" territory. Don't raise the limit — shrink the event.
- **`schemaIdLocation: 'payload'`** means the first 5 bytes of the message value are the Confluent magic byte + schema ID. Consumers that bypass the Confluent deserializer will read garbage if they forget this.
- **`cleanup.policy: compact`** + event sourcing = data loss. Compacted topics keep only the latest per key; old events disappear. Never compact an event store.

## AMQP 0-9-1 / RabbitMQ (`bindingVersion: 0.3.0`, key: `amqp`)

```yaml
channels:
  orderEvents:
    address: 'orders'   # the exchange name
    bindings:
      amqp:
        is: routingKey
        exchange:
          name: 'orders'
          type: topic       # direct | topic | fanout | headers
          durable: true
          autoDelete: false
          vhost: '/'
        bindingVersion: '0.3.0'

operations:
  sendOrderPlaced:
    action: send
    bindings:
      amqp:
        expiration: 100000
        mandatory: true
        deliveryMode: 2        # 2 = persistent, 1 = transient
        bindingVersion: '0.3.0'
```

### AMQP traps
- **`amqp` binding is 0-9-1 only.** AMQP 1.0 is `amqp1` — a completely different binding. Solace, Azure Service Bus, ActiveMQ Artemis default to 1.0. Picking the wrong one silently validates but cannot describe real wire behavior.
- **Never spec the queue.** Queues are consumer concerns. Spec the exchange + routing key; let consumers bind their own queues.
- **`deliveryMode: 1`** (transient) loses messages on broker restart. Use `2` for anything business-critical.
- **`mandatory: true`** returns unroutable messages to the producer. Without it, messages hit the void silently when no queue is bound.
- **`is: routingKey`** vs `is: queue` — if you accidentally use `queue`, you're describing a direct queue binding, not an exchange. Very different semantics.

## MQTT 3.1.1 (`bindingVersion: 0.2.0`, key: `mqtt`) and MQTT 5 (`mqtt5`)

```yaml
operations:
  receiveDeviceTelemetry:
    action: receive
    bindings:
      mqtt:
        qos: 1              # 0 fire-and-forget | 1 at-least-once | 2 exactly-once
        retain: false       # CRITICAL — see below
        bindingVersion: '0.2.0'
```

### MQTT traps
- **`retain: true` = broker remembers the last message on the topic and delivers it to every new subscriber.** Perfect for device state (`device/42/status` → "online"). **Catastrophic** for transaction events — every new consumer reprocesses the stored "last" event as if it were live. Default to `false`; only flip to `true` for last-known-value semantics.
- **`qos: 2` (exactly-once) is expensive**: four-way handshake per message, broker state per in-flight. Use only when duplicates are dangerous (financial, medical). `qos: 1` + idempotent consumers is the usual answer.
- **MQTT 5 (`mqtt5`) adds user properties** — the headers-equivalent. If you need CloudEvents context attributes, MQTT 5 is required; MQTT 3.1.1 has no headers at all.
- **Wildcards (`+`, `#`)** are subscription patterns, not valid in published topics or spec addresses.

## WebSocket (`bindingVersion: 0.1.0`, key: `ws`)

```yaml
servers:
  chat:
    host: ws.example.com
    protocol: wss
    bindings:
      ws:
        query:
          type: object
          properties:
            token:
              type: string
        headers:
          type: object
          properties:
            X-Client-Id:
              type: string
        bindingVersion: '0.1.0'
```

### WebSocket traps
- **Bindings apply to the handshake, not the messages.** Per-message bindings are mostly empty. The protocol-specific stuff (auth, subprotocol) lives in server bindings.
- **A single WebSocket channel carries heterogeneous messages.** Use `oneOf` in the message schema and a discriminator field. Don't try to spec one channel per message type — WebSocket multiplexes at the app level.

## NATS (`bindingVersion: 0.1.0`, key: `nats`)

NATS subjects use `.` as separator and `*`/`>` as wildcards. Spec the **pattern**, not a specific subject:

```yaml
channels:
  orderEvents:
    address: 'orders.*.placed'   # consumers can subscribe to patterns
    bindings:
      nats:
        queue: 'order-processors' # optional queue group for load balancing
        bindingVersion: '0.1.0'
```

### NATS traps
- **JetStream vs Core NATS are different worlds.** Core is fire-and-forget; JetStream is persistent with acks, replay, and consumers. A spec that works for one may be meaningless for the other.
- **Subjects are hierarchical.** `orders.>` matches `orders.placed`, `orders.shipped.eu`, etc. Don't use flat names; you give up routing flexibility.

## Pulsar (`bindingVersion: 0.1.0`, key: `pulsar`)

```yaml
servers:
  pulsar:
    bindings:
      pulsar:
        tenant: 'retail'
        bindingVersion: '0.1.0'
channels:
  orderPlaced:
    address: 'persistent://retail/orders/placed'   # full Pulsar topic URI
    bindings:
      pulsar:
        namespace: 'orders'
        persistence: persistent
        compaction: 1000000
        geo-replication: ['us-east', 'eu-west']
        retention: { time: 7, size: 1000 }
        ttl: 360
        deduplication: true
        bindingVersion: '0.1.0'
```

### Pulsar traps
- **`persistent://` vs `non-persistent://`** is encoded in the topic URI. Setting `persistence: persistent` in bindings while the address is `non-persistent://` creates a lie.
- **Tenant and namespace** are in the address AND in bindings. They must match.

## Microcks Compatibility (for mock generation)

Microcks uses AsyncAPI examples to generate realistic mocks, but has quirks:

- **Default binding is Kafka** if none specified. If you target MQTT but forget to declare the binding, Microcks will silently create a Kafka destination.
- **v3 examples** don't need `name` — Microcks computes one from message name + index.
- **Dynamic parameter location** (`location: $message.payload#/fieldName`) only works in v3 Microcks. In v2, you must enumerate examples with static values per parameter.
- **`x-microcks-operation: frequency: 30`** makes Microcks auto-publish mock messages every 30 seconds — useful for integration testing but will drown a shared dev broker.

## Universal Binding Review Checklist

Before merging any spec with bindings, verify:

1. **`bindingVersion` is explicit** on every binding block.
2. **No queue names, no consumer groups** in the spec (consumer-side concerns).
3. **Address vs channel key** — address is the real broker topic, channel key is just a local identifier.
4. **Message `key` on Kafka is a schema**, describing the partition key shape, not a literal value.
5. **`retain`/`durable`/`persistence`** match the business requirement, not "defaults look fine."
6. **Binding protocol version** (`amqp` vs `amqp1`, `mqtt` vs `mqtt5`) matches the actual broker, not a guess.
7. **Max message size** on Kafka is at or below `1048588` (the default 1 MB). If above, redesign toward thin events + S3/object-store references.
