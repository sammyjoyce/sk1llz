# Schema Evolution — The Real Compatibility Matrix

Load this before proposing any schema change. The matrix below is what actually happens in production, not the simplified "always add optional fields" advice.

## Compatibility Modes (Confluent Schema Registry terminology, widely adopted)

| Mode | Producer upgrade order | Can rewind to offset 0? | Use when |
|---|---|---|---|
| `BACKWARD` (default) | Upgrade **consumers first**, then producers | ✅ yes | Default. Lets you reprocess full topic. |
| `BACKWARD_TRANSITIVE` | Consumers first | ✅ yes | Long-lived topics, replay mandatory. |
| `FORWARD` | Producers first | ❌ no | Rare. Consumers have version-pinned logic. |
| `FORWARD_TRANSITIVE` | Producers first | ❌ no | Same, but stricter. |
| `FULL` | Either order, latest two schemas only | ⚠️ partial | Short-lived topics with rapid iteration. |
| `FULL_TRANSITIVE` | Either order, all historical schemas | ⚠️ partial | Feels safest but **blocks rewind** — don't pick by default. |
| `NONE` | Coordinated big-bang | ❌ no | Migration windows only; create a new topic instead. |

**Kafka Streams is special**: only `BACKWARD`, `BACKWARD_TRANSITIVE`, `FULL`, `FULL_TRANSITIVE` work. Streams reads from its own changelog (old schema), so **upgrade the Streams app first, then the upstream producer.** Opposite of a plain consumer. This catches teams off guard because the compatibility mode lies: `BACKWARD` normally means "consumers first," but Streams is simultaneously consumer AND producer of its own state.

## Change-by-Change Matrix (what Confluent actually enforces)

| Change | Avro BW | Avro FW | Avro FULL | Protobuf BW | Protobuf FW | Protobuf FULL | JSON Schema (lenient) BW/FW | JSON Schema (strict) BW/FW |
|---|---|---|---|---|---|---|---|---|
| Add optional field w/ default | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅/✅ | **closed/open** ⚠️ |
| Add required field | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌/✅ | ❌/open |
| Remove optional field | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅/✅ | **open/closed** ⚠️ |
| Remove required field | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅/❌ | open/❌ |
| Add union/oneof variant | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅/❌ | ✅/❌ |
| Remove union/oneof variant | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌/✅ | ❌/✅ |
| Widen scalar (int32→int64) | ✅ | ❌ | ❌ | ✅ | ✅ | **✅** | ✅/❌ | ✅/❌ |
| Narrow scalar | ❌ | ✅ | ❌ | ✅ | ✅ | **✅** | ❌/✅ | ❌/✅ |
| Rename field | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌/❌ | ❌/❌ |

**Key gotchas from this table:**

- **Protobuf is the only format where scalar widening/narrowing is fully compatible.** Protobuf encodes type tags, not field types, so int32↔int64↔uint32 all interchange on the wire. Avro cannot do this.
- **JSON Schema has two policies**: "lenient" treats unknown fields permissively (`additionalProperties: true` behavior); "strict" enforces `additionalProperties: false`. A **strict** consumer will reject new optional fields, which is why "add optional" is only partially safe. Confluent defaults lenient; you can override per subject.
- **Removing a required field** is backward compatible in Avro (readers can default it) but not in Protobuf or strict JSON Schema. This trips over teams migrating formats.

## The Four-Deploy Rename Dance

Renaming `customerId` → `accountId` on a live topic:

1. **Deploy A**: producer writes both `customerId` AND `accountId` (same value). Consumers read `customerId`. All green.
2. **Deploy B**: consumers migrate to read `accountId`, falling back to `customerId` if absent. All green.
3. **Wait** for retention to drain old events that have only `customerId` (or use upcasting on read).
4. **Deploy C**: producer stops writing `customerId`.
5. **Deploy D**: remove the `customerId` field from the schema entirely.

Skipping any step breaks someone. There is no shortcut.

## Upcasting for Long-Lived Streams (event sourcing)

Event stores are immutable. You cannot rewrite history. Upcasters transform old event shapes to new ones **in memory at read time**.

**Register upcasters between deserialization and domain logic:**

```python
# events/account_credited.py
class AccountCreditedV1(BaseModel):
    account_id: str
    amount: float

class AccountCreditedV2(BaseModel):
    account_id: str
    amount: float
    currency: Currency   # new required field

class AccountCreditedV3(BaseModel):
    account_id: str
    amount: float
    currency: Currency
    note: str | None     # new optional

def upcast_v1_to_v2(e: AccountCreditedV1) -> AccountCreditedV2:
    return AccountCreditedV2(**e.dict(), currency=Currency.USD)  # sensible default

def upcast_v2_to_v3(e: AccountCreditedV2) -> AccountCreditedV3:
    return AccountCreditedV3(**e.dict(), note=None)

# Central alias always points to latest
AccountCredited = AccountCreditedV3
```

**Non-obvious upcaster rules:**

- **Upcasters must be pure functions.** No I/O, no database lookups, no clock reads. They run on every replay, potentially millions of times.
- **Chain, don't jump.** v1→v2→v3, not v1→v3 directly. Each hop is individually reviewable; jumps hide assumptions.
- **Extract fixtures from production.** Take a representative sample of real events from every version, freeze them in `tests/fixtures/`, and run upcasters over them in CI. This is the only way to catch "this event I wrote in 2019 no longer parses" before it's too late to fix.
- **Never upcast outside the domain boundary.** Upcasting happens *inside* the consuming service, not in a shared library. Each consumer owns its read model.
- **Default values matter legally and ethically.** `Currency.USD` is a safe default for a US-only app but would be silently wrong in an EU app. Document every default's assumption.

## Testing Compatibility Before You Deploy

```bash
# Ask the registry: "Is this new schema compatible with the current subject?"
curl -X POST -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  --data @new-schema.json \
  http://registry:8081/compatibility/subjects/orders.placed-value/versions/latest

# Set compatibility mode per subject (not globally — be surgical)
curl -X PUT -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  --data '{"compatibility": "BACKWARD_TRANSITIVE"}' \
  http://registry:8081/config/orders.placed-value
```

**Put the compatibility check in CI.** Not "we'll check it before deploying" — the PR fails if the schema is incompatible. This is the single highest-ROI automation in the entire event-driven stack.

## The "Add Optional Field Broke Everything" Post-Mortem Template

When a "safe" change breaks prod, check in this order:

1. Is the consumer using **strict** JSON Schema (`additionalProperties: false`)? → Move to lenient or add the field to the consumer's schema.
2. Is the consumer using **Avro with a strict reader schema** that doesn't list the new field? → Avro readers need the field in their own schema even to ignore it.
3. Is **Kafka Streams** involved? → The changelog has old events; upgrade order is inverted.
4. Is the event being **replayed** from an old offset? → Upcast old events on read.
5. Is a **middleware/translator** rewriting events in flight? → It may be filtering your new field.

If none of the above, the change wasn't safe. Revert first, diagnose second.
