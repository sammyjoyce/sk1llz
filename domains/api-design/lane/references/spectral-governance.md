# Spectral Governance Strategy u2014 Lane Style

This reference covers the expert-level Spectral governance approach Lane developed
across Bloomberg, Postman, and enterprise clients. Load when **setting up or
auditing an API governance program**.

## The Minimum Viable Ruleset (Start Here)

Lane's Bloomberg experience showed that governance programs die when they launch
with 200+ rules. Start with the **base 12** that produce zero false-positive
push-back, then expand by policy tier.

### Tier 0 u2014 Non-negotiable (block merge)

| Rule | Why it matters |
|---|---|
| `info-contact` | Without ownership, APIs become zombies within 18 months |
| `operation-operationId` | Missing operationId breaks every codegen target |
| `oas3-api-servers` | No server URL = spec is un-testable |
| `operation-description` | Undescribed operations are undiscoverable |
| `no-eval` / `no-script-tags-in-markdown` | Security baseline |

### Tier 1 u2014 Design consistency (warn in CI, block after 90 days)

| Rule | Why |
|---|---|
| `paths-kebab-case` | Naming divergence is the #1 cross-team friction |
| `request-body-on-get` | GET with body breaks caches and many client libraries |
| `typed-enum` | Untyped enums break codegen in strongly-typed languages |
| `no-$ref-siblings` | Sibling keywords next to `$ref` are silently ignored in OAS 3.0 |

### Tier 2 u2014 Operational maturity (advisory only initially)

| Rule | Why |
|---|---|
| `pagination-required-for-lists` (custom) | Unpaginated lists create P0 incidents at scale |
| `rate-limit-response-required` (custom) | Consumers without 429 handling cause cascading failures |
| `sunset-header-on-deprecated` (custom) | RFC 8594 Sunset header enables automated migration tracking |

## Policy u2192 Rules u2192 Lifecycle Mapping

Lane's key insight: atomic Spectral rules are useless without **policy grouping**
and **lifecycle placement**.

```
Policy: "Every API must be discoverable"
  u2514u2500 Rule: info-contact (design-time)
  u2514u2500 Rule: info-description (design-time)
  u2514u2500 Rule: operation-tags (design-time)
  u2514u2500 Rule: apis-json-entry (build-time, custom)

Policy: "Every API must be safely consumable"
  u2514u2500 Rule: rate-limit-response-required (design-time)
  u2514u2500 Rule: pagination-required-for-lists (design-time)
  u2514u2500 Rule: no-breaking-changes (CI-time, Optic)
```

## Positive Rules (Lane's Bloomberg Innovation)

Most governance focuses on what's wrong. Lane insists **every negative rule must
have a positive counterpart** that reinforces correct behavior.

```yaml
# Negative rule: warns when description is missing
operation-description:
  severity: warn
  message: "Operation is missing a description."

# Positive rule: celebrates when description exceeds quality threshold
operation-description-quality:
  severity: info
  message: "u2705 Excellent: operation description exceeds 50 characters."
  given: "$.paths[*][*].description"
  then:
    function: length
    functionOptions:
      min: 50
```

This reframes governance from policing to enablement.

## When to Use JSON Schema vs. Spectral

| Use JSON Schema when... | Use Spectral when... |
|---|---|
| Validating request/response payloads at runtime | Linting the OpenAPI document at design-time |
| Enforcing structural shape of data | Enforcing organizational conventions |
| You need `if/then/else` conditional validation | You need JSON Path traversal across the spec |
| The rule is about the API's data contract | The rule is about the API's metadata/hygiene |

Confusing these two is the #1 source of brittle governance rules that produce
false positives.

## Custom Functions: When (and When Not)

Reach for custom Spectral functions **only** when:
1. You've exhausted the default function library (pattern, length, schema, truthy, etc.)
2. The rule requires cross-referencing multiple spec locations
3. You've validated the JSON Schema for the custom function itself

NEVER write a custom function for something a `pattern` or `truthy` check handles.
Custom functions are maintenance debtu2014every one must be tested separately and
kept compatible across Spectral version bumps.

## Rollout Timeline That Actually Works

| Week | Action | Mode |
|---|---|---|
| 1u20132 | Deploy Tier 0 rules in advisory mode | `warn` only |
| 3u20134 | Hold design review office hours; address false positives | `warn` only |
| 5u20136 | Promote Tier 0 to `error` (block merge) | `error` for Tier 0 |
| 7u201310 | Add Tier 1 in `warn` mode | `warn` for Tier 1 |
| 11u201312 | Promote Tier 1 to `error` | `error` for Tier 0+1 |
| 13+ | Introduce Tier 2 as `info` (advisory) | `info` for Tier 2 |

Key insight: **never promote a rule to `error` until zero false positives for
4 consecutive weeks.** False positives destroy governance credibility faster than
any policy benefit.

## Governing Beyond OpenAPI

Lane's approach extends Spectral to non-OpenAPI artifacts:

- **APIs.json** u2014 lint the discovery index for completeness
- **AsyncAPI** u2014 apply equivalent rules for event-driven APIs
- **Custom lifecycle schemas** u2014 lint deployment manifests, pricing pages, etc.

The linting engine is just a tool; the policies transcend any single spec format.
