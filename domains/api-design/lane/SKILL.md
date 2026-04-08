---
name: lane-api-evangelist
description: >
  Design APIs using Kin Lane's "API Evangelist" philosophy. Emphasizes
  Design-First (OpenAPI), governance as enablement, treating APIs as products
  with business lifecycles, and the political/business impact of interfaces.
  Use when designing public API platforms, standing up API governance programs,
  auditing enterprise API consistency, planning API deprecation/versioning
  strategy, or building large-scale internal API ecosystems. Trigger keywords:
  API governance, OpenAPI spec, Spectral linting, API lifecycle, API-as-product,
  design-first, API deprecation, API versioning strategy, API portal, developer
  experience, API commons, API contract.
---

# Lane API Evangelist Style

## Core Mindset

The contract is a political document. Every field you expose, every error format
you choose, every version you deprecate is a business decision with
organizational consequences. The spec is not a tech artifact—it's a negotiated
agreement between teams who may never speak to each other.

Before designing any API, ask yourself:
- **Who are the consumers I've never met?** Public APIs get embedded by
  developers who integrate at 2 AM and never touch the code again for three
  years. Design for the person who will never read your changelog.
- **What happens when this API outlives my team?** If `info.contact` is empty,
  the API becomes a zombie portal within 18 months—discoverable but unowned.
- **Am I governing or policing?** Governance that developers revolt against is
  worse than no governance. Every rule needs a "why" that a skeptical engineer
  would accept.
- **Is this spec reviewable by a non-engineer?** Product managers and business
  stakeholders must be able to review the contract. If only engineers understand
  it, you've already lost the "design" in design-first.

## Decision Tree: What Kind of API Work Is This?

```
New API from scratch?
  → MANDATORY: Read references/openapi-contract-template.md before writing YAML.
  → Do NOT load references/spectral-governance.md for this task.

Standing up / auditing a governance program?
  → MANDATORY: Read references/spectral-governance.md entirely.
  → Do NOT load references/openapi-contract-template.md for this task.

Auditing an existing API spec?
  → Use the Audit Checklist below. Load references as needed.

Planning deprecation or versioning?
  → Use the Deprecation Framework below. No reference files needed.

Adding governance to an existing CI pipeline?
  → Start with IDE integration (VS Code Spectral extension) not CI blocking.
  → Read references/spectral-governance.md for rule tier strategy.
```

## Expert Heuristics

### Governance Rollout (Lane's Bloomberg Experience)

Lane stood up governance across a 25,000-person org with several hundred APIs,
half Swagger 2.0, half OpenAPI 3.x, plus async and GraphQL. Key lessons:

- **Start with ≤12 Spectral rules.** Launching with a large ruleset guarantees
  revolt. Deploy Tier 0 as warnings for 4 weeks, promote to errors only after
  zero false positives. One false positive destroys governance credibility
  exponentially—teams distrust all rules after one bad rule.
- **Every negative rule needs a positive counterpart.** Don't only flag missing
  descriptions—also emit `info`-level messages when descriptions exceed 50
  characters. This reframes governance from policing to enablement. Research
  shows pointing out errors without guidance rarely leads to improvement.
- **Group rules by policy, not by spec location.** Don't organize as "paths
  rules" vs "schema rules." Organize as "Every API must be discoverable" and
  "Every API must be safely consumable." Policies connect rules to business
  value that leadership understands. Spec-location grouping is meaningless to
  anyone outside the API team.
- **The Spectral rule hierarchy matters.** Before writing custom JS functions:
  (1) JSON Schema validation, (2) Spectral default functions, (3) Spectral JSON
  Schema function for modular rules, (4) custom functions only when nothing else
  works. Most teams jump to custom functions without mastering the defaults.
- **Layer rulesets with metadata dimensions.** Organize rules by policy (why),
  lifecycle stage (when), platform (where), and solution (what problem). This
  prevents the all-or-nothing problem where teams face every rule on every API
  regardless of maturity or context.
- **80% of engineers comply when they know the standards.** The remaining 20%
  question standards that lack rationale. Publish the "why" alongside every
  rule. If you can't articulate why, drop the rule.

### Deprecation Framework

Lane's real-world story: a print API with only 5 consumers on the oldest
version. They migrated 4. The 5th submitted a single weekly job worth $500K.
The original developer had left. The company couldn't migrate.

**The procedure most teams skip:** Track deprecated endpoint usage by API key
for ≥30 days before committing to any removal timeline. Contact high-volume
consumers directly. Deprecation is a business decision, not an engineering one.

| Change type | Minimum notice | Recommended |
|---|---|---|
| Field removal | 3 months | 6 months |
| Endpoint removal | 6 months | 12 months |
| Major version sunset | 12 months | 18–24 months |
| Security-critical removal | 30 days | 90 days |

After sunset, return `410 Gone` with `application/problem+json` body containing
`successor` URL and `documentation` link—never `404`, which is ambiguous.

### Versioning Traps

- **URL versioning: major versions only.** `/v1/`, `/v2/`. Never `/v1.2/`—minor
  versions in URLs force consumers to redeploy for non-breaking changes.
- **Version lives in `servers[].url`, not in path segments.** Putting versions
  in paths (`/v1/books`) tempts teams to cram v1 and v2 into one OpenAPI doc,
  causing accidental cross-version `$ref` leaks.
- **One API version per OpenAPI document.** Two `servers` entries with different
  versions in one file guarantees shared schemas drift silently. The moment you
  see `v1` and `v2` in the same file, split immediately.

### Spec Quality Signals Only Experts Check

- **`operationId` is a codegen contract.** Changing it breaks every generated
  SDK. Treat as immutable after first publish. Use `verbNoun` (`listBooks`,
  `createOrder`). This is the single most expensive field to change post-launch.
- **Separate input schemas from output schemas.** `readOnly: true` on shared
  schemas confuses codegen—some generators emit write models with read-only
  fields. Use `BookInput` and `Book` explicitly.
- **Cursor pagination > offset pagination.** Offset breaks when rows are
  inserted/deleted mid-traversal. For any dataset that may exceed 1,000 rows,
  use keyset/cursor pagination.
- **`application/problem+json` (RFC 9457) for all errors.** The `type` URI must
  be a real resolvable URL documenting the error. Custom error envelopes create
  per-API integration tax that compounds across an enterprise.
- **Make pricing, rate limits, and docs machine-readable.** Lane uses APIs.json
  to make the entire portal experience discoverable—not just the endpoints.
  Standard JSON Schema for pricing pages and rate limit pages means crawlers and
  AI agents can consume your API metadata programmatically.

## NEVER List

NEVER auto-generate the OpenAPI spec from code and call it "design-first."
Code-first specs inherit implementation details (internal field names, database
column leakage like `created_at`), produce specs no stakeholder reviews, and
couple the API permanently to one language's serialization quirks. The concept
of design-first originated from Apiary's deliberate separation of spec from
implementation. Instead, write the spec in YAML first, get stakeholder sign-off,
then generate server stubs.

NEVER ship an OpenAPI spec without `info.contact` containing a real team email.
APIs without ownership metadata become zombie portals—discoverable in the
catalog but unowned. Within 18 months nobody knows who to ask when it breaks.
This is the #1 cause of enterprise API catalog rot.

NEVER promote a Spectral rule to `error` severity until it has produced zero
false positives for 4 consecutive weeks in `warn` mode. False positives destroy
governance credibility exponentially. Teams that encounter one bad rule begin
ignoring all rules, and recovering trust takes 6+ months.

NEVER govern everything. Governance programs with 200+ rules have diminishing
returns—teams game the system with minimal-compliance specs that pass linting
but are unusable. Govern the 15–20 things that matter. Lane's rule: if a rule
can't be tied to a business policy a VP would endorse, drop it.

NEVER put deprecation decisions in engineering alone. A consumer doing $500K/week
through a legacy endpoint is a stakeholder. Check usage analytics by API key and
contact high-volume consumers directly before publishing any sunset timeline.
The cost of surprising a high-value consumer exceeds any engineering convenience.

NEVER use `404` for sunset endpoints. `404` is ambiguous—resource never existed
vs. removed. Use `410 Gone` with `application/problem+json` body pointing to
the successor. This is the only signal that lets automated clients self-heal.

NEVER chase the latest API paradigm (GraphQL, MCP) by throwing away existing
REST investment. REST is still the simplest, cheapest, widest-audience option
with the most governance and documentation tooling. Evaluate new paradigms as
additions to the toolbox, not replacements. An API team that rewrites a
functioning REST API "because GraphQL" will spend 6 months rebuilding what
already worked.

## Audit Checklist (Existing Specs)

| Dimension | Check | Red flag |
|---|---|---|
| Ownership | `info.contact` with real email | Missing or placeholder |
| Descriptions | Every operation, param, schema has prose | >30% undescribed |
| Consistency | Uniform naming (`snake_case` or `camelCase`, not both) | Mixed conventions |
| Error format | All 4xx/5xx use `application/problem+json` | Custom error envelopes |
| Leaky abstractions | No internal IDs, timestamps, or DB columns in public schemas | `created_at`, `_id` exposed |
| Pagination | Collections use cursor-based pagination | Offset-only on large sets |
| Versioning | Single version per document; version in `servers` URL | Multi-version paths |
| Security | Auth described; no secrets in examples | API keys in query params |
| Reusability | Shared schemas in `components`; `$ref` for repeated structures | Copy-pasted schemas |
| operationId | Present, immutable, `verbNoun` format | Missing or inconsistent |

**Triage:** >3 red flags → recommend a governance program before further dev.
>6 red flags → the spec needs a rewrite, not patching.

## Fallback Strategies

| Primary approach fails... | Do this instead |
|---|---|
| Stakeholders won't review the spec | Generate a mock server and demo the API interactively—stakeholders respond to running software, not YAML |
| Teams resist Spectral in CI | Start with IDE integration only (VS Code Spectral extension)—lower friction, same learning |
| No OpenAPI exists for a running API | Capture traffic with a proxy, generate a draft spec, then refine manually against actual responses |
| Org has Swagger 2.0 only | Convert with `swagger2openapi`, then incrementally improve; don't attempt a full rewrite |
| Governance program stalls politically | Shift from enforcement to storytelling—generate reports showing improvement trends, not failure counts |
| Design-first process feels slow | The spec IS the prototype. Run mock servers from the spec to prove speed parity with code-first |
| Too many APIs to catalog | Use APIs.json crawlers on GitHub and internal repos to auto-discover specs; manual cataloging doesn't scale past ~50 APIs |
